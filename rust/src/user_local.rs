use chinese_regional_localizer::{LocalizerEngine, LocalizerError};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone)]
struct UserTerm {
    user_term_id: i64,
    kind: String,
    source_text: String,
    replacement: Option<String>,
    source_locale: Option<String>,
    target_locale: Option<String>,
    priority: i64,
    note: Option<String>,
}

impl UserTerm {
    fn specificity(&self) -> usize {
        usize::from(self.source_locale.is_some()) + usize::from(self.target_locale.is_some())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserCandidate {
    pub user_term_id: i64,
    pub replacement: Option<String>,
    pub source_locale: Option<String>,
    pub target_locale: Option<String>,
    pub priority: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserChange {
    #[serde(rename = "type")]
    pub kind: String,
    pub reason: String,
    pub original: String,
    pub replacement: String,
    pub applied: bool,
    pub review_needed: bool,
    pub source_locale: String,
    pub target_locale: String,
    pub original_input_span: [usize; 2],
    pub final_output_span: [usize; 2],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_term_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub provenance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<UserCandidate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_layer_segment_input_span: Option<[usize; 2]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserLocalizationResult {
    pub input: String,
    pub output: String,
    pub source_locale: String,
    pub target_locale: String,
    pub route: Vec<String>,
    pub changes: Vec<UserChange>,
    pub review_needed: bool,
    pub user_dictionary_applied: bool,
}

pub struct UserControlledLocalizer {
    shared: LocalizerEngine,
    user_conn: Connection,
}

impl UserControlledLocalizer {
    pub fn open(
        shared_db: impl AsRef<Path>,
        user_db: impl AsRef<Path>,
    ) -> Result<Self, LocalizerError> {
        Ok(Self {
            shared: LocalizerEngine::open(shared_db)?,
            user_conn: Connection::open(user_db)?,
        })
    }

    pub fn localize(
        &self,
        text: &str,
        source_locale: &str,
        target_locale: &str,
        context: Option<&Value>,
    ) -> Result<UserLocalizationResult, LocalizerError> {
        // Validate route up front through the shared engine contract. Empty text has no
        // user segments, so use a harmless empty localization to keep unsupported-route
        // behavior identical to the shared engine.
        if text.is_empty() {
            let shared = self
                .shared
                .localize(text, source_locale, target_locale, context)?;
            return Ok(UserLocalizationResult {
                input: text.to_owned(),
                output: shared.output,
                source_locale: source_locale.to_owned(),
                target_locale: target_locale.to_owned(),
                route: shared.route,
                changes: Vec::new(),
                review_needed: false,
                user_dictionary_applied: false,
            });
        }

        let terms = self.candidates(source_locale, target_locale)?;
        let by_surface = group_by_surface(terms);
        let mut surfaces: Vec<&String> = by_surface.keys().collect();
        surfaces.sort_by(|a, b| {
            b.chars()
                .count()
                .cmp(&a.chars().count())
                .then_with(|| a.cmp(b))
        });

        let segments = segments(text, &surfaces);
        let mut output = String::new();
        let mut changes = Vec::new();
        let mut output_chars = 0usize;
        let mut route = None;

        for segment in segments {
            let source_text = &text[segment.start..segment.end];
            let input_char_start = byte_to_char(text, segment.start);
            let input_char_end = byte_to_char(text, segment.end);

            if !segment.user {
                let shared = self
                    .shared
                    .localize(source_text, source_locale, target_locale, context)?;
                if route.is_none() {
                    route = Some(shared.route.clone());
                }
                let replacement = shared.output;
                let segment_output_start = output_chars;
                let mut original_search = 0usize;
                let mut final_search = 0usize;

                for change in shared.changes {
                    let local_original = locate_chars(source_text, &change.original, original_search)
                        .unwrap_or([0, source_text.chars().count()]);
                    original_search = local_original[1];
                    let local_final = locate_chars(&replacement, &change.replacement, final_search)
                        .unwrap_or([0, replacement.chars().count()]);
                    final_search = local_final[1];
                    changes.push(UserChange {
                        kind: change.kind,
                        reason: change.reason,
                        original: change.original,
                        replacement: change.replacement,
                        applied: change.applied,
                        review_needed: change.review_needed,
                        source_locale: source_locale.to_owned(),
                        target_locale: target_locale.to_owned(),
                        original_input_span: [
                            input_char_start + local_original[0],
                            input_char_start + local_original[1],
                        ],
                        final_output_span: [
                            segment_output_start + local_final[0],
                            segment_output_start + local_final[1],
                        ],
                        user_term_id: None,
                        note: None,
                        provenance: "shared_database".into(),
                        candidates: None,
                        user_layer_segment_input_span: Some([
                            input_char_start,
                            input_char_end,
                        ]),
                    });
                }
                output_chars += replacement.chars().count();
                output.push_str(&replacement);
                continue;
            }

            let segment_terms = &by_surface[source_text];
            let (replacement, event) = resolve_user_segment(
                source_text,
                segment_terms,
                source_locale,
                target_locale,
                [input_char_start, input_char_end],
                output_chars,
            );
            output_chars += replacement.chars().count();
            output.push_str(&replacement);
            changes.push(event);
        }

        if route.is_none() {
            route = Some(
                self.shared
                    .localize("", source_locale, target_locale, context)?
                    .route,
            );
        }
        let review_needed = changes.iter().any(|event| event.review_needed);
        let user_dictionary_applied = changes
            .iter()
            .any(|event| event.kind.starts_with("user_"));
        Ok(UserLocalizationResult {
            input: text.to_owned(),
            output,
            source_locale: source_locale.to_owned(),
            target_locale: target_locale.to_owned(),
            route: route.expect("route assigned"),
            changes,
            review_needed,
            user_dictionary_applied,
        })
    }

    fn candidates(
        &self,
        source_locale: &str,
        target_locale: &str,
    ) -> Result<Vec<UserTerm>, LocalizerError> {
        let mut stmt = self.user_conn.prepare(
            r#"
            SELECT user_term_id, kind, source_text, replacement,
                   source_locale, target_locale, priority, note
            FROM user_terms
            WHERE enabled = 1
              AND (source_locale IS NULL OR source_locale = ?1)
              AND (target_locale IS NULL OR target_locale = ?2)
            ORDER BY length(source_text) DESC, source_text, priority DESC, user_term_id
            "#,
        )?;
        let rows = stmt.query_map(params![source_locale, target_locale], |row| {
            Ok(UserTerm {
                user_term_id: row.get(0)?,
                kind: row.get(1)?,
                source_text: row.get(2)?,
                replacement: row.get(3)?,
                source_locale: row.get(4)?,
                target_locale: row.get(5)?,
                priority: row.get(6)?,
                note: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    start: usize,
    end: usize,
    user: bool,
}

fn segments(text: &str, surfaces: &[&String]) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut shared_start = 0usize;
    let mut i = 0usize;
    while i < text.len() {
        let matched = surfaces.iter().find(|surface| text[i..].starts_with(surface.as_str()));
        let Some(surface) = matched else {
            let ch = text[i..].chars().next().expect("valid character boundary");
            i += ch.len_utf8();
            continue;
        };
        if shared_start < i {
            result.push(Segment {
                start: shared_start,
                end: i,
                user: false,
            });
        }
        let end = i + surface.len();
        result.push(Segment {
            start: i,
            end,
            user: true,
        });
        i = end;
        shared_start = end;
    }
    if shared_start < text.len() {
        result.push(Segment {
            start: shared_start,
            end: text.len(),
            user: false,
        });
    }
    if result.is_empty() {
        result.push(Segment {
            start: 0,
            end: text.len(),
            user: false,
        });
    }
    result
}

fn group_by_surface(terms: Vec<UserTerm>) -> HashMap<String, Vec<UserTerm>> {
    let mut grouped: HashMap<String, Vec<UserTerm>> = HashMap::new();
    for term in terms {
        grouped
            .entry(term.source_text.clone())
            .or_default()
            .push(term);
    }
    grouped
}

fn rank(mut terms: Vec<&UserTerm>) -> Vec<&UserTerm> {
    terms.sort_by(|a, b| {
        b.specificity()
            .cmp(&a.specificity())
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| a.user_term_id.cmp(&b.user_term_id))
    });
    terms
}

fn resolve_user_segment(
    source_text: &str,
    terms: &[UserTerm],
    source_locale: &str,
    target_locale: &str,
    input_span: [usize; 2],
    output_start: usize,
) -> (String, UserChange) {
    let protected = rank(terms.iter().filter(|term| term.kind == "protected").collect());
    if let Some(winner) = protected.first() {
        let replacement = source_text.to_owned();
        return (
            replacement.clone(),
            UserChange {
                kind: "user_protected".into(),
                reason: "protected_by_user".into(),
                original: source_text.into(),
                replacement,
                applied: false,
                review_needed: false,
                source_locale: source_locale.into(),
                target_locale: target_locale.into(),
                original_input_span: input_span,
                final_output_span: [output_start, output_start + source_text.chars().count()],
                user_term_id: Some(winner.user_term_id),
                note: winner.note.clone(),
                provenance: "user_dictionary".into(),
                candidates: None,
                user_layer_segment_input_span: None,
            },
        );
    }

    let ranked = rank(terms.iter().filter(|term| term.kind == "override").collect());
    let Some(first) = ranked.first() else {
        let replacement = source_text.to_owned();
        return (
            replacement.clone(),
            UserChange {
                kind: "user_override".into(),
                reason: "invalid_user_rule".into(),
                original: source_text.into(),
                replacement,
                applied: false,
                review_needed: true,
                source_locale: source_locale.into(),
                target_locale: target_locale.into(),
                original_input_span: input_span,
                final_output_span: [output_start, output_start + source_text.chars().count()],
                user_term_id: None,
                note: None,
                provenance: "user_dictionary".into(),
                candidates: None,
                user_layer_segment_input_span: None,
            },
        );
    };

    let top_specificity = first.specificity();
    let top_priority = first.priority;
    let top: Vec<_> = ranked
        .into_iter()
        .filter(|term| term.specificity() == top_specificity && term.priority == top_priority)
        .collect();
    let replacements: HashSet<Option<&str>> = top
        .iter()
        .map(|term| term.replacement.as_deref())
        .collect();
    if replacements.len() != 1 {
        let replacement = source_text.to_owned();
        let candidates = top
            .iter()
            .map(|term| UserCandidate {
                user_term_id: term.user_term_id,
                replacement: term.replacement.clone(),
                source_locale: term.source_locale.clone(),
                target_locale: term.target_locale.clone(),
                priority: term.priority,
            })
            .collect();
        return (
            replacement.clone(),
            UserChange {
                kind: "user_override".into(),
                reason: "ambiguous_user_override".into(),
                original: source_text.into(),
                replacement,
                applied: false,
                review_needed: true,
                source_locale: source_locale.into(),
                target_locale: target_locale.into(),
                original_input_span: input_span,
                final_output_span: [output_start, output_start + source_text.chars().count()],
                user_term_id: None,
                note: None,
                provenance: "user_dictionary".into(),
                candidates: Some(candidates),
                user_layer_segment_input_span: None,
            },
        );
    }

    let winner = top[0];
    let replacement = winner
        .replacement
        .clone()
        .unwrap_or_else(|| source_text.to_owned());
    let output_end = output_start + replacement.chars().count();
    (
        replacement.clone(),
        UserChange {
            kind: "user_override".into(),
            reason: "user_fixed_override".into(),
            original: source_text.into(),
            replacement: replacement.clone(),
            applied: replacement != source_text,
            review_needed: false,
            source_locale: source_locale.into(),
            target_locale: target_locale.into(),
            original_input_span: input_span,
            final_output_span: [output_start, output_end],
            user_term_id: Some(winner.user_term_id),
            note: winner.note.clone(),
            provenance: "user_dictionary".into(),
            candidates: None,
            user_layer_segment_input_span: None,
        },
    )
}

fn byte_to_char(text: &str, byte: usize) -> usize {
    text[..byte].chars().count()
}

fn locate_chars(text: &str, needle: &str, min_char: usize) -> Option<[usize; 2]> {
    if needle.is_empty() {
        return Some([min_char, min_char]);
    }
    let min_byte = text
        .char_indices()
        .nth(min_char)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    let found = text[min_byte..].find(needle)? + min_byte;
    let start = byte_to_char(text, found);
    Some([start, start + needle.chars().count()])
}
