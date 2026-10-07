use crate::{LocalizerEngine, LocalizerError, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};
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
    fn specificity(&self) -> i64 {
        i64::from(self.source_locale.is_some()) + i64::from(self.target_locale.is_some())
    }
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
        Ok(Self {
            shared: LocalizerEngine::open(shared_db)?,
            user_conn: Connection::open(user_db)?,
        })
    }

    pub fn from_connections(shared_conn: Connection, user_conn: Connection) -> Self {
        Self {
            shared: LocalizerEngine::from_connection(shared_conn),
            user_conn,
        }
    }

    pub fn localize(
        &self,
        text: &str,
        source_locale: &str,
        target_locale: &str,
        context: Option<&Value>,
    ) -> Result<UserLocalizationResult> {
        let route = route_strings(source_locale, target_locale)?;
        let candidates = self.candidates(source_locale, target_locale)?;
        let mut by_surface: HashMap<String, Vec<UserTerm>> = HashMap::new();
        for term in candidates {
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

        let mut output = String::new();
        let mut changes = Vec::new();
        let mut input_byte = 0usize;
        let mut shared_start = 0usize;

        while input_byte < text.len() {
            let matched = surfaces.iter().find(|surface| text[input_byte..].starts_with(surface.as_str()));
            let Some(surface) = matched else {
                input_byte += text[input_byte..]
                    .chars()
                    .next()
                    .expect("valid UTF-8 boundary")
                    .len_utf8();
                continue;
            };

            if shared_start < input_byte {
                self.append_shared_segment(
                    &text[shared_start..input_byte],
                    source_locale,
                    target_locale,
                    context,
                    shared_start,
                    &mut output,
                    &mut changes,
                )?;
            }

            let end = input_byte + surface.len();
            let terms = by_surface.get(surface).expect("surface indexed");
            let event = self.resolve_user_segment(
                surface,
                terms,
                source_locale,
                target_locale,
                text,
                input_byte,
                end,
                output.chars().count(),
            );
            let replacement = event
                .get("replacement")
                .and_then(Value::as_str)
                .unwrap_or(surface);
            output.push_str(replacement);
            changes.push(event);
            input_byte = end;
            shared_start = end;
        }

        if shared_start < text.len() {
            self.append_shared_segment(
                &text[shared_start..],
                source_locale,
                target_locale,
                context,
                shared_start,
                &mut output,
                &mut changes,
            )?;
        } else if text.is_empty() {
            return Ok(UserLocalizationResult {
                input: String::new(),
                output: String::new(),
                source_locale: source_locale.to_owned(),
                target_locale: target_locale.to_owned(),
                route,
                changes: Vec::new(),
                review_needed: false,
                user_dictionary_applied: false,
            });
        }

        let review_needed = changes
            .iter()
            .any(|event| event.get("review_needed").and_then(Value::as_bool).unwrap_or(false));
        let user_dictionary_applied = changes.iter().any(|event| {
            event
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind.starts_with("user_"))
        });

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

    fn candidates(&self, source_locale: &str, target_locale: &str) -> Result<Vec<UserTerm>> {
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

    #[allow(clippy::too_many_arguments)]
    fn append_shared_segment(
        &self,
        segment: &str,
        source_locale: &str,
        target_locale: &str,
        context: Option<&Value>,
        segment_input_byte: usize,
        output: &mut String,
        changes: &mut Vec<Value>,
    ) -> Result<()> {
        let result = self
            .shared
            .localize(segment, source_locale, target_locale, context)?;
        let output_char_base = output.chars().count();
        let input_char_base = byte_to_char_index_at_source(segment_input_byte, segment, segment_input_byte);

        let mut original_cursor = 0usize;
        let mut final_cursor = 0usize;
        for change in result.changes {
            let original_pos = find_from_char(segment, &change.original, original_cursor).unwrap_or(original_cursor);
            let replacement_pos = find_from_char(&result.output, &change.replacement, final_cursor)
                .unwrap_or(final_cursor);
            original_cursor = original_pos + change.original.chars().count();
            final_cursor = replacement_pos + change.replacement.chars().count();

            let mut event = serde_json::to_value(change).expect("serialize shared change");
            if let Some(object) = event.as_object_mut() {
                if let Some(kind) = object.remove("kind") {
                    object.insert("type".into(), kind);
                }
                object.insert(
                    "original_input_span".into(),
                    json!([
                        input_char_base + original_pos,
                        input_char_base + original_cursor
                    ]),
                );
                object.insert(
                    "final_output_span".into(),
                    json!([
                        output_char_base + replacement_pos,
                        output_char_base + final_cursor
                    ]),
                );
                object.insert(
                    "user_layer_segment_input_span".into(),
                    json!([
                        input_char_base,
                        input_char_base + segment.chars().count()
                    ]),
                );
            }
            changes.push(event);
        }
        output.push_str(&result.output);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_user_segment(
        &self,
        source_text: &str,
        terms: &[UserTerm],
        source_locale: &str,
        target_locale: &str,
        full_text: &str,
        input_start_byte: usize,
        input_end_byte: usize,
        output_start_char: usize,
    ) -> Value {
        let input_start_char = full_text[..input_start_byte].chars().count();
        let input_end_char = full_text[..input_end_byte].chars().count();

        let mut protected: Vec<&UserTerm> = terms.iter().filter(|term| term.kind == "protected").collect();
        if !protected.is_empty() {
            rank_terms(&mut protected);
            let winner = protected[0];
            return json!({
                "type": "user_protected",
                "applied": false,
                "review_needed": false,
                "reason": "protected_by_user",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": [input_start_char, input_end_char],
                "final_output_span": [output_start_char, output_start_char + source_text.chars().count()],
                "user_term_id": winner.user_term_id,
                "note": winner.note,
                "provenance": "user_dictionary"
            });
        }

        let mut overrides: Vec<&UserTerm> = terms.iter().filter(|term| term.kind == "override").collect();
        if overrides.is_empty() {
            return json!({
                "type": "user_override",
                "applied": false,
                "review_needed": true,
                "reason": "invalid_user_rule",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": [input_start_char, input_end_char],
                "final_output_span": [output_start_char, output_start_char + source_text.chars().count()],
                "provenance": "user_dictionary"
            });
        }
        rank_terms(&mut overrides);
        let top_specificity = overrides[0].specificity();
        let top_priority = overrides[0].priority;
        let top: Vec<&UserTerm> = overrides
            .into_iter()
            .filter(|term| term.specificity() == top_specificity && term.priority == top_priority)
            .collect();
        let replacements: BTreeSet<&str> = top
            .iter()
            .filter_map(|term| term.replacement.as_deref())
            .collect();

        if replacements.len() != 1 {
            let candidates: Vec<Value> = top
                .iter()
                .map(|term| {
                    json!({
                        "user_term_id": term.user_term_id,
                        "replacement": term.replacement,
                        "source_locale": term.source_locale,
                        "target_locale": term.target_locale,
                        "priority": term.priority
                    })
                })
                .collect();
            return json!({
                "type": "user_override",
                "applied": false,
                "review_needed": true,
                "reason": "ambiguous_user_override",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": [input_start_char, input_end_char],
                "final_output_span": [output_start_char, output_start_char + source_text.chars().count()],
                "candidates": candidates,
                "provenance": "user_dictionary"
            });
        }

        let winner = top[0];
        let replacement = winner.replacement.as_deref().unwrap_or(source_text);
        json!({
            "type": "user_override",
            "applied": replacement != source_text,
            "review_needed": false,
            "reason": "user_fixed_override",
            "original": source_text,
            "replacement": replacement,
            "source_locale": source_locale,
            "target_locale": target_locale,
            "original_input_span": [input_start_char, input_end_char],
            "final_output_span": [output_start_char, output_start_char + replacement.chars().count()],
            "user_term_id": winner.user_term_id,
            "note": winner.note,
            "provenance": "user_dictionary"
        })
    }
}

fn rank_terms(terms: &mut Vec<&UserTerm>) {
    terms.sort_by(|a, b| {
        b.specificity()
            .cmp(&a.specificity())
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| a.user_term_id.cmp(&b.user_term_id))
    });
}

fn route_strings(source: &str, target: &str) -> Result<Vec<String>> {
    let pairs = match (source, target) {
        ("zh-CN", "zh-HK") => vec![("zh-CN", "zh-Hant"), ("zh-Hant", "zh-HK")],
        ("zh-CN", "zh-TW") => vec![("zh-CN", "zh-Hant"), ("zh-Hant", "zh-TW")],
        ("zh-Hant", "zh-HK") => vec![("zh-Hant", "zh-HK")],
        ("zh-Hant", "zh-TW") => vec![("zh-Hant", "zh-TW")],
        _ => return Err(LocalizerError::UnsupportedRoute(source.into(), target.into())),
    };
    Ok(pairs
        .into_iter()
        .map(|(a, b)| format!("{a}->{b}"))
        .collect())
}

fn find_from_char(haystack: &str, needle: &str, char_start: usize) -> Option<usize> {
    let byte_start = haystack
        .char_indices()
        .nth(char_start)
        .map(|(index, _)| index)
        .unwrap_or(haystack.len());
    haystack[byte_start..].find(needle).map(|relative| {
        haystack[..byte_start + relative].chars().count()
    })
}

fn byte_to_char_index_at_source(byte_index: usize, _segment: &str, _segment_start: usize) -> usize {
    // The caller passes a byte offset into the original string. User-local segmentation always
    // begins on UTF-8 boundaries; counting preceding bytes as chars is only correct for ASCII,
    // so callers should prefer explicit full-text char counting where available. This helper is
    // retained as a narrow compatibility point and is overridden by the segment's byte offset.
    byte_index
}
