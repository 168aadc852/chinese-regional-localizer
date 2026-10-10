//! Local versioned private terminology. Shared databases are never migrated here.
use crate::context_profiles::ContextProfiles;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub type StoreResult<T> = Result<T, StoreError>;
#[derive(Debug)]
pub enum StoreError {
    Sql(rusqlite::Error),
    Invalid(String),
}
impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(_) => f.write_str("Private store operation failed; nothing remembered"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for StoreError {}
fn invalid(message: &str) -> StoreError {
    StoreError::Invalid(message.into())
}

pub(crate) fn schema_version(conn: &Connection) -> StoreResult<String> {
    if !conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='user_metadata'")?
        .exists([])?
    {
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(user_terms)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<_, _>>()?;
        if columns
            == [
                "user_term_id",
                "kind",
                "source_text",
                "replacement",
                "source_locale",
                "target_locale",
                "priority",
                "enabled",
                "note",
                "created_at",
                "updated_at",
            ]
        {
            return Ok("legacy-unversioned".into());
        }
        return Err(invalid("Unrecognized unversioned private store"));
    }
    let version: Option<String> = conn
        .query_row(
            "SELECT value FROM user_metadata WHERE key='schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match version.as_deref() {
        Some("0.1" | "2") => Ok(version.unwrap()),
        _ => Err(invalid("Unsupported private-store schema version")),
    }
}
fn validate(conn: &Connection) -> StoreResult<()> {
    let integrity: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if integrity != "ok" || conn.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(invalid("Invalid private store"));
    }
    for id in ["personal", "legacy"] {
        let reserved: Option<i64> = conn
            .query_row(
                "SELECT system_managed FROM user_dictionaries WHERE dictionary_id=?",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        if reserved != Some(1) {
            return Err(invalid("Missing reserved private dictionary"));
        }
    }
    conn.prepare("SELECT dictionary_id,usage_context_id,legacy FROM user_terms LIMIT 0")?;
    Ok(())
}

pub(crate) fn prepare_store(conn: &mut Connection) -> StoreResult<()> {
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table'",
        [],
        |r| r.get(0),
    )?;
    let fresh = count == 0;
    let version = if fresh {
        String::new()
    } else {
        schema_version(conn)?
    };
    if version == "2" {
        return validate(conn);
    }
    if !fresh && conn.prepare("SELECT name FROM sqlite_master WHERE type='view' OR (type='trigger' AND tbl_name='user_terms') OR (type='table' AND name NOT IN ('user_terms','user_metadata') AND name NOT LIKE 'sqlite_%') LIMIT 1")?.exists([])? {
        return Err(invalid("Unsupported legacy private-store extension; original store retained"));
    }
    let transaction = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let before: i64 = if fresh {
        0
    } else {
        transaction.query_row("SELECT count(*) FROM user_terms", [], |r| r.get(0))?
    };
    if version == "legacy-unversioned" {
        transaction.execute_batch("CREATE TABLE user_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL); INSERT INTO user_metadata VALUES ('schema_version','0.1');")?;
    }
    transaction.execute_batch(if fresh {
        include_str!("../../schema/user-dictionary-v2.sql")
    } else {
        include_str!("../../schema/private-migrations/0001-v2.sql")
    })?;
    let after: i64 = transaction.query_row("SELECT count(*) FROM user_terms", [], |r| r.get(0))?;
    if before != after {
        return Err(invalid("Private migration count mismatch"));
    }
    validate(&transaction)?;
    transaction.commit()?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub source: String,
    pub replacement: String,
    pub target: String,
    pub usage: Option<String>,
    pub source_locale: Option<String>,
    pub note: Option<String>,
}
fn locale(value: &str) -> bool {
    matches!(value, "zh-CN" | "zh-Hant" | "zh-HK" | "zh-TW")
}
fn validate_term(term: &Preference, profiles: &ContextProfiles) -> StoreResult<()> {
    if term.source.trim().is_empty()
        || term.replacement.is_empty()
        || !locale(&term.target)
        || term
            .source_locale
            .as_ref()
            .is_some_and(|value| !locale(value))
    {
        return Err(invalid(
            "Source, preferred text and valid target locale are required",
        ));
    }
    if let Some(id) = &term.usage {
        profiles
            .resolve_chain(id)
            .map_err(|_| invalid("Invalid Usage Context"))?;
    }
    Ok(())
}
fn put(
    conn: &Connection,
    dictionary: &str,
    term: &Preference,
    profiles: &ContextProfiles,
) -> StoreResult<i64> {
    validate_term(term, profiles)?;
    conn.execute("INSERT INTO user_terms(kind,source_text,replacement,source_locale,target_locale,priority,enabled,note,created_at,updated_at,dictionary_id,usage_context_id,legacy)
        VALUES ('override',?1,?2,?3,?4,0,1,?5,strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now'),?6,?7,0)
        ON CONFLICT DO UPDATE SET replacement=excluded.replacement,enabled=1,note=excluded.note,updated_at=excluded.updated_at,legacy=0,priority=0",
        params![term.source, term.replacement, term.source_locale, term.target, term.note, dictionary, term.usage])?;
    Ok(conn.query_row("SELECT user_term_id FROM user_terms WHERE dictionary_id=?1 AND kind='override' AND source_text=?2 AND source_locale IS ?3 AND target_locale=?4 AND usage_context_id IS ?5",
        params![dictionary,term.source,term.source_locale,term.target,term.usage], |row| row.get(0))?)
}

pub struct PrivateStore {
    conn: Connection,
    profiles: ContextProfiles,
}
impl PrivateStore {
    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        Self::from_connection(Connection::open(path)?)
    }
    pub fn from_connection(mut conn: Connection) -> StoreResult<Self> {
        prepare_store(&mut conn)?;
        Ok(Self {
            conn,
            profiles: ContextProfiles::default(),
        })
    }
    pub fn with_context_profiles(mut self, profiles: ContextProfiles) -> Self {
        self.profiles = profiles;
        self
    }
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
    pub fn create_dictionary(&mut self, id: &str, name: &str) -> StoreResult<()> {
        if id.trim().is_empty() || name.trim().is_empty() {
            return Err(invalid("Dictionary identity and name are required"));
        }
        self.conn.execute(
            "INSERT INTO user_dictionaries(dictionary_id,name) VALUES (?,?)",
            params![id, name],
        )?;
        Ok(())
    }
    fn mutable_dictionary(&self, id: &str) -> StoreResult<()> {
        let reserved: Option<i64> = self
            .conn
            .query_row(
                "SELECT system_managed FROM user_dictionaries WHERE dictionary_id=?",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        if reserved != Some(0) {
            return Err(invalid("Unknown or reserved dictionary"));
        }
        Ok(())
    }
    pub fn rename_dictionary(&mut self, id: &str, name: &str) -> StoreResult<()> {
        if name.trim().is_empty() {
            return Err(invalid("Dictionary name is required"));
        }
        self.mutable_dictionary(id)?;
        self.conn.execute(
            "UPDATE user_dictionaries SET name=? WHERE dictionary_id=?",
            params![name, id],
        )?;
        Ok(())
    }
    pub fn delete_dictionary(&mut self, id: &str) -> StoreResult<()> {
        self.mutable_dictionary(id)?;
        let transaction = self.conn.transaction()?;
        transaction.execute("DELETE FROM user_terms WHERE dictionary_id=?", [id])?;
        transaction.execute("DELETE FROM user_dictionaries WHERE dictionary_id=?", [id])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn set_dictionary_enabled(&mut self, id: &str, enabled: bool) -> StoreResult<()> {
        if self.conn.execute(
            "UPDATE user_dictionaries SET enabled=? WHERE dictionary_id=?",
            params![enabled, id],
        )? == 0
        {
            return Err(invalid("Unknown dictionary"));
        }
        Ok(())
    }
    pub fn put_preference(&mut self, dictionary: &str, term: &Preference) -> StoreResult<i64> {
        let transaction = self.conn.transaction()?;
        let id = put(&transaction, dictionary, term, &self.profiles)?;
        transaction.commit()?;
        Ok(id)
    }
    pub fn delete_term(&mut self, id: i64) -> StoreResult<()> {
        self.conn
            .execute("DELETE FROM user_terms WHERE user_term_id=?", [id])?;
        Ok(())
    }
    pub fn edit_preference(&mut self, id: i64, term: &Preference) -> StoreResult<()> {
        validate_term(term, &self.profiles)?;
        let transaction = self.conn.transaction()?;
        if transaction.execute("UPDATE user_terms SET source_text=?1,replacement=?2,target_locale=?3,usage_context_id=?4,source_locale=?5,note=?6,legacy=0,priority=0,updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE user_term_id=?7 AND kind='override'",
            params![term.source,term.replacement,term.target,term.usage,term.source_locale,term.note,id])?==0 {return Err(invalid("Unknown preferred term"));}
        transaction.commit()?;
        Ok(())
    }
    pub fn set_term_enabled(&mut self, id: i64, enabled: bool) -> StoreResult<()> {
        self.conn.execute(
            "UPDATE user_terms SET enabled=? WHERE user_term_id=?",
            params![enabled, id],
        )?;
        Ok(())
    }
    pub fn preview_csv(&self, text: &str, target: Option<&str>) -> CsvPreview {
        let mut preview = CsvPreview {
            rows: Vec::new(),
            errors: Vec::new(),
        };
        let parsed = match parse_csv(text) {
            Ok(rows) => rows,
            Err((row, reason)) => {
                preview.errors.push(CsvProblem { row, reason });
                return preview;
            }
        };
        let Some((_, header)) = parsed.first() else {
            preview.errors.push(CsvProblem {
                row: 1,
                reason: "Missing CSV header".into(),
            });
            return preview;
        };
        let header = header.clone();
        let allowed = [
            "source_text",
            "replacement",
            "target_locale",
            "usage_context_id",
            "note",
        ];
        if !["source_text", "replacement"]
            .iter()
            .all(|name| header.iter().any(|value| value == name))
            || header.iter().any(|name| !allowed.contains(&name.as_str()))
            || header
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != header.len()
        {
            preview.errors.push(CsvProblem {
                row: 1,
                reason: "Invalid CSV header".into(),
            });
            return preview;
        }
        let mut scopes = std::collections::BTreeMap::new();
        for (row, values) in parsed.into_iter().skip(1) {
            if values.len() != header.len() {
                preview.errors.push(CsvProblem {
                    row,
                    reason: "Wrong column count".into(),
                });
                continue;
            }
            let get = |name: &str| {
                header
                    .iter()
                    .position(|value| value == name)
                    .map(|index| values[index].clone())
                    .filter(|value| !value.is_empty())
            };
            let term = Preference {
                source: get("source_text").unwrap_or_default(),
                replacement: get("replacement").unwrap_or_default(),
                target: get("target_locale")
                    .or_else(|| target.map(str::to_owned))
                    .unwrap_or_default(),
                usage: get("usage_context_id"),
                source_locale: None,
                note: get("note"),
            };
            match validate_term(&term, &self.profiles) {
                Ok(()) => {
                    let scope = (term.source.clone(), term.target.clone(), term.usage.clone());
                    if scopes
                        .get(&scope)
                        .is_some_and(|replacement| replacement != &term.replacement)
                    {
                        preview.errors.push(CsvProblem {
                            row,
                            reason: "Conflicting duplicate CSV scope".into(),
                        });
                    } else {
                        scopes.insert(scope, term.replacement.clone());
                        preview.rows.push(term);
                    }
                }
                Err(error) => preview.errors.push(CsvProblem {
                    row,
                    reason: error.to_string(),
                }),
            }
        }
        preview
    }
    pub fn commit_csv(&mut self, dictionary: &str, preview: &CsvPreview) -> StoreResult<()> {
        if !preview.errors.is_empty() {
            return Err(invalid("CSV has invalid rows; nothing imported"));
        }
        let transaction = self.conn.transaction()?;
        let mut scopes = std::collections::BTreeMap::new();
        for term in &preview.rows {
            let scope = (&term.source, &term.target, &term.usage, &term.source_locale);
            if scopes
                .get(&scope)
                .is_some_and(|replacement| *replacement != &term.replacement)
            {
                return Err(invalid("Conflicting duplicate CSV scope"));
            }
            scopes.insert(scope, &term.replacement);
            put(&transaction, dictionary, term, &self.profiles)?;
        }
        transaction.commit()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvProblem {
    pub row: usize,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct CsvPreview {
    pub rows: Vec<Preference>,
    pub errors: Vec<CsvProblem>,
}

// RFC-style quoted fields, escaped quotes, embedded newlines, CRLF; no file/network IO.
type CsvRecords = Vec<(usize, Vec<String>)>;
type CsvParseResult = Result<CsvRecords, (usize, String)>;
fn parse_csv(text: &str) -> CsvParseResult {
    let mut rows = Vec::new();
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut closed = false;
    let mut line = 1;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                if ch == '\n' {
                    line += 1;
                }
                field.push(ch);
            }
        } else {
            match ch {
                '"' if field.is_empty() && !closed => quoted = true,
                ',' => {
                    fields.push(std::mem::take(&mut field));
                    closed = false;
                }
                '\n' | '\r' => {
                    if ch == '\r' && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    if !fields.is_empty() || !field.is_empty() || closed {
                        fields.push(std::mem::take(&mut field));
                        rows.push((line, std::mem::take(&mut fields)));
                    }
                    closed = false;
                    line += 1;
                }
                _ if closed => return Err((line, "Invalid quoted CSV field".into())),
                _ => field.push(ch),
            }
        }
    }
    if quoted {
        return Err((line, "Unterminated quoted CSV field".into()));
    }
    if !fields.is_empty() || !field.is_empty() || closed {
        fields.push(field);
        rows.push((line, fields));
    }
    Ok(rows)
}
