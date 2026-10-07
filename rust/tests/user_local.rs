use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[path = "../src/user_local.rs"]
mod user_local;
use user_local::UserControlledLocalizer;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust crate inside repo")
        .to_path_buf()
}

fn demo_db() -> PathBuf {
    repo_root().join("build/regional-demo.sqlite")
}

fn user_db() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("user_dictionary.sqlite");
    let conn = Connection::open(&path).expect("user db");
    conn.execute_batch(
        r#"
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
            created_at TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT ''
        );
        "#,
    )
    .expect("schema");
    drop(conn);
    (temp, path)
}

fn add_term(
    path: &Path,
    kind: &str,
    source: &str,
    replacement: Option<&str>,
    source_locale: Option<&str>,
    target_locale: Option<&str>,
    priority: i64,
    enabled: bool,
) -> i64 {
    let conn = Connection::open(path).expect("user db");
    conn.execute(
        "INSERT INTO user_terms (kind, source_text, replacement, source_locale, target_locale, priority, enabled, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'test')",
        params![kind, source, replacement, source_locale, target_locale, priority, i64::from(enabled)],
    )
    .expect("insert term");
    conn.last_insert_rowid()
}

#[test]
fn protected_beats_entity() {
    let (_temp, user) = user_db();
    add_term(&user, "protected", "布拉德·皮特", None, None, None, 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    let result = engine.localize("布拉德·皮特", "zh-CN", "zh-HK", None).expect("localize");
    assert_eq!(result.output, "布拉德·皮特");
    assert_eq!(result.changes[0].kind, "user_protected");
}

#[test]
fn override_beats_entity_and_reports_span() {
    let (_temp, user) = user_db();
    let id = add_term(&user, "override", "布拉德·皮特", Some("我指定的彼特"), Some("zh-CN"), Some("zh-HK"), 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    let result = engine.localize("布拉德·皮特", "zh-CN", "zh-HK", None).expect("localize");
    assert_eq!(result.output, "我指定的彼特");
    assert_eq!(result.changes[0].user_term_id, Some(id));
    assert_eq!(result.changes[0].final_output_span, [0, 6]);
}

#[test]
fn protected_beats_opencc() {
    let (_temp, user) = user_db();
    add_term(&user, "protected", "人工智能", None, Some("zh-CN"), Some("zh-TW"), 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    let result = engine.localize("人工智能", "zh-CN", "zh-TW", None).expect("localize");
    assert_eq!(result.output, "人工智能");
}

#[test]
fn locale_scope_does_not_leak() {
    let (_temp, user) = user_db();
    add_term(&user, "override", "人工智能", Some("AI香港用詞"), Some("zh-CN"), Some("zh-HK"), 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    assert_eq!(engine.localize("人工智能", "zh-CN", "zh-HK", None).unwrap().output, "AI香港用詞");
    assert_eq!(engine.localize("人工智能", "zh-CN", "zh-TW", None).unwrap().output, "人工智慧");
}

#[test]
fn longest_user_surface_wins() {
    let (_temp, user) = user_db();
    add_term(&user, "override", "人工", Some("人造"), None, Some("zh-TW"), 0, true);
    add_term(&user, "override", "人工智能", Some("自訂AI"), None, Some("zh-TW"), 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    assert_eq!(engine.localize("人工智能", "zh-CN", "zh-TW", None).unwrap().output, "自訂AI");
}

#[test]
fn protected_beats_override_same_surface() {
    let (_temp, user) = user_db();
    add_term(&user, "override", "OpenAI", Some("開放人工智能"), None, Some("zh-HK"), 0, true);
    add_term(&user, "protected", "OpenAI", None, None, Some("zh-HK"), 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    let result = engine.localize("OpenAI", "zh-CN", "zh-HK", None).unwrap();
    assert_eq!(result.output, "OpenAI");
    assert_eq!(result.changes[0].kind, "user_protected");
}

#[test]
fn more_specific_scope_beats_global() {
    let (_temp, user) = user_db();
    add_term(&user, "override", "測試詞", Some("全球答案"), None, None, 0, true);
    add_term(&user, "override", "測試詞", Some("香港答案"), Some("zh-Hant"), Some("zh-HK"), 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    assert_eq!(engine.localize("測試詞", "zh-Hant", "zh-HK", None).unwrap().output, "香港答案");
}

#[test]
fn disabled_rule_restores_shared_behavior() {
    let (_temp, user) = user_db();
    add_term(&user, "override", "人工智能", Some("停用答案"), None, Some("zh-TW"), 0, false);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    assert_eq!(engine.localize("人工智能", "zh-CN", "zh-TW", None).unwrap().output, "人工智慧");
}

#[test]
fn conflicting_top_overrides_are_not_guessed() {
    let (_temp, user) = user_db();
    add_term(&user, "override", "測試詞", Some("答案甲"), None, Some("zh-HK"), 5, true);
    add_term(&user, "override", "測試詞", Some("答案乙"), None, Some("zh-HK"), 5, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    let result = engine.localize("測試詞", "zh-Hant", "zh-HK", None).unwrap();
    assert_eq!(result.output, "測試詞");
    assert!(result.review_needed);
    assert_eq!(result.changes[0].reason, "ambiguous_user_override");
}

#[test]
fn shared_change_spans_are_global_after_user_segment() {
    let (_temp, user) = user_db();
    add_term(&user, "protected", "OpenAI", None, None, None, 0, true);
    let engine = UserControlledLocalizer::open(demo_db(), &user).expect("engine");
    let result = engine.localize("OpenAI人工智能", "zh-CN", "zh-TW", None).unwrap();
    assert_eq!(result.output, "OpenAI人工智慧");
    let shared = result.changes.iter().find(|event| event.kind == "term_rule").expect("shared change");
    assert_eq!(shared.original_input_span, [6, 10]);
    assert_eq!(shared.final_output_span, [6, 10]);
}
