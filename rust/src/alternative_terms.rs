//! Frozen, occurrence-specific one-time choices. No database writes or reruns.
use crate::RuntimeResponse;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate {
    pub candidate_id: String,
    pub target_text: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Occurrence {
    pub occurrence_id: String,
    pub state: String,
    pub source_text: String,
    pub source_span: [usize; 2],
    pub stage_input_span: [usize; 2],
    pub output_span: [usize; 2],
    pub expected_text: String,
    pub selected_candidate_id: Option<String>,
    pub candidates: Vec<Candidate>,
}

impl Occurrence {
    pub(crate) fn assign_id(&mut self, index: usize) {
        let id = format!("occurrence-{}", index + 1);
        let selected = self.selected_candidate_id.clone();
        for (index, candidate) in self.candidates.iter_mut().enumerate() {
            let previous = candidate.candidate_id.clone();
            candidate.candidate_id = format!("{id}/candidate-{}", index + 1);
            if selected.as_ref() == Some(&previous) {
                self.selected_candidate_id = Some(candidate.candidate_id.clone());
            }
        }
        self.occurrence_id = id;
    }

    pub(crate) fn offset(&mut self, input: usize, output: usize) {
        self.source_span = self.source_span.map(|value| value + input);
        self.output_span = self.output_span.map(|value| value + output);
    }
}

pub(crate) fn build_choice(
    rows: impl IntoIterator<Item = (String, usize, i64)>,
    source_text: &str,
    source_span: [usize; 2],
    stage_input_span: [usize; 2],
    output_span: [usize; 2],
    current: &str,
) -> Occurrence {
    let mut best = BTreeMap::<String, (usize, i64)>::new();
    for (target, level, priority) in rows {
        let rank = (level, priority);
        best.entry(target)
            .and_modify(|old| {
                if level < old.0 || (level == old.0 && priority > old.1) {
                    *old = rank;
                }
            })
            .or_insert(rank);
    }
    let mut targets: Vec<_> = best.into_iter().collect();
    targets.sort_by(|a, b| {
        a.1 .0
            .cmp(&b.1 .0)
            .then_with(|| b.1 .1.cmp(&a.1 .1))
            .then_with(|| a.0.cmp(&b.0))
    });
    let top = targets[0].1;
    let tied = targets.iter().filter(|(_, rank)| *rank == top).count() > 1;
    let candidates = targets
        .into_iter()
        .enumerate()
        .map(|(index, (target_text, rank))| Candidate {
            candidate_id: format!("candidate-{}", index + 1),
            target_text,
            status: if rank != top {
                "also_valid"
            } else if tied {
                "needs_decision"
            } else {
                "recommended"
            }
            .into(),
        })
        .collect();
    Occurrence {
        occurrence_id: String::new(),
        state: if tied {
            "needs_decision"
        } else {
            "recommended"
        }
        .into(),
        source_text: source_text.into(),
        source_span,
        stage_input_span,
        output_span,
        expected_text: current.into(),
        selected_candidate_id: (!tied).then(|| "candidate-1".into()),
        candidates,
    }
}

pub(crate) fn assign_event_ids(events: &mut [serde_json::Value]) {
    let mut indexes: Vec<_> = events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            event
                .get("choice")
                .map(|choice| (index, choice["output_span"][0].as_u64().unwrap_or(0)))
        })
        .collect();
    indexes.sort_by_key(|(_, start)| *start);
    for (order, (index, _)) in indexes.into_iter().enumerate() {
        let mut choice: Occurrence =
            serde_json::from_value(events[index]["choice"].clone()).expect("core-generated choice");
        choice.assign_id(order);
        events[index]["choice"] = serde_json::to_value(choice).expect("serializable choice");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceIntent {
    UseThisTimeOnly,
    RememberForThisContext,
    RememberForAllContexts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceError {
    StaleRevision,
    InvalidTracking,
    UnknownOccurrence,
    InvalidCandidate,
    UnsupportedIntent,
    NoUndo,
    RevisionExhausted,
}
impl std::fmt::Display for ChoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::StaleRevision => "Stale review revision; refresh review",
            Self::InvalidTracking => "Occurrence tracking is stale or invalid; refresh review",
            Self::UnknownOccurrence => "Unknown occurrence; refresh review",
            Self::InvalidCandidate => "Candidate does not belong to this occurrence",
            Self::UnsupportedIntent => "Choice intent is not implemented; nothing was saved",
            Self::NoUndo => "No one-time choice to undo",
            Self::RevisionExhausted => "Review revision exhausted; start a new review",
        })
    }
}
impl std::error::Error for ChoiceError {}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReviewSnapshot {
    pub text: String,
    pub revision: u64,
    pub occurrences: Vec<Occurrence>,
}

#[derive(Clone)]
struct Edit {
    index: usize,
    previous: Occurrence,
    expected: Occurrence,
}

pub struct ReviewSession {
    text: String,
    occurrences: Vec<Occurrence>,
    revision: u64,
    undo: Vec<Edit>,
}

impl ReviewSession {
    /// Trusted core response only: never reconstruct candidate sets in a frontend.
    pub fn from_response(response: &RuntimeResponse) -> Result<Self, ChoiceError> {
        let mut occurrences: Vec<Occurrence> = response
            .changes
            .iter()
            .filter_map(|event| event.get("choice"))
            .map(|value| {
                serde_json::from_value(value.clone()).map_err(|_| ChoiceError::InvalidTracking)
            })
            .collect::<Result<_, _>>()?;
        occurrences.sort_by_key(|item| item.output_span);
        if occurrences.iter().any(|item| {
            item.source_span[0] >= item.source_span[1]
                || item.source_span[1] > response.input.chars().count()
                || item.stage_input_span[0] >= item.stage_input_span[1]
        }) {
            return Err(ChoiceError::InvalidTracking);
        }
        let session = Self {
            text: response.output.clone(),
            occurrences,
            revision: 0,
            undo: Vec::new(),
        };
        session.validate()?;
        Ok(session)
    }
    pub fn snapshot(&self) -> ReviewSnapshot {
        ReviewSnapshot {
            text: self.text.clone(),
            revision: self.revision,
            occurrences: self.occurrences.clone(),
        }
    }
    fn validate(&self) -> Result<(), ChoiceError> {
        let mut end = 0;
        let mut ids = BTreeSet::new();
        for item in &self.occurrences {
            let [start, stop] = item.output_span;
            if start < end
                || start > stop
                || char_slice(&self.text, item.output_span) != Some(item.expected_text.as_str())
                || item.occurrence_id.is_empty()
                || !ids.insert(&item.occurrence_id)
            {
                return Err(ChoiceError::InvalidTracking);
            }
            end = stop;
            let mut candidate_ids = BTreeSet::new();
            let mut targets = BTreeSet::new();
            if item.candidates.is_empty()
                || item.candidates.iter().any(|candidate| {
                    !candidate
                        .candidate_id
                        .starts_with(&format!("{}/", item.occurrence_id))
                        || !candidate_ids.insert(&candidate.candidate_id)
                        || !targets.insert(&candidate.target_text)
                        || !matches!(
                            candidate.status.as_str(),
                            "recommended" | "also_valid" | "needs_decision"
                        )
                })
            {
                return Err(ChoiceError::InvalidTracking);
            }
            match item.state.as_str() {
                "needs_decision"
                    if item.selected_candidate_id.is_none()
                        && item.expected_text == item.source_text => {}
                "recommended" | "chosen"
                    if item.candidates.iter().any(|candidate| {
                        Some(&candidate.candidate_id) == item.selected_candidate_id.as_ref()
                            && candidate.target_text == item.expected_text
                    }) => {}
                _ => return Err(ChoiceError::InvalidTracking),
            }
        }
        Ok(())
    }
    fn check(&self, revision: u64) -> Result<u64, ChoiceError> {
        if revision != self.revision {
            return Err(ChoiceError::StaleRevision);
        }
        self.validate()?;
        self.revision
            .checked_add(1)
            .ok_or(ChoiceError::RevisionExhausted)
    }
    fn replace(&mut self, index: usize, replacement: &str) {
        let [start, end] = self.occurrences[index].output_span;
        let start_byte = byte_offset(&self.text, start).expect("validated span");
        let end_byte = byte_offset(&self.text, end).expect("validated span");
        self.text.replace_range(start_byte..end_byte, replacement);
        let length = replacement.chars().count();
        self.occurrences[index].output_span = [start, start + length];
        self.occurrences[index].expected_text = replacement.into();
        for later in &mut self.occurrences[index + 1..] {
            later.output_span = later.output_span.map(|value| {
                if length >= end - start {
                    value + (length - (end - start))
                } else {
                    value - ((end - start) - length)
                }
            });
        }
    }
    pub fn apply_choice(
        &mut self,
        revision: u64,
        occurrence_id: &str,
        candidate_id: &str,
        intent: ChoiceIntent,
    ) -> Result<ReviewSnapshot, ChoiceError> {
        if intent != ChoiceIntent::UseThisTimeOnly {
            return Err(ChoiceError::UnsupportedIntent);
        }
        let next = self.check(revision)?;
        let index = self
            .occurrences
            .iter()
            .position(|item| item.occurrence_id == occurrence_id)
            .ok_or(ChoiceError::UnknownOccurrence)?;
        let candidate = self.occurrences[index]
            .candidates
            .iter()
            .find(|candidate| candidate.candidate_id == candidate_id)
            .ok_or(ChoiceError::InvalidCandidate)?
            .clone();
        let previous = self.occurrences[index].clone();
        self.replace(index, &candidate.target_text);
        self.occurrences[index].selected_candidate_id = Some(candidate_id.into());
        self.occurrences[index].state = "chosen".into();
        self.undo.push(Edit {
            index,
            previous,
            expected: self.occurrences[index].clone(),
        });
        self.revision = next;
        Ok(self.snapshot())
    }
    pub fn undo(&mut self, revision: u64) -> Result<ReviewSnapshot, ChoiceError> {
        let next = self.check(revision)?;
        let edit = self.undo.last().ok_or(ChoiceError::NoUndo)?.clone();
        if self.occurrences[edit.index] != edit.expected {
            return Err(ChoiceError::InvalidTracking);
        }
        self.replace(edit.index, &edit.previous.expected_text);
        self.occurrences[edit.index] = edit.previous;
        self.undo.pop();
        self.revision = next;
        Ok(self.snapshot())
    }
}

fn byte_offset(text: &str, position: usize) -> Option<usize> {
    if position == text.chars().count() {
        Some(text.len())
    } else {
        text.char_indices().nth(position).map(|(offset, _)| offset)
    }
}
fn char_slice(text: &str, [start, end]: [usize; 2]) -> Option<&str> {
    if start > end {
        return None;
    }
    text.get(byte_offset(text, start)?..byte_offset(text, end)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> ReviewSession {
        let mut choice = build_choice(
            [("科技詞".into(), 0, 10), ("短".into(), 0, 1)],
            "測試詞",
            [0, 3],
            [0, 3],
            [0, 3],
            "科技詞",
        );
        choice.assign_id(0);
        let mut later = choice.clone();
        later.assign_id(1);
        later.offset(3, 3);
        ReviewSession {
            text: "科技詞科技詞".into(),
            occurrences: vec![choice, later],
            revision: 0,
            undo: Vec::new(),
        }
    }

    #[test]
    fn corrupt_anchor_overlap_bounds_and_lost_candidates_reject_atomically() {
        for kind in ["anchor", "overlap", "bounds", "candidates"] {
            let mut session = session();
            match kind {
                "anchor" => session.text = "錯字詞科技詞".into(),
                "overlap" => session.occurrences[1].output_span = [2, 5],
                "bounds" => session.occurrences[0].output_span = [0, usize::MAX],
                _ => session.occurrences[0].candidates.clear(),
            }
            let before = session.snapshot();
            assert_eq!(
                session
                    .apply_choice(
                        0,
                        "occurrence-1",
                        "occurrence-1/candidate-1",
                        ChoiceIntent::UseThisTimeOnly
                    )
                    .unwrap_err(),
                ChoiceError::InvalidTracking
            );
            assert_eq!(session.snapshot(), before);
            assert_eq!(session.undo(0).unwrap_err(), ChoiceError::InvalidTracking);
            assert_eq!(session.snapshot(), before);
        }
    }

    #[test]
    fn stale_undo_anchor_and_revision_exhaustion_do_not_mutate_history() {
        let mut session = session();
        session
            .apply_choice(
                0,
                "occurrence-1",
                "occurrence-1/candidate-2",
                ChoiceIntent::UseThisTimeOnly,
            )
            .unwrap();
        session.text = "錯科技詞".into();
        let before = session.snapshot();
        assert_eq!(session.undo(1).unwrap_err(), ChoiceError::InvalidTracking);
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.undo.len(), 1);
        let mut exhausted = super::tests::session();
        exhausted.revision = u64::MAX;
        let before = exhausted.snapshot();
        assert_eq!(
            exhausted
                .apply_choice(
                    u64::MAX,
                    "occurrence-1",
                    "occurrence-1/candidate-1",
                    ChoiceIntent::UseThisTimeOnly
                )
                .unwrap_err(),
            ChoiceError::RevisionExhausted
        );
        assert_eq!(exhausted.snapshot(), before);
    }
}
