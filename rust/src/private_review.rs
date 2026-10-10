//! Request-bound owned review; remembered writes are separate atomic transactions.
use crate::alternative_terms::{ChoiceError, ChoiceIntent, ReviewSession, ReviewSnapshot};
use crate::private_store::{Preference, PrivateStore, StoreError};
use crate::RuntimeResponse;

#[derive(Debug)]
pub enum RememberError {
    Choice(ChoiceError),
    Persistence(StoreError),
}
impl From<ChoiceError> for RememberError {
    fn from(error: ChoiceError) -> Self {
        Self::Choice(error)
    }
}
impl std::fmt::Display for RememberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Choice(error) => write!(f, "{error}"),
            Self::Persistence(_) => f.write_str("Text choice applied, but nothing was remembered"),
        }
    }
}
impl std::error::Error for RememberError {}

pub struct PrivateReviewSession {
    review: ReviewSession,
    input: String,
    source: String,
    target: String,
    usage: Option<String>,
}
impl PrivateReviewSession {
    pub(crate) fn from_response(
        response: &RuntimeResponse,
        usage: Option<String>,
    ) -> Result<Self, ChoiceError> {
        Ok(Self {
            review: ReviewSession::from_response(response)?,
            input: response.input.clone(),
            source: response.source_locale.clone(),
            target: response.target_locale.clone(),
            usage,
        })
    }
    pub fn snapshot(&self) -> ReviewSnapshot {
        self.review.snapshot()
    }
    pub fn undo(&mut self, revision: u64) -> Result<ReviewSnapshot, ChoiceError> {
        self.review.undo(revision)
    }
    pub fn apply_choice(
        &mut self,
        revision: u64,
        occurrence: &str,
        candidate: &str,
        intent: ChoiceIntent,
        store: Option<&mut PrivateStore>,
    ) -> Result<ReviewSnapshot, RememberError> {
        if intent == ChoiceIntent::RememberForThisContext && self.usage.is_none() {
            return Err(ChoiceError::MissingUsageContext.into());
        }
        if intent != ChoiceIntent::UseThisTimeOnly
            && self
                .review
                .snapshot()
                .occurrences
                .iter()
                .any(|item| item.occurrence_id == occurrence && !item.rememberable)
        {
            return Err(ChoiceError::UnsafeRememberSource.into());
        }
        let snapshot = self.review.apply_choice(
            revision,
            occurrence,
            candidate,
            ChoiceIntent::UseThisTimeOnly,
        )?;
        if intent == ChoiceIntent::UseThisTimeOnly {
            return Ok(snapshot);
        }
        let item = snapshot
            .occurrences
            .iter()
            .find(|item| item.occurrence_id == occurrence)
            .expect("validated owned occurrence");
        let source: String = self
            .input
            .chars()
            .skip(item.source_span[0])
            .take(item.source_span[1] - item.source_span[0])
            .collect();
        let term = Preference {
            source,
            replacement: item.expected_text.clone(),
            target: self.target.clone(),
            source_locale: Some(self.source.clone()),
            usage: if intent == ChoiceIntent::RememberForThisContext {
                self.usage.clone()
            } else {
                None
            },
            note: None,
        };
        let store = store.ok_or_else(|| {
            RememberError::Persistence(StoreError::Invalid("No private store".into()))
        })?;
        store
            .put_preference("personal", &term)
            .map_err(RememberError::Persistence)?;
        Ok(snapshot)
    }
}
