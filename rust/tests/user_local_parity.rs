use chinese_regional_localizer::UserControlledLocalizer;
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const USER_SCHEMA: &str = r#"
CREATE TABLE user_terms (
    user_term_id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('protected','override')),
    source_text TEXT NOT NULL CHECK (length(source_text) > 0),
    replacement TEXT,
    source_locale TEXT,
    target_locale TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1)),
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK ((kind = 'protected' AND replacement IS NULL) OR (kind = 'override' AND replacement IS NOT NULL))
);
CREATE UNIQUE INDEX uq_user_terms_scope
ON user_terms(kind, source_text, ifnull(source_locale, ''), ifnull(target_locale, ''));
"#;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust crate inside repo")
        .to_path_buf()
}

fn setup() -> (TempDir, PathBuf, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let shared = temp.path().join("shared.sqlite");
    let user = temp.path().join("user.sqlite");
    fs::copy(repo_root().join("build/regional-demo.sqlite"), &shared).expect("copy shared db");
    let conn = Connection::open(&user).expect("open user db");
    conn.execute_batch(USER_SCHEMA).expect("create user schema");
    drop(conn);
    (temp, shared, user)
}

fn insert_term(
    user: &Path,
    kind: &str,
    source: &str,
    replacement: Option<&str>,
    source_locale: Option<&str>,
    target_locale: Option<&str>,
    priority: i64,
    enabled: bool,
) -> i64 {
    let conn = Connection::open(user).expect("open user db");
    conn.execute(
        "INSERT INTO user_terms (kind, source_text, replacement, source_locale, target_locale, priority, enabled, note, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, '2026-10-07T00:00:00Z', '2026-10-07T00:00:00Z')",
        params![kind, source, replacement, source_locale, target_locale, priority, i64::from(enabled)],
    )
    .expect("insert term");
    conn.last_insert_rowid()
}

#[test]
fn protected_term_beats_entity() {
    let (_temp, shared, user) = setup();
    insert_term(&user, "protected", "布拉德·皮特", None, None, None, 0, true);
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("布拉德·皮特", "zh-CN", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "布拉德·皮特");
    assert_eq!(result.changes[0]["type"], "user_protected");
    assert_eq!(result.changes[0]["reason"], "protected_by_user");
    assert_eq!(result.changes[0]["provenance"], "user_dictionary");
}

#[test]
fn override_beats_entity_and_reports_span() {
    let (_temp, shared, user) = setup();
    let term_id = insert_term(
        &user,
        "override",
        "布拉德·皮特",
        Some("我指定的彼特"),
        Some("zh-CN"),
        Some("zh-HK"),
        0,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("布拉德·皮特", "zh-CN", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "我指定的彼特");
    assert_eq!(result.changes[0]["user_term_id"], term_id);
    assert_eq!(
        result.changes[0]["final_output_span"],
        serde_json::json!([0, 6])
    );
}

#[test]
fn protected_term_beats_opencc() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "protected",
        "人工智能",
        None,
        Some("zh-CN"),
        Some("zh-TW"),
        0,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("人工智能", "zh-CN", "zh-TW", None)
        .expect("localize");
    assert_eq!(result.output, "人工智能");
    assert!(!result
        .changes
        .iter()
        .any(|event| event["type"] == "term_rule"));
}

#[test]
fn locale_scoped_override_does_not_leak() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "override",
        "人工智能",
        Some("AI香港用詞"),
        Some("zh-CN"),
        Some("zh-HK"),
        0,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let hk = engine
        .localize("人工智能", "zh-CN", "zh-HK", None)
        .expect("hk");
    let tw = engine
        .localize("人工智能", "zh-CN", "zh-TW", None)
        .expect("tw");
    assert_eq!(hk.output, "AI香港用詞");
    assert_eq!(tw.output, "人工智慧");
}

#[test]
fn longest_user_surface_wins() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "override",
        "人工",
        Some("人造"),
        None,
        Some("zh-TW"),
        0,
        true,
    );
    insert_term(
        &user,
        "override",
        "人工智能",
        Some("自訂AI"),
        None,
        Some("zh-TW"),
        0,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("人工智能", "zh-CN", "zh-TW", None)
        .expect("localize");
    assert_eq!(result.output, "自訂AI");
}

#[test]
fn protected_beats_override_for_same_surface() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "override",
        "OpenAI",
        Some("開放人工智能"),
        None,
        Some("zh-HK"),
        0,
        true,
    );
    insert_term(
        &user,
        "protected",
        "OpenAI",
        None,
        None,
        Some("zh-HK"),
        0,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("OpenAI", "zh-CN", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "OpenAI");
    assert_eq!(result.changes[0]["type"], "user_protected");
}

#[test]
fn more_specific_scope_beats_global_override() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "override",
        "測試詞",
        Some("全球答案"),
        None,
        None,
        0,
        true,
    );
    insert_term(
        &user,
        "override",
        "測試詞",
        Some("香港答案"),
        Some("zh-Hant"),
        Some("zh-HK"),
        0,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("測試詞", "zh-Hant", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "香港答案");
}

#[test]
fn disabled_rule_restores_shared_behavior() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "override",
        "人工智能",
        Some("停用答案"),
        None,
        Some("zh-TW"),
        0,
        false,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("人工智能", "zh-CN", "zh-TW", None)
        .expect("localize");
    assert_eq!(result.output, "人工智慧");
    assert!(!result.user_dictionary_applied);
}

#[test]
fn equal_rank_conflicting_overrides_are_not_guessed() {
    let (_temp, shared, user) = setup();
    insert_term(
        &user,
        "override",
        "測試詞",
        Some("答案甲"),
        Some("zh-Hant"),
        None,
        5,
        true,
    );
    insert_term(
        &user,
        "override",
        "測試詞",
        Some("答案乙"),
        None,
        Some("zh-HK"),
        5,
        true,
    );
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("測試詞", "zh-Hant", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "測試詞");
    assert!(result.review_needed);
    assert_eq!(result.changes[0]["reason"], "ambiguous_user_override");
}

#[test]
fn shared_changes_keep_global_spans_after_user_segment() {
    let (_temp, shared, user) = setup();
    insert_term(&user, "protected", "OpenAI", None, None, None, 0, true);
    let engine = UserControlledLocalizer::open(&shared, &user).expect("engine");
    let result = engine
        .localize("OpenAI人工智能", "zh-CN", "zh-TW", None)
        .expect("localize");
    assert_eq!(result.output, "OpenAI人工智慧");
    let shared_event = result
        .changes
        .iter()
        .find(|event| event["type"] == "term_rule" && event["original"] == "人工智能")
        .expect("shared term event");
    assert_eq!(
        shared_event["original_input_span"],
        serde_json::json!([6, 10])
    );
    assert_eq!(
        shared_event["final_output_span"],
        serde_json::json!([6, 10])
    );
}
