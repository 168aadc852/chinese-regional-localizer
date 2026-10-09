//! Request validation and ranking against the existing #46 resolver, never file input.
use crate::context_profiles::ContextProfiles;
use crate::{LocalizerError, Result};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct ContextSelection {
    pub usage_context_id: String,
    pub matched_usage_context_id: Option<String>,
    pub context_distance: Option<usize>,
    pub context_level: String,
}

pub(crate) fn resolve_usage_context(
    profiles: &ContextProfiles,
    context: Option<&Value>,
) -> Result<Option<Vec<String>>> {
    let Some(value) = context.and_then(|context| context.get("usage_context_id")) else {
        return Ok(None);
    };
    let id = value.as_str().filter(|id| !id.is_empty()).ok_or_else(|| {
        LocalizerError::UsageContext("usage_context_id must be a nonempty context ID string".into())
    })?;
    let chain = profiles
        .resolve_chain(id)
        .map_err(|error| LocalizerError::UsageContext(error.to_string()))?;
    Ok(Some(
        chain
            .iter()
            .map(|profile| profile.context_id.clone())
            .collect(),
    ))
}

/// Scoped rules need an explicit selected chain and a regional stage. A custom
/// root never receives an invented General ancestor. No lists/wildcards/coercion.
pub(crate) fn rule_context_level(
    metadata: Option<&Value>,
    chain: Option<&[String]>,
) -> Option<usize> {
    let Some(metadata) = metadata else {
        return Some(chain.map_or(0, <[String]>::len));
    };
    let object = metadata.as_object()?;
    if object.contains_key("usage_context_id") {
        return None;
    }
    let Some(constraints) = object.get("constraints") else {
        return Some(chain.map_or(0, <[String]>::len));
    };
    let constraints = constraints.as_object()?;
    match constraints.get("usage_context_id") {
        None => Some(chain.map_or(0, <[String]>::len)),
        Some(value) => {
            let id = value.as_str()?;
            chain?.iter().position(|profile| profile == id)
        }
    }
}

pub(crate) fn selection(chain: &[String], level: usize) -> ContextSelection {
    let matched = chain.get(level);
    ContextSelection {
        usage_context_id: chain[0].clone(),
        matched_usage_context_id: matched.cloned(),
        context_distance: matched.map(|_| level),
        context_level: match matched {
            None => "unscoped",
            Some(_) if level == 0 => "exact",
            Some(id) if id == "general" => "general",
            Some(_) => "parent",
        }
        .into(),
    }
}
