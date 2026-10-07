use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum LocalizerError {
    Sql(rusqlite::Error),
    UnsupportedRoute(String, String),
}

impl From<rusqlite::Error> for LocalizerError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

pub type Result<T> = std::result::Result<T, LocalizerError>;

#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub kind: String,
    pub reason: String,
    pub original: String,
    pub replacement: String,
    pub applied: bool,
    pub review_needed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalizationResult {
    pub input: String,
    pub output: String,
    pub source_locale: String,
    pub target_locale: String,
    pub route: Vec<String>,
    pub changes: Vec<Change>,
    pub review_needed: bool,
}

#[derive(Debug, Clone, Copy)]
struct ProtectedSpan {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
struct EntitySurface {
    concept_id: i64,
}

#[derive(Debug, Clone)]
struct TermRule {
    target_text: String,
    priority: i64,
    context_constraint: Option<String>,
}

pub struct LocalizerEngine {
    conn: Connection,
}

impl LocalizerEngine {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn localize(
        &self,
        text: &str,
        source_locale: &str,
        target_locale: &str,
        context: Option<&Value>,
    ) -> Result<LocalizationResult> {
        let route = route(source_locale, target_locale)?;
        let entity_result = self.apply_entities(text, source_locale, target_locale)?;
        let mut current = entity_result.0;
        let mut protected = entity_result.1;
        let mut changes = entity_result.2;

        for (stage_source, stage_target) in &route {
            let stage = self.apply_term_stage(
                &current,
                &protected,
                stage_source,
                stage_target,
                context,
            )?;
            current = stage.0;
            protected = stage.1;
            changes.extend(stage.2);
        }

        let review_needed = changes.iter().any(|change| change.review_needed);
        Ok(LocalizationResult {
            input: text.to_owned(),
            output: current,
            source_locale: source_locale.to_owned(),
            target_locale: target_locale.to_owned(),
            route: route
                .iter()
                .map(|(a, b)| format!("{a}->{b}"))
                .collect(),
            changes,
            review_needed,
        })
    }

    fn apply_entities(
        &self,
        text: &str,
        source_locale: &str,
        target_locale: &str,
    ) -> Result<(String, Vec<ProtectedSpan>, Vec<Change>)> {
        let index = self.entity_index(source_locale)?;
        let mut keys: Vec<&String> = index.keys().collect();
        keys.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));

        let mut out = String::new();
        let mut protected = Vec::new();
        let mut changes = Vec::new();
        let mut i = 0usize;

        while i < text.len() {
            let source_text = keys.iter().find_map(|key| {
                let end = i + key.len();
                if end <= text.len()
                    && text[i..].starts_with(key.as_str())
                    && boundary_ok(text, i, end, key)
                {
                    Some((*key).as_str())
                } else {
                    None
                }
            });

            let Some(source_text) = source_text else {
                let ch = text[i..].chars().next().expect("valid char boundary");
                out.push(ch);
                i += ch.len_utf8();
                continue;
            };

            let concepts = &index[source_text];
            let output_start = out.len();
            if concepts.len() != 1 {
                out.push_str(source_text);
                protected.push(ProtectedSpan {
                    start: output_start,
                    end: out.len(),
                });
                changes.push(Change {
                    kind: "entity".into(),
                    reason: "ambiguous_source_entity".into(),
                    original: source_text.into(),
                    replacement: source_text.into(),
                    applied: false,
                    review_needed: true,
                });
                i += source_text.len();
                continue;
            }

            let targets = self.target_entity_names(concepts[0].concept_id, target_locale)?;
            if targets.len() != 1 {
                out.push_str(source_text);
                protected.push(ProtectedSpan {
                    start: output_start,
                    end: out.len(),
                });
                changes.push(Change {
                    kind: "entity".into(),
                    reason: if targets.is_empty() {
                        "missing_target_name".into()
                    } else {
                        "ambiguous_target_name".into()
                    },
                    original: source_text.into(),
                    replacement: source_text.into(),
                    applied: false,
                    review_needed: true,
                });
                i += source_text.len();
                continue;
            }

            let replacement = &targets[0];
            out.push_str(replacement);
            protected.push(ProtectedSpan {
                start: output_start,
                end: out.len(),
            });
            changes.push(Change {
                kind: "entity".into(),
                reason: "localized_entity_name".into(),
                original: source_text.into(),
                replacement: replacement.clone(),
                applied: replacement != source_text,
                review_needed: false,
            });
            i += source_text.len();
        }

        Ok((out, protected, changes))
    }

    fn entity_index(&self, source_locale: &str) -> Result<HashMap<String, Vec<EntitySurface>>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT ln.concept_id, ln.text, ln.name_type
            FROM localized_names ln
            JOIN concepts c ON c.concept_id = ln.concept_id
            WHERE ln.locale = ?1 AND ln.domain = 'entity'
              AND (
                NOT EXISTS (
                    SELECT 1 FROM name_evidence any_ne
                    WHERE any_ne.localized_name_id = ln.localized_name_id
                )
                OR EXISTS (
                    SELECT 1
                    FROM name_evidence current_ne
                    LEFT JOIN source_versions current_sv
                      ON current_sv.source_version_id = current_ne.source_version_id
                    WHERE current_ne.localized_name_id = ln.localized_name_id
                      AND (
                        current_ne.source_version_id IS NULL
                        OR current_sv.is_current = 1
                      )
                )
              )
            "#,
        )?;
        let rows = stmt.query_map(params![source_locale], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut grouped: HashMap<String, HashSet<i64>> = HashMap::new();
        for row in rows {
            let (concept_id, text, name_type) = row?;
            if safe_entity_surface(&text, &name_type) {
                grouped.entry(text).or_default().insert(concept_id);
            }
        }

        Ok(grouped
            .into_iter()
            .map(|(text, ids)| {
                let mut concepts: Vec<_> = ids
                    .into_iter()
                    .map(|concept_id| EntitySurface { concept_id })
                    .collect();
                concepts.sort_by_key(|item| item.concept_id);
                (text, concepts)
            })
            .collect())
    }

    fn target_entity_names(&self, concept_id: i64, target_locale: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT ln.text
            FROM localized_names ln
            WHERE ln.concept_id = ?1 AND ln.locale = ?2
              AND ln.name_type = 'preferred' AND ln.is_preferred = 1
              AND (
                NOT EXISTS (
                    SELECT 1 FROM name_evidence any_ne
                    WHERE any_ne.localized_name_id = ln.localized_name_id
                )
                OR EXISTS (
                    SELECT 1
                    FROM name_evidence current_ne
                    LEFT JOIN source_versions current_sv
                      ON current_sv.source_version_id = current_ne.source_version_id
                    WHERE current_ne.localized_name_id = ln.localized_name_id
                      AND (
                        current_ne.source_version_id IS NULL
                        OR current_sv.is_current = 1
                      )
                )
              )
            ORDER BY ln.text
            "#,
        )?;
        let rows = stmt.query_map(params![concept_id, target_locale], |row| row.get(0))?;
        Ok(rows.collect::<std::result::Result<Vec<String>, _>>()?)
    }

    fn apply_term_stage(
        &self,
        text: &str,
        protected: &[ProtectedSpan],
        source_locale: &str,
        target_locale: &str,
        context: Option<&Value>,
    ) -> Result<(String, Vec<ProtectedSpan>, Vec<Change>)> {
        let index = self.term_rule_index(source_locale, target_locale)?;
        let mut keys: Vec<&String> = index.keys().collect();
        keys.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));

        let mut spans = protected.to_vec();
        spans.sort_by_key(|span| span.start);
        let mut span_cursor = 0usize;
        let mut out = String::new();
        let mut new_protected = Vec::new();
        let mut changes = Vec::new();
        let mut i = 0usize;

        while i < text.len() {
            if let Some(span) = spans.get(span_cursor).copied() {
                if i == span.start {
                    let output_start = out.len();
                    out.push_str(&text[span.start..span.end]);
                    new_protected.push(ProtectedSpan {
                        start: output_start,
                        end: out.len(),
                    });
                    i = span.end;
                    span_cursor += 1;
                    continue;
                }
            }

            let next_protected_start = spans
                .get(span_cursor)
                .map(|span| span.start)
                .unwrap_or(text.len());

            let mut selected_key: Option<&str> = None;
            let mut selected_rows: Vec<&TermRule> = Vec::new();
            for key in &keys {
                let end = i + key.len();
                if end > next_protected_start || end > text.len() || !text[i..].starts_with(key.as_str()) {
                    continue;
                }
                let applicable: Vec<_> = index[*key]
                    .iter()
                    .filter(|rule| rule_context_allows(rule, text, i, end, context))
                    .collect();
                if !applicable.is_empty() {
                    selected_key = Some(key.as_str());
                    selected_rows = applicable;
                    break;
                }
            }

            let Some(source_text) = selected_key else {
                let ch = text[i..].chars().next().expect("valid char boundary");
                out.push(ch);
                i += ch.len_utf8();
                continue;
            };

            let max_priority = selected_rows
                .iter()
                .map(|rule| rule.priority)
                .max()
                .expect("non-empty rules");
            let winners: Vec<_> = selected_rows
                .into_iter()
                .filter(|rule| rule.priority == max_priority)
                .collect();
            let targets: HashSet<&str> = winners.iter().map(|rule| rule.target_text.as_str()).collect();

            if targets.len() != 1 {
                let output_start = out.len();
                out.push_str(source_text);
                new_protected.push(ProtectedSpan {
                    start: output_start,
                    end: out.len(),
                });
                changes.push(Change {
                    kind: "term_rule".into(),
                    reason: "ambiguous_rule_tie".into(),
                    original: source_text.into(),
                    replacement: source_text.into(),
                    applied: false,
                    review_needed: true,
                });
                i += source_text.len();
                continue;
            }

            let replacement = (*targets.iter().next().expect("one target")).to_owned();
            out.push_str(&replacement);
            changes.push(Change {
                kind: "term_rule".into(),
                reason: "term_rule".into(),
                original: source_text.into(),
                replacement: replacement.clone(),
                applied: replacement != source_text,
                review_needed: false,
            });
            i += source_text.len();
        }

        Ok((out, new_protected, changes))
    }

    fn term_rule_index(
        &self,
        source_locale: &str,
        target_locale: &str,
    ) -> Result<HashMap<String, Vec<TermRule>>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT tr.source_text, tr.target_text, tr.priority, tr.context_constraint
            FROM term_rules tr
            LEFT JOIN source_versions sv ON sv.source_version_id = tr.source_version_id
            WHERE tr.source_locale = ?1 AND tr.target_locale = ?2 AND tr.active = 1
              AND (tr.source_version_id IS NULL OR sv.is_current = 1)
            ORDER BY tr.source_text, tr.priority DESC, tr.rule_id
            "#,
        )?;
        let rows = stmt.query_map(params![source_locale, target_locale], |row| {
            Ok((
                row.get::<_, String>(0)?,
                TermRule {
                    target_text: row.get(1)?,
                    priority: row.get(2)?,
                    context_constraint: row.get(3)?,
                },
            ))
        })?;

        let mut index: HashMap<String, Vec<TermRule>> = HashMap::new();
        for row in rows {
            let (source, rule) = row?;
            index.entry(source).or_default().push(rule);
        }
        Ok(index)
    }
}

fn route(source: &str, target: &str) -> Result<Vec<(String, String)>> {
    let pairs = match (source, target) {
        ("zh-CN", "zh-HK") => vec![("zh-CN", "zh-Hant"), ("zh-Hant", "zh-HK")],
        ("zh-CN", "zh-TW") => vec![("zh-CN", "zh-Hant"), ("zh-Hant", "zh-TW")],
        ("zh-Hant", "zh-HK") => vec![("zh-Hant", "zh-HK")],
        ("zh-Hant", "zh-TW") => vec![("zh-Hant", "zh-TW")],
        _ => return Err(LocalizerError::UnsupportedRoute(source.into(), target.into())),
    };
    Ok(pairs
        .into_iter()
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect())
}

fn safe_entity_surface(text: &str, name_type: &str) -> bool {
    if text.is_empty() || text.trim() != text {
        return false;
    }
    let chars: Vec<char> = text.chars().collect();
    let has_cjk = chars.iter().any(|ch| ('\u{3400}'..='\u{9fff}').contains(ch));
    let pure_cjk = chars.iter().all(|ch| ('\u{3400}'..='\u{9fff}').contains(ch));
    let punctuation = chars
        .iter()
        .any(|ch| "·•・-–—()（）[]【】/\\ ".contains(*ch));
    let has_ascii = chars.iter().any(|ch| ch.is_ascii_alphanumeric());

    if has_cjk {
        let mut minimum = if name_type == "alias" && pure_cjk { 4 } else { 3 };
        if punctuation || has_ascii {
            minimum = 2;
        }
        chars.len() >= minimum
    } else {
        chars.len() >= 4
    }
}

fn boundary_ok(text: &str, start: usize, end: usize, surface: &str) -> bool {
    let first = surface.chars().next();
    let last = surface.chars().next_back();
    if first.is_some_and(|ch| ch.is_ascii_alphanumeric()) {
        if text[..start]
            .chars()
            .next_back()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
        {
            return false;
        }
    }
    if last.is_some_and(|ch| ch.is_ascii_alphanumeric()) {
        if text[end..]
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
        {
            return false;
        }
    }
    true
}

fn string_list(value: Option<&Value>) -> Vec<&str> {
    match value {
        Some(Value::String(item)) => vec![item.as_str()],
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn rule_context_allows(
    rule: &TermRule,
    text: &str,
    start: usize,
    end: usize,
    runtime_context: Option<&Value>,
) -> bool {
    let Some(raw) = &rule.context_constraint else {
        return true;
    };
    let Ok(metadata) = serde_json::from_str::<Value>(raw) else {
        return false;
    };
    let Some(constraints) = metadata.get("constraints") else {
        return true;
    };
    let Some(constraints) = constraints.as_object() else {
        return false;
    };

    let domains = string_list(constraints.get("domain"));
    if !domains.is_empty() {
        let runtime_domain = runtime_context
            .and_then(|ctx| ctx.get("domain"))
            .and_then(Value::as_str);
        if !runtime_domain.is_some_and(|domain| domains.contains(&domain)) {
            return false;
        }
    }

    for (key, positive, before) in [
        ("preceded_by", true, true),
        ("followed_by", true, false),
        ("not_preceded_by", false, true),
        ("not_followed_by", false, false),
    ] {
        let values = string_list(constraints.get(key));
        if values.is_empty() {
            continue;
        }
        let matched = if before {
            values.iter().any(|value| text[..start].ends_with(value))
        } else {
            values.iter().any(|value| text[end..].starts_with(value))
        };
        if matched != positive {
            return false;
        }
    }

    if constraints
        .get("word_boundary")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && !boundary_ok(text, start, end, "x")
    {
        return false;
    }
    true
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn entity_surface_policy_rejects_short_common_cjk() {
        assert!(!safe_entity_surface("苹果", "preferred"));
        assert!(safe_entity_surface("布拉德·皮特", "preferred"));
    }

    #[test]
    fn unsupported_route_fails() {
        assert!(route("zh-HK", "zh-CN").is_err());
    }
}
