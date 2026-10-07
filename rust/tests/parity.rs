use chinese_regional_localizer::LocalizerEngine;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    source_locale: String,
    target_locale: String,
    input: String,
    expected: String,
    #[serde(default)]
    expected_review_needed: bool,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust crate inside repo")
        .to_path_buf()
}

fn demo_db() -> PathBuf {
    repo_root().join("build/regional-demo.sqlite")
}

fn read_cases(path: &str) -> Vec<Case> {
    let data = fs::read_to_string(repo_root().join(path)).expect("read corpus");
    serde_json::from_str(&data).expect("valid corpus json")
}

fn copied_demo_db() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("test.sqlite");
    fs::copy(demo_db(), &path).expect("copy demo db");
    (temp, path)
}

#[test]
fn short_evaluation_cases_match_python_contract() {
    let engine = LocalizerEngine::open(demo_db()).expect("open demo db");
    for case in read_cases("data/fixtures/evaluation_cases.json") {
        let result = engine
            .localize(
                &case.input,
                &case.source_locale,
                &case.target_locale,
                None,
            )
            .unwrap_or_else(|error| panic!("{} failed: {error:?}", case.id));
        assert_eq!(result.output, case.expected, "case {}", case.id);
        assert!(!result.review_needed, "case {} unexpectedly needs review", case.id);
    }
}

#[test]
fn realistic_corpus_matches_python_contract() {
    let engine = LocalizerEngine::open(demo_db()).expect("open demo db");
    for case in read_cases("data/fixtures/realistic_corpus.json") {
        let result = engine
            .localize(
                &case.input,
                &case.source_locale,
                &case.target_locale,
                None,
            )
            .unwrap_or_else(|error| panic!("{} failed: {error:?}", case.id));
        assert_eq!(result.output, case.expected, "case {}", case.id);
        assert_eq!(
            result.review_needed, case.expected_review_needed,
            "case {} review state",
            case.id
        );
    }
}

#[test]
fn ambiguous_entity_is_preserved_and_reviewed() {
    let (_temp, path) = copied_demo_db();
    let conn = Connection::open(&path).expect("open copy");
    let concept_id = conn
        .execute(
            "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES ('person', 'rust:duplicate-person', 'entity')",
            [],
        )
        .map(|_| conn.last_insert_rowid())
        .expect("insert concept");
    conn.execute(
        "INSERT INTO localized_names (concept_id, locale, text, name_type, domain, is_preferred, confidence) VALUES (?1, 'zh-CN', '布拉德·皮特', 'preferred', 'entity', 1, 1.0)",
        params![concept_id],
    )
    .expect("insert source name");
    conn.execute(
        "INSERT INTO localized_names (concept_id, locale, text, name_type, domain, is_preferred, confidence) VALUES (?1, 'zh-HK', '另一個人', 'preferred', 'entity', 1, 1.0)",
        params![concept_id],
    )
    .expect("insert target name");
    drop(conn);

    let engine = LocalizerEngine::open(&path).expect("open engine");
    let result = engine
        .localize("布拉德·皮特", "zh-CN", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "布拉德·皮特");
    assert!(result.review_needed);
}

#[test]
fn ambiguous_term_rule_is_preserved_and_reviewed() {
    let (_temp, path) = copied_demo_db();
    let conn = Connection::open(&path).expect("open copy");
    for (record, target) in [("rust:tie:a", "候選甲"), ("rust:tie:b", "候選乙")] {
        conn.execute(
            "INSERT INTO term_rules (source_locale, target_locale, source_text, target_text, domain, rule_type, priority, source_id, source_version_id, upstream_record_id, upstream_url, confidence, active) VALUES ('zh-Hant', 'zh-HK', '測試詞', ?1, 'test', 'test_rule', 999999, 'opencc', NULL, ?2, 'fixture://rust-test', 1.0, 1)",
            params![target, record],
        )
        .expect("insert tie rule");
    }
    drop(conn);

    let engine = LocalizerEngine::open(&path).expect("open engine");
    let result = engine
        .localize("測試詞", "zh-Hant", "zh-HK", None)
        .expect("localize");
    assert_eq!(result.output, "測試詞");
    assert!(result.review_needed);
}

#[test]
fn short_common_entity_surface_is_not_forced() {
    let (_temp, path) = copied_demo_db();
    let conn = Connection::open(&path).expect("open copy");
    let concept_id = conn
        .execute(
            "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES ('company', 'rust:apple-company', 'entity')",
            [],
        )
        .map(|_| conn.last_insert_rowid())
        .expect("insert concept");
    conn.execute(
        "INSERT INTO localized_names (concept_id, locale, text, name_type, domain, is_preferred, confidence) VALUES (?1, 'zh-CN', '苹果', 'preferred', 'entity', 1, 1.0)",
        params![concept_id],
    )
    .expect("insert source name");
    conn.execute(
        "INSERT INTO localized_names (concept_id, locale, text, name_type, domain, is_preferred, confidence) VALUES (?1, 'zh-TW', 'Apple 公司', 'preferred', 'entity', 1, 1.0)",
        params![concept_id],
    )
    .expect("insert target name");
    drop(conn);

    let engine = LocalizerEngine::open(&path).expect("open engine");
    let result = engine
        .localize("我今日食苹果。", "zh-CN", "zh-TW", None)
        .expect("localize");
    assert_eq!(result.output, "我今日食苹果。");
    assert!(!result.review_needed);
}
