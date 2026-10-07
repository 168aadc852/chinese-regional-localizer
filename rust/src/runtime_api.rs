use crate::{LocalizerEngine, Result, UserControlledLocalizer};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const RUNTIME_API_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    pub text: String,
    pub source_locale: String,
    pub target_locale: String,
    #[serde(default)]
    pub context: Option<Value>,
}

fn default_api_version() -> String {
    RUNTIME_API_VERSION.to_owned()
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeResponse {
    pub api_version: String,
    pub input: String,
    pub output: String,
    pub source_locale: String,
    pub target_locale: String,
    pub route: Vec<String>,
    pub changes: Vec<Value>,
    pub review_needed: bool,
    pub user_dictionary_applied: bool,
}

pub struct Runtime {
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
}

impl Runtime {
    pub fn new(shared_db: impl AsRef<Path>, user_db: Option<impl AsRef<Path>>) -> Self {
        Self {
            shared_db: shared_db.as_ref().to_path_buf(),
            user_db: user_db.map(|path| path.as_ref().to_path_buf()),
        }
    }

    pub fn localize(&self, request: &RuntimeRequest) -> Result<RuntimeResponse> {
        let context = request.context.as_ref();
        let (output, route, review_needed, user_dictionary_applied, raw_changes) =
            if let Some(user_db) = &self.user_db {
                let engine = UserControlledLocalizer::open(&self.shared_db, user_db)?;
                let result = engine.localize(
                    &request.text,
                    &request.source_locale,
                    &request.target_locale,
                    context,
                )?;
                (
                    result.output,
                    result.route,
                    result.review_needed,
                    result.user_dictionary_applied,
                    result.changes,
                )
            } else {
                let engine = LocalizerEngine::open(&self.shared_db)?;
                let result = engine.localize(
                    &request.text,
                    &request.source_locale,
                    &request.target_locale,
                    context,
                )?;
                let changes = result
                    .changes
                    .into_iter()
                    .map(|change| {
                        json!({
                            "type": change.kind,
                            "reason": change.reason,
                            "original": change.original,
                            "replacement": change.replacement,
                            "applied": change.applied,
                            "review_needed": change.review_needed,
                            "source_locale": request.source_locale,
                            "target_locale": request.target_locale,
                            "provenance": "shared_database",
                        })
                    })
                    .collect();
                (result.output, result.route, result.review_needed, false, changes)
            };

        let conn = Connection::open(&self.shared_db)?;
        let changes = enrich_changes(
            &conn,
            &request.text,
            &output,
            &route,
            &request.source_locale,
            &request.target_locale,
            raw_changes,
        )?;

        Ok(RuntimeResponse {
            api_version: RUNTIME_API_VERSION.to_owned(),
            input: request.text.clone(),
            output,
            source_locale: request.source_locale.clone(),
            target_locale: request.target_locale.clone(),
            route,
            changes,
            review_needed,
            user_dictionary_applied,
        })
    }
}

fn enrich_changes(
    conn: &Connection,
    input: &str,
    output: &str,
    route: &[String],
    source_locale: &str,
    target_locale: &str,
    changes: Vec<Value>,
) -> Result<Vec<Value>> {
    let mut input_cursors: HashMap<String, usize> = HashMap::new();
    let mut output_cursors: HashMap<String, usize> = HashMap::new();
    let mut enriched = Vec::with_capacity(changes.len());

    for change in changes {
        let mut object = change.as_object().cloned().unwrap_or_else(Map::new);
        let kind = object
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let original = object
            .get("original")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let replacement = object
            .get("replacement")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        if !object.contains_key("original_input_span") {
            if let Some(span) = next_char_span(input, &original, &mut input_cursors) {
                object.insert("original_input_span".into(), json!(span));
            }
        }
        if !object.contains_key("final_output_span") {
            if let Some(span) = next_char_span(output, &replacement, &mut output_cursors) {
                object.insert("final_output_span".into(), json!(span));
            }
        }

        if kind == "entity" {
            enrich_entity(conn, source_locale, target_locale, &original, &replacement, &mut object)?;
        } else if kind == "term_rule" {
            enrich_term_rule(conn, route, &original, &replacement, &mut object)?;
        }
        enriched.push(Value::Object(object));
    }
    Ok(enriched)
}

fn enrich_entity(
    conn: &Connection,
    source_locale: &str,
    target_locale: &str,
    original: &str,
    replacement: &str,
    object: &mut Map<String, Value>,
) -> Result<()> {
    let row = conn
        .query_row(
            r#"
            SELECT c.concept_id, c.concept_type,
                   (SELECT external_value FROM external_ids e
                    WHERE e.concept_id = c.concept_id AND e.namespace = 'wikidata'
                    LIMIT 1) AS qid
            FROM localized_names ln
            JOIN concepts c ON c.concept_id = ln.concept_id
            WHERE ln.locale = ?1 AND ln.text = ?2 AND ln.domain = 'entity'
            ORDER BY ln.is_preferred DESC, ln.confidence DESC, ln.localized_name_id
            LIMIT 1
            "#,
            params![source_locale, original],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?;
    if let Some((concept_id, concept_type, qid)) = row {
        object.insert("concept_id".into(), json!(concept_id));
        object.insert("concept_type".into(), json!(concept_type));
        object.insert("qid".into(), json!(qid));

        let target = conn
            .query_row(
                r#"
                SELECT ln.localized_name_id, ln.confidence
                FROM localized_names ln
                WHERE ln.concept_id = ?1 AND ln.locale = ?2 AND ln.text = ?3
                ORDER BY ln.is_preferred DESC, ln.confidence DESC, ln.localized_name_id
                LIMIT 1
                "#,
                params![concept_id, target_locale, replacement],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<f64>>(1)?)),
            )
            .optional()?;
        if let Some((localized_name_id, confidence)) = target {
            object.insert("confidence".into(), json!(confidence));
            let mut stmt = conn.prepare(
                r#"
                SELECT ne.source_id, ne.upstream_record_id, ne.upstream_url,
                       ne.upstream_revision, ne.evidence_type, ne.confidence,
                       ne.retrieved_at, sv.version_label, sv.revision_id,
                       sv.checksum_sha256
                FROM name_evidence ne
                LEFT JOIN source_versions sv ON sv.source_version_id = ne.source_version_id
                WHERE ne.localized_name_id = ?1
                  AND (ne.source_version_id IS NULL OR sv.is_current = 1)
                ORDER BY ne.evidence_id
                "#,
            )?;
            let rows = stmt.query_map(params![localized_name_id], |row| {
                Ok(json!({
                    "source_id": row.get::<_, Option<String>>(0)?,
                    "upstream_record_id": row.get::<_, Option<String>>(1)?,
                    "upstream_url": row.get::<_, Option<String>>(2)?,
                    "upstream_revision": row.get::<_, Option<String>>(3)?,
                    "evidence_type": row.get::<_, Option<String>>(4)?,
                    "confidence": row.get::<_, Option<f64>>(5)?,
                    "retrieved_at": row.get::<_, Option<String>>(6)?,
                    "version_label": row.get::<_, Option<String>>(7)?,
                    "revision_id": row.get::<_, Option<String>>(8)?,
                    "checksum_sha256": row.get::<_, Option<String>>(9)?,
                }))
            })?;
            object.insert(
                "evidence".into(),
                Value::Array(rows.collect::<std::result::Result<Vec<_>, _>>()?),
            );
        }
    }
    Ok(())
}

fn enrich_term_rule(
    conn: &Connection,
    route: &[String],
    original: &str,
    replacement: &str,
    object: &mut Map<String, Value>,
) -> Result<()> {
    for stage in route {
        let Some((source_locale, target_locale)) = stage.split_once("->") else {
            continue;
        };
        let row = conn
            .query_row(
                r#"
                SELECT tr.rule_id, tr.priority, tr.rule_type, tr.domain,
                       tr.source_id, tr.source_version_id, tr.upstream_record_id,
                       tr.upstream_url, tr.confidence, sv.version_label,
                       sv.revision_id, sv.checksum_sha256
                FROM term_rules tr
                LEFT JOIN source_versions sv ON sv.source_version_id = tr.source_version_id
                WHERE tr.source_locale = ?1 AND tr.target_locale = ?2
                  AND tr.source_text = ?3 AND tr.target_text = ?4 AND tr.active = 1
                  AND (tr.source_version_id IS NULL OR sv.is_current = 1)
                ORDER BY tr.priority DESC, tr.rule_id
                LIMIT 1
                "#,
                params![source_locale, target_locale, original, replacement],
                |row| {
                    Ok(json!({
                        "rule_id": row.get::<_, i64>(0)?,
                        "priority": row.get::<_, i64>(1)?,
                        "rule_type": row.get::<_, String>(2)?,
                        "domain": row.get::<_, Option<String>>(3)?,
                        "source_id": row.get::<_, String>(4)?,
                        "source_version_id": row.get::<_, Option<i64>>(5)?,
                        "upstream_record_id": row.get::<_, Option<String>>(6)?,
                        "upstream_url": row.get::<_, Option<String>>(7)?,
                        "confidence": row.get::<_, Option<f64>>(8)?,
                        "version_label": row.get::<_, Option<String>>(9)?,
                        "revision_id": row.get::<_, Option<String>>(10)?,
                        "checksum_sha256": row.get::<_, Option<String>>(11)?,
                    }))
                },
            )
            .optional()?;
        if let Some(Value::Object(metadata)) = row {
            object.insert("stage".into(), json!(stage));
            for (key, value) in metadata {
                object.insert(key, value);
            }
            break;
        }
    }
    Ok(())
}

fn next_char_span(
    haystack: &str,
    needle: &str,
    cursors: &mut HashMap<String, usize>,
) -> Option<[usize; 2]> {
    if needle.is_empty() {
        return None;
    }
    let cursor = cursors.get(needle).copied().unwrap_or(0);
    let relative = haystack[cursor..].find(needle)?;
    let byte_start = cursor + relative;
    let byte_end = byte_start + needle.len();
    cursors.insert(needle.to_owned(), byte_end);
    Some([
        haystack[..byte_start].chars().count(),
        haystack[..byte_end].chars().count(),
    ])
}
