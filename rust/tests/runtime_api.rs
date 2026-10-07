use chinese_regional_localizer::{Runtime, RuntimeRequest, RUNTIME_API_VERSION};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const USER_SCHEMA: &str = r#"
CREATE TABLE user_terms (
    user_term_id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    source_text TEXT NOT NULL,
    replacement TEXT,
    source_locale TEXT,
    target_locale TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust crate inside repo")
        .to_path_buf()
}

fn demo_db() -> PathBuf {
    repo_root().join("build/regional-demo.sqlite")
}

fn request(text: &str, source: &str, target: &str) -> RuntimeRequest {
    RuntimeRequest {
        api_version: RUNTIME_API_VERSION.to_owned(),
        text: text.to_owned(),
        source_locale: source.to_owned(),
        target_locale: target.to_owned(),
        context: None,
    }
}

#[test]
fn shared_entity_event_contains_qid_and_evidence() {
    let runtime = Runtime::new(demo_db(), Option::<&Path>::None);
    let result = runtime
        .localize(&request("布拉德·皮特", "zh-CN", "zh-HK"))
        .expect("localize");
    let event = result
        .changes
        .iter()
        .find(|event| event["type"] == "entity")
        .expect("entity event");
    assert!(event["concept_id"].as_i64().is_some());
    assert!(event["qid"].as_str().is_some());
    assert!(event["evidence"].as_array().is_some());
    assert_eq!(event["original_input_span"], serde_json::json!([0, 6]));
    assert_eq!(event["final_output_span"], serde_json::json!([0, 5]));
}

#[test]
fn shared_term_event_contains_rule_provenance() {
    let runtime = Runtime::new(demo_db(), Option::<&Path>::None);
    let result = runtime
        .localize(&request("人工智能", "zh-CN", "zh-TW"))
        .expect("localize");
    let event = result
        .changes
        .iter()
        .find(|event| event["type"] == "term_rule" && event["replacement"] == "人工智慧")
        .expect("term event");
    assert!(event["rule_id"].as_i64().is_some());
    assert_eq!(event["source_id"], "opencc");
    assert!(event["stage"].as_str().is_some());
    assert!(event["upstream_record_id"].as_str().is_some());
    assert_eq!(event["original_input_span"], serde_json::json!([0, 4]));
    assert_eq!(event["final_output_span"], serde_json::json!([0, 4]));
}

#[test]
fn runtime_uses_user_database_when_configured() {
    let temp = TempDir::new().expect("tempdir");
    let user_db = temp.path().join("user.sqlite");
    let conn = Connection::open(&user_db).expect("user db");
    conn.execute_batch(USER_SCHEMA).expect("schema");
    conn.execute(
        "INSERT INTO user_terms (kind, source_text, replacement, source_locale, target_locale, priority, enabled, note, created_at, updated_at) VALUES ('override', '人工智能', '我的AI', 'zh-CN', 'zh-TW', 0, 1, 'runtime test', '2026-10-07T00:00:00Z', '2026-10-07T00:00:00Z')",
        params![],
    )
    .expect("insert");
    drop(conn);

    let runtime = Runtime::new(demo_db(), Some(&user_db));
    let result = runtime
        .localize(&request("人工智能", "zh-CN", "zh-TW"))
        .expect("localize");
    assert_eq!(result.output, "我的AI");
    assert!(result.user_dictionary_applied);
    assert_eq!(result.changes[0]["provenance"], "user_dictionary");
}

#[test]
fn response_is_versioned() {
    let runtime = Runtime::new(demo_db(), Option::<&Path>::None);
    let result = runtime
        .localize(&request("保持原樣", "zh-Hant", "zh-TW"))
        .expect("localize");
    assert_eq!(result.api_version, RUNTIME_API_VERSION);
}
