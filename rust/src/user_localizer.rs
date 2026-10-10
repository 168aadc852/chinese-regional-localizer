use crate::context_profiles::ContextProfiles;
use crate::{LocalizerEngine, LocalizerError, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::{json, Value};
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
    dictionary_id: String,
    usage_context_id: Option<String>,
    legacy: bool,
    context_level: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserLocalizationResult {
    pub input: String,
    pub output: String,
    pub source_locale: String,
    pub target_locale: String,
    pub route: Vec<String>,
    pub changes: Vec<Value>,
    pub review_needed: bool,
    pub user_dictionary_applied: bool,
}

pub struct UserControlledLocalizer {
    shared: LocalizerEngine,
    user_conn: Connection,
}

impl UserControlledLocalizer {
    pub fn open(shared_db: impl AsRef<Path>, user_db: impl AsRef<Path>) -> Result<Self> {
        let shared = LocalizerEngine::open(shared_db)?;
        let mut user_conn = Connection::open(user_db)?;
        crate::private_store::prepare_store(&mut user_conn)?;
        Ok(Self { shared, user_conn })
    }

    pub fn from_connections(shared_conn: Connection, user_conn: Connection) -> Self {
        Self {
            shared: LocalizerEngine::from_connection(shared_conn),
            user_conn,
        }
    }

    pub fn with_context_profiles(mut self, profiles: ContextProfiles) -> Self {
        self.shared = self.shared.with_context_profiles(profiles);
        self
    }

    pub fn localize(
        &self,
        text: &str,
        source_locale: &str,
        target_locale: &str,
        context: Option<&Value>,
    ) -> Result<UserLocalizationResult> {
        let route = route_strings(source_locale, target_locale)?;
        // Validate even empty/fully user-protected input; never hide an invalid request.
        let chain = self.shared.validate_usage_context(context)?;
        let terms = self.candidates(source_locale, target_locale, chain.as_deref())?;
        let mut by_surface: HashMap<String, Vec<UserTerm>> = HashMap::new();
        for term in terms {
            by_surface
                .entry(term.source_text.clone())
                .or_default()
                .push(term);
        }
        let mut surfaces: Vec<String> = by_surface.keys().cloned().collect();
        surfaces.sort_by(|a, b| {
            b.chars()
                .count()
                .cmp(&a.chars().count())
                .then_with(|| a.cmp(b))
        });

        let segments = segments(text, &surfaces);
        let mut output = String::new();
        let mut changes = Vec::new();
        let mut user_dictionary_applied = false;

        for segment in segments {
            match segment.kind {
                SegmentKind::Shared => {
                    let source_text = &text[segment.start..segment.end];
                    let result =
                        self.shared
                            .localize(source_text, source_locale, target_locale, context)?;
                    let output_start_chars = output.chars().count();
                    let input_start_chars = text[..segment.start].chars().count();
                    for change in &result.changes {
                        let local_input = change
                            .choice
                            .as_ref()
                            .map(|choice| choice.source_span)
                            .or_else(|| find_char_span(source_text, &change.original));
                        let local_output = change
                            .choice
                            .as_ref()
                            .map(|choice| choice.output_span)
                            .or_else(|| find_char_span(&result.output, &change.replacement));
                        let mut event = json!({
                            "type": change.kind,
                            "applied": change.applied,
                            "review_needed": change.review_needed,
                            "reason": change.reason,
                            "original": change.original,
                            "replacement": change.replacement,
                            "source_locale": source_locale,
                            "target_locale": target_locale,
                            "original_input_span": local_input.map(|[a,b]| [a + input_start_chars, b + input_start_chars]),
                            "final_output_span": local_output.map(|[a,b]| [a + output_start_chars, b + output_start_chars]),
                            "user_layer_segment_input_span": [input_start_chars, text[..segment.end].chars().count()],
                            "provenance": "shared_database",
                        });
                        if let Some(selection) = &change.context_selection {
                            event["context_selection"] = json!(selection);
                            if let Some(id) = change.matched_rule_id {
                                event["rule_id"] = json!(id);
                            }
                            if let Some(stage) = &change.matched_stage {
                                event["stage"] = json!(stage);
                            }
                        }
                        if let Some(mut choice) = change.choice.clone() {
                            if let Some(id) = change.matched_rule_id {
                                event["rule_id"] = json!(id);
                            }
                            if let Some(stage) = &change.matched_stage {
                                event["stage"] = json!(stage);
                            }
                            choice.offset(input_start_chars, output_start_chars);
                            event["original_input_span"] = json!(choice.source_span);
                            event["final_output_span"] = json!(choice.output_span);
                            event["choice"] = json!(choice);
                        }
                        changes.push(event);
                    }
                    output.push_str(&result.output);
                }
                SegmentKind::User => {
                    let source_text = &text[segment.start..segment.end];
                    let terms = by_surface
                        .get(source_text)
                        .expect("user segment surface must exist");
                    let input_span = [
                        text[..segment.start].chars().count(),
                        text[..segment.end].chars().count(),
                    ];
                    let output_start = output.chars().count();
                    let (replacement, mut event) = resolve_user_segment(
                        source_text,
                        terms,
                        source_locale,
                        target_locale,
                        input_span,
                        output_start,
                    );
                    let preferred: Vec<_> = terms
                        .iter()
                        .filter(|term| term.kind == "override")
                        .collect();
                    if !preferred.is_empty()
                        && !terms.iter().any(|term| term.kind == "protected")
                        && preferred.iter().any(|term| !term.legacy)
                    {
                        let level = preferred
                            .iter()
                            .map(|term| term.context_level)
                            .min()
                            .unwrap();
                        let output_span =
                            [output_start, output_start + replacement.chars().count()];
                        let keys: Vec<_> = preferred
                            .iter()
                            .map(|term| {
                                let legacy_level = preferred
                                    .iter()
                                    .filter(|other| other.context_level == term.context_level)
                                    .all(|other| other.legacy);
                                // Reverse avoids negating a potentially i64::MIN legacy priority.
                                (
                                    term.context_level,
                                    std::cmp::Reverse(if legacy_level {
                                        specificity(term)
                                    } else {
                                        0
                                    }),
                                    std::cmp::Reverse(if legacy_level { term.priority } else { 0 }),
                                )
                            })
                            .collect();
                        let levels: std::collections::BTreeSet<_> = keys.iter().copied().collect();
                        event["choice"] = json!(crate::alternative_terms::build_choice(
                            preferred.iter().zip(keys.iter()).map(|(term, key)| (
                                term.replacement.clone().unwrap(),
                                levels.iter().position(|level| level == key).unwrap(),
                                0
                            )),
                            source_text,
                            input_span,
                            input_span,
                            output_span,
                            &replacement,
                        ));
                        let ids: std::collections::BTreeSet<_> = preferred
                            .iter()
                            .filter(|term| term.context_level == level)
                            .map(|term| &term.dictionary_id)
                            .collect();
                        let all_contexts = preferred.iter().any(|term| {
                            term.context_level == level && term.usage_context_id.is_none()
                        });
                        event["private_selection"] = json!({"level": if all_contexts {"all_contexts"} else if level == 0 {"exact"} else {"ancestor"}, "dictionary_ids": ids});
                    }
                    user_dictionary_applied = true;
                    output.push_str(&replacement);
                    changes.push(event);
                }
            }
        }

        let review_needed = changes
            .iter()
            .any(|event| event.get("review_needed").and_then(Value::as_bool) == Some(true));
        crate::alternative_terms::assign_event_ids(&mut changes);
        Ok(UserLocalizationResult {
            input: text.to_owned(),
            output,
            source_locale: source_locale.to_owned(),
            target_locale: target_locale.to_owned(),
            route,
            changes,
            review_needed,
            user_dictionary_applied,
        })
    }

    fn candidates(
        &self,
        source_locale: &str,
        target_locale: &str,
        chain: Option<&[String]>,
    ) -> Result<Vec<UserTerm>> {
        let v2 = crate::private_store::schema_version(&self.user_conn)? == "2";
        let mut stmt = self.user_conn.prepare(if v2 {
            "SELECT user_term_id,kind,source_text,replacement,source_locale,target_locale,priority,t.note,t.dictionary_id,usage_context_id,legacy
             FROM user_terms t JOIN user_dictionaries d USING(dictionary_id) WHERE t.enabled=1 AND d.enabled=1
             AND (source_locale IS NULL OR source_locale=?1) AND (target_locale IS NULL OR target_locale=?2)"
        } else {
            r#"
            SELECT user_term_id, kind, source_text, replacement,
                   source_locale, target_locale, priority, note, 'legacy', NULL, 1
            FROM user_terms
            WHERE enabled = 1
              AND (source_locale IS NULL OR source_locale = ?1)
              AND (target_locale IS NULL OR target_locale = ?2)
            ORDER BY length(source_text) DESC, source_text, priority DESC, user_term_id
            "#
        })?;
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
                dictionary_id: row.get(8)?,
                usage_context_id: row.get(9)?,
                legacy: row.get(10)?,
                context_level: 0,
            })
        })?;
        let terms = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(terms
            .into_iter()
            .filter_map(|mut term| {
                term.context_level = match &term.usage_context_id {
                    None => chain.map_or(0, <[String]>::len),
                    Some(id) => chain?.iter().position(|value| value == id)?,
                };
                Some(term)
            })
            .collect())
    }
}

#[derive(Clone, Copy)]
enum SegmentKind {
    Shared,
    User,
}

#[derive(Clone, Copy)]
struct Segment {
    kind: SegmentKind,
    start: usize,
    end: usize,
}

fn segments(text: &str, surfaces: &[String]) -> Vec<Segment> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut shared_start = 0usize;
    let mut i = 0usize;
    while i < text.len() {
        let matched = surfaces
            .iter()
            .find(|surface| text[i..].starts_with(surface.as_str()));
        if let Some(surface) = matched {
            if shared_start < i {
                result.push(Segment {
                    kind: SegmentKind::Shared,
                    start: shared_start,
                    end: i,
                });
            }
            let end = i + surface.len();
            result.push(Segment {
                kind: SegmentKind::User,
                start: i,
                end,
            });
            i = end;
            shared_start = end;
        } else {
            let ch = text[i..].chars().next().expect("valid char boundary");
            i += ch.len_utf8();
        }
    }
    if shared_start < text.len() {
        result.push(Segment {
            kind: SegmentKind::Shared,
            start: shared_start,
            end: text.len(),
        });
    }
    if result.is_empty() {
        result.push(Segment {
            kind: SegmentKind::Shared,
            start: 0,
            end: text.len(),
        });
    }
    result
}

fn specificity(term: &UserTerm) -> i64 {
    i64::from(term.source_locale.is_some()) + i64::from(term.target_locale.is_some())
}

fn rank(mut terms: Vec<&UserTerm>) -> Vec<&UserTerm> {
    terms.sort_by(|a, b| {
        specificity(b)
            .cmp(&specificity(a))
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| a.dictionary_id.cmp(&b.dictionary_id))
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
) -> (String, Value) {
    let protected = rank(
        terms
            .iter()
            .filter(|term| term.kind == "protected")
            .collect(),
    );
    if let Some(winner) = protected.first() {
        let output_span = [output_start, output_start + source_text.chars().count()];
        return (
            source_text.to_owned(),
            json!({
                "type": "user_protected",
                "applied": false,
                "review_needed": false,
                "reason": "protected_by_user",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": input_span,
                "final_output_span": output_span,
                "user_term_id": winner.user_term_id,
                "note": winner.note,
                "provenance": "user_dictionary",
            }),
        );
    }

    let ranked = rank(
        terms
            .iter()
            .filter(|term| term.kind == "override")
            .collect(),
    );
    if ranked.is_empty() {
        let output_span = [output_start, output_start + source_text.chars().count()];
        return (
            source_text.to_owned(),
            json!({
                "type": "user_override",
                "applied": false,
                "review_needed": true,
                "reason": "invalid_user_rule",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": input_span,
                "final_output_span": output_span,
                "provenance": "user_dictionary",
            }),
        );
    }

    let level = ranked.iter().map(|term| term.context_level).min().unwrap();
    let ranked: Vec<_> = ranked
        .into_iter()
        .filter(|term| term.context_level == level)
        .collect();
    let legacy_only = ranked.iter().all(|term| term.legacy);
    let top_rank = (specificity(ranked[0]), ranked[0].priority);
    let top: Vec<&UserTerm> = ranked
        .into_iter()
        .filter(|term| !legacy_only || (specificity(term), term.priority) == top_rank)
        .collect();
    let replacements: HashSet<Option<&str>> =
        top.iter().map(|term| term.replacement.as_deref()).collect();

    if replacements.len() != 1 || replacements.contains(&None) {
        let output_span = [output_start, output_start + source_text.chars().count()];
        let candidates: Vec<Value> = top
            .iter()
            .map(|term| {
                json!({
                    "user_term_id": term.user_term_id,
                    "replacement": term.replacement,
                    "source_locale": term.source_locale,
                    "target_locale": term.target_locale,
                })
            })
            .collect();
        return (
            source_text.to_owned(),
            json!({
                "type": "user_override",
                "applied": false,
                "review_needed": true,
                "reason": if replacements.contains(&None) { "invalid_user_rule" } else { "ambiguous_user_override" },
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": input_span,
                "final_output_span": output_span,
                "candidates": candidates,
                "provenance": "user_dictionary",
            }),
        );
    }

    let winner = top[0];
    let replacement = winner
        .replacement
        .as_deref()
        .unwrap_or(source_text)
        .to_owned();
    let output_span = [output_start, output_start + replacement.chars().count()];
    (
        replacement.clone(),
        json!({
            "type": "user_override",
            "applied": replacement != source_text,
            "review_needed": false,
            "reason": "user_fixed_override",
            "original": source_text,
            "replacement": replacement,
            "source_locale": source_locale,
            "target_locale": target_locale,
            "original_input_span": input_span,
            "final_output_span": output_span,
            "user_term_id": winner.user_term_id,
            "note": winner.note,
            "provenance": "user_dictionary",
        }),
    )
}

fn find_char_span(haystack: &str, needle: &str) -> Option<[usize; 2]> {
    let byte_start = haystack.find(needle)?;
    let byte_end = byte_start + needle.len();
    Some([
        haystack[..byte_start].chars().count(),
        haystack[..byte_end].chars().count(),
    ])
}

fn route_strings(source: &str, target: &str) -> Result<Vec<String>> {
    let route = match (source, target) {
        ("zh-CN", "zh-HK") => vec!["zh-CN->zh-Hant", "zh-Hant->zh-HK"],
        ("zh-CN", "zh-TW") => vec!["zh-CN->zh-Hant", "zh-Hant->zh-TW"],
        ("zh-Hant", "zh-HK") => vec!["zh-Hant->zh-HK"],
        ("zh-Hant", "zh-TW") => vec!["zh-Hant->zh-TW"],
        _ => {
            return Err(LocalizerError::UnsupportedRoute(
                source.to_owned(),
                target.to_owned(),
            ))
        }
    };
    Ok(route.into_iter().map(str::to_owned).collect())
}
