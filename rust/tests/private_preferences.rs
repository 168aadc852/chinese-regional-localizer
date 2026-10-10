use chinese_regional_localizer::alternative_terms::{ChoiceIntent, ReviewSession};
use chinese_regional_localizer::context_profiles::{ContextProfile, ContextProfiles};
use chinese_regional_localizer::private_review::RememberError;
use chinese_regional_localizer::private_store::{Preference, PrivateStore};
use chinese_regional_localizer::{Runtime, RuntimeRequest};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
fn profiles() -> ContextProfiles {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../data/fixtures/context_selection_cases.json"
    ))
    .unwrap();
    let items: Vec<ContextProfile> = serde_json::from_value(fixture["profiles"].clone()).unwrap();
    ContextProfiles::with_custom_profiles(items).unwrap()
}
fn shared(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(include_str!("../../schema/sqlite-v0.2.sql"))
        .unwrap();
    conn.execute("INSERT INTO sources(source_id,name,status,pack,ingest_allowed,review_record,manifest_version) VALUES ('fixture','Synthetic','approved','core',0,'project-authored',1)",[]).unwrap();
    conn.execute("INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (1,'fixture',1,'2026-10-10')",[]).unwrap();
    for (source, target, a, b, priority) in [
        ("測試詞", "官方詞", "zh-Hant", "zh-HK", 10),
        ("測試詞", "另詞", "zh-Hant", "zh-HK", 1),
        ("測試詞", "台灣詞", "zh-Hant", "zh-TW", 10),
        ("测试词", "測試詞", "zh-CN", "zh-Hant", 0),
    ] {
        conn.execute("INSERT INTO term_rules(source_text,target_text,source_locale,target_locale,priority,rule_type,source_id,source_version_id) VALUES (?,?,?,?,?,'fixture','fixture',1)",params![source,target,a,b,priority]).unwrap();
    }
}
fn preference(source: &str, replacement: &str, usage: Option<&str>) -> Preference {
    Preference {
        source: source.into(),
        replacement: replacement.into(),
        target: "zh-HK".into(),
        usage: usage.map(str::to_owned),
        source_locale: None,
        note: None,
    }
}
fn request(text: &str, source: &str, target: &str, context: Option<&str>) -> RuntimeRequest {
    RuntimeRequest {
        api_version: "1".into(),
        text: text.into(),
        source_locale: source.into(),
        target_locale: target.into(),
        context: context.map(|id| json!({"usage_context_id":id})),
    }
}
fn run_case(case: &Value, reverse: bool) -> Value {
    let directory = tempdir().unwrap();
    let s = directory.path().join("shared.sqlite");
    let u = directory.path().join("user.sqlite");
    shared(&s);
    let mut store = PrivateStore::open(&u)
        .unwrap()
        .with_context_profiles(profiles());
    let mut terms = case["terms"].as_array().unwrap().clone();
    if reverse {
        terms.reverse();
    }
    let mut ids: Vec<_> = terms
        .iter()
        .map(|term| term[0].as_str().unwrap())
        .filter(|id| *id != "personal" && *id != "legacy")
        .collect();
    ids.sort();
    ids.dedup();
    if reverse {
        ids.reverse();
    }
    for id in ids {
        store.create_dictionary(id, id).unwrap();
    }
    for term in &terms {
        let preference = Preference {
            source: term[1].as_str().unwrap().into(),
            replacement: term[2].as_str().unwrap().into(),
            usage: term[3].as_str().map(str::to_owned),
            target: term[4].as_str().unwrap_or("zh-HK").into(),
            source_locale: term[5].as_str().map(str::to_owned),
            note: None,
        };
        let id = store
            .put_preference(term[0].as_str().unwrap(), &preference)
            .unwrap();
        if term[6] == false {
            store.set_term_enabled(id, false).unwrap();
        }
    }
    if let Some(ids) = case["disabled"].as_array() {
        for id in ids {
            store
                .set_dictionary_enabled(id.as_str().unwrap(), false)
                .unwrap();
        }
    }
    if let Some(terms) = case["legacy"].as_array() {
        for term in terms {
            store.connection().execute("INSERT INTO user_terms(kind,source_text,replacement,source_locale,target_locale,priority,enabled,created_at,updated_at) VALUES ('override',?,?,?,?,?,1,'old','old')",params![term[0].as_str(),term[1].as_str(),term[2].as_str(),term[3].as_str(),term[4].as_i64()]).unwrap();
        }
    }
    if case["protect"] == true {
        store.connection().execute("INSERT INTO user_terms(kind,source_text,created_at,updated_at) VALUES ('protected','測試詞','old','old')",[]).unwrap();
    }
    drop(store);
    let context = match case.get("context") {
        None => Some("technology-software"),
        Some(value) => value.as_str(),
    };
    let response = Runtime::new(&s, Some(&u))
        .with_context_profiles(profiles())
        .localize(&request(
            case["input"].as_str().unwrap_or("測試詞"),
            case["source"].as_str().unwrap_or("zh-Hant"),
            case["target"].as_str().unwrap_or("zh-HK"),
            context,
        ))
        .unwrap();
    let snapshot = ReviewSession::from_response(&response).unwrap().snapshot();
    assert_eq!(
        serde_json::to_value(&response)
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        9
    );
    json!({"output":response.output,"review":response.review_needed,"snapshot":snapshot})
}
fn persistence_trace(intent: ChoiceIntent, fail: bool) -> Value {
    let directory = tempdir().unwrap();
    let s = directory.path().join("shared.sqlite");
    let u = directory.path().join("user.sqlite");
    shared(&s);
    let mut store = PrivateStore::open(&u)
        .unwrap()
        .with_context_profiles(profiles());
    let runtime = Runtime::new(&s, Some(&u)).with_context_profiles(profiles());
    let req = request(
        "😀测试词测试词",
        "zh-CN",
        "zh-HK",
        Some("technology-software"),
    );
    let mut session = runtime.review(&req).unwrap();
    let first = session.snapshot().occurrences[0].clone();
    let candidate = first
        .candidates
        .iter()
        .find(|item| item.target_text == "另詞")
        .unwrap();
    if fail {
        store.connection().execute_batch("CREATE TRIGGER fail_write BEFORE INSERT ON user_terms BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    }
    let error = session
        .apply_choice(
            0,
            &first.occurrence_id,
            &candidate.candidate_id,
            intent,
            Some(&mut store),
        )
        .err();
    assert_eq!(error.is_some(), fail);
    if fail {
        assert!(matches!(error, Some(RememberError::Persistence(_))));
    }
    let snapshot = session.snapshot();
    let undo = session.undo(snapshot.revision).unwrap();
    drop(store);
    let reopened = PrivateStore::open(&u).unwrap();
    let rows:Vec<Value>=reopened.connection().prepare("SELECT dictionary_id,source_text,replacement,source_locale,target_locale,usage_context_id,legacy FROM user_terms ORDER BY user_term_id").unwrap().query_map([],|row|Ok(json!([row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,Option<String>>(3)?,row.get::<_,String>(4)?,row.get::<_,Option<String>>(5)?,row.get::<_,i64>(6)?]))).unwrap().collect::<Result<_,_>>().unwrap();
    drop(reopened);
    let output = runtime
        .localize(&request(
            "测试词",
            "zh-CN",
            "zh-HK",
            Some("technology-software"),
        ))
        .unwrap()
        .output;
    json!({"snapshot":snapshot,"undo":undo,"rows":rows,"error":fail,"reopened_output":output})
}

#[test]
fn golden_selection_repeated_both_dictionary_and_term_orders() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../data/fixtures/private_preferences_cases.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let result = run_case(case, false);
        assert_eq!(result["output"], case["expected"], "{}", case["id"]);
        assert_eq!(
            result["review"],
            case["review"].as_bool().unwrap_or(false),
            "{}",
            case["id"]
        );
        assert_eq!(result, run_case(case, true), "{}", case["id"]);
    }
}

#[test]
fn live_python_rust_selection_persistence_and_failure_parity() {
    let output =
        Command::new(std::env::var_os("CRL_TEST_PYTHON").unwrap_or_else(|| "python".into()))
            .arg(root().join("tests/test_private_preferences.py"))
            .arg("--emit-parity")
            .env("PYTHONIOENCODING", "utf-8")
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let python: Value = serde_json::from_slice(&output.stdout).unwrap();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../data/fixtures/private_preferences_cases.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        assert_eq!(
            run_case(case, false),
            python["cases"][case["id"].as_str().unwrap()],
            "{}",
            case["id"]
        );
    }
    for (intent, key) in [
        (ChoiceIntent::UseThisTimeOnly, "use_this_time_only"),
        (
            ChoiceIntent::RememberForThisContext,
            "remember_for_this_context",
        ),
        (
            ChoiceIntent::RememberForAllContexts,
            "remember_for_all_contexts",
        ),
    ] {
        assert_eq!(persistence_trace(intent, false), python["persistence"][key]);
    }
    assert_eq!(
        persistence_trace(ChoiceIntent::RememberForThisContext, true),
        python["failed_write"]
    );
    assert_eq!(csv_trace(), python["csv"]);
}

fn csv_trace() -> Value {
    let inputs=[
        ("source_text,replacement\n詞,甲\n詞,乙\n",Some("zh-HK")),
        ("source_text,replacement\n詞,答案\n",Some("zh-HK")),
        ("source_text,replacement,target_locale,note\r\n\"詞,一\",\"替代\n文\",zh-HK,\"引號\"\"備註\"\r\n",None),
        ("source_text,replacement\n有效,答案\n,錯誤\n",Some("zh-HK")),
        ("source_text,replacement\n詞,答案\n",None),
        ("source_text,replacement,target_locale,usage_context_id\n詞,答案,zh-HK,disabled\n",None),
        ("source_text,replacement\n有效,答案\n\"壞詞,答案",Some("zh-HK")),
        ("source_text,source_text,replacement\n詞,詞,答案\n",Some("zh-HK")),
        ("source_text,replacement\n詞\n",Some("zh-HK")),
    ];
    let store = PrivateStore::from_connection(Connection::open_in_memory().unwrap())
        .unwrap()
        .with_context_profiles(profiles());
    json!(inputs.into_iter().map(|(text,target)| {
        let preview=store.preview_csv(text,target);
        json!({"rows":preview.rows.iter().map(|row|json!([row.source,row.replacement,row.target,row.usage,row.note])).collect::<Vec<_>>(),"errors":preview.errors.iter().map(|error|error.row).collect::<Vec<_>>()})
    }).collect::<Vec<_>>())
}

#[test]
fn unversioned_migration_and_unknown_extensions_fail_safe() {
    let conn = Connection::open_in_memory().unwrap();
    legacy(&conn);
    let before = legacy_rows(&conn);
    conn.execute_batch("DROP TABLE user_metadata;").unwrap();
    let store = PrivateStore::from_connection(conn).unwrap();
    assert_eq!(legacy_rows(store.connection()), before);
    for extension in [
        "CREATE TABLE extra(value TEXT);",
        "CREATE VIEW extra AS SELECT * FROM user_terms;",
    ] {
        let directory = tempdir().unwrap();
        let path = directory.path().join("extended.sqlite");
        let conn = Connection::open(&path).unwrap();
        legacy(&conn);
        conn.execute_batch(extension).unwrap();
        drop(conn);
        assert!(PrivateStore::open(&path).is_err());
        let conn = Connection::open(&path).unwrap();
        let version: String = conn
            .query_row(
                "SELECT value FROM user_metadata WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "0.1");
    }
}

#[test]
fn remembered_personal_idempotence_update_other_dictionary_and_target_isolation() {
    let directory = tempdir().unwrap();
    let s = directory.path().join("s.sqlite");
    let u = directory.path().join("u.sqlite");
    shared(&s);
    let mut store = PrivateStore::open(&u).unwrap();
    store.create_dictionary("company", "Company").unwrap();
    let term = preference("測試詞", "公司詞", None);
    let company = store.put_preference("company", &term).unwrap();
    let term = preference("測試詞", "一", None);
    let first = store.put_preference("personal", &term).unwrap();
    assert_eq!(first, store.put_preference("personal", &term).unwrap());
    let term = preference("測試詞", "二", None);
    assert_eq!(first, store.put_preference("personal", &term).unwrap());
    let company_text: String = store
        .connection()
        .query_row(
            "SELECT replacement FROM user_terms WHERE user_term_id=?",
            [company],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(company_text, "公司詞");
    drop(store);
    assert_eq!(
        Runtime::new(&s, Some(&u))
            .localize(&request("測試詞", "zh-Hant", "zh-TW", None))
            .unwrap()
            .output,
        "台灣詞"
    );
}

fn legacy(conn: &Connection) {
    conn.execute_batch(include_str!("../../schema/user-dictionary-v0.1.sql"))
        .unwrap();
    conn.execute_batch("INSERT INTO user_terms(user_term_id,kind,source_text,replacement,source_locale,target_locale,priority,enabled,note,created_at,updated_at) VALUES (42,'protected','OpenAI',NULL,NULL,NULL,77,1,'保留','old','old'),(99,'override','測試詞','舊詞','zh-Hant',NULL,123,0,'note','old','old');").unwrap();
}
fn legacy_rows(conn: &Connection) -> Vec<Value> {
    conn.prepare("SELECT user_term_id,kind,source_text,replacement,source_locale,target_locale,priority,enabled,note,created_at,updated_at FROM user_terms ORDER BY user_term_id").unwrap().query_map([],|r|Ok(json!([r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,i64>(6)?,r.get::<_,i64>(7)?,r.get::<_,Option<String>>(8)?,r.get::<_,String>(9)?,r.get::<_,String>(10)?]))).unwrap().collect::<Result<_,_>>().unwrap()
}

#[test]
fn migration_retains_ids_all_fields_null_scope_disabled_and_reopens_idempotently() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("legacy.sqlite");
    let conn = Connection::open(&path).unwrap();
    legacy(&conn);
    let before = legacy_rows(&conn);
    drop(conn);
    let store = PrivateStore::open(&path).unwrap();
    assert_eq!(legacy_rows(store.connection()), before);
    let count:i64=store.connection().query_row("SELECT count(*) FROM user_terms WHERE dictionary_id='legacy' AND usage_context_id IS NULL AND legacy=1",[],|r|r.get(0)).unwrap();
    assert_eq!(count, 2);
    drop(store);
    let reopened = PrivateStore::open(&path).unwrap();
    assert_eq!(legacy_rows(reopened.connection()), before);
}

#[test]
fn injected_migration_failure_rolls_back_to_usable_old_schema() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("legacy.sqlite");
    let conn = Connection::open(&path).unwrap();
    legacy(&conn);
    let before = legacy_rows(&conn);
    conn.execute_batch("CREATE TRIGGER fail_migration BEFORE UPDATE ON user_metadata BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    drop(conn);
    assert!(PrivateStore::open(&path).is_err());
    let conn = Connection::open(&path).unwrap();
    assert_eq!(legacy_rows(&conn), before);
    let version: String = conn
        .query_row(
            "SELECT value FROM user_metadata WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(version, "0.1");
    assert!(conn
        .prepare("SELECT dictionary_id FROM user_terms")
        .is_err());
    let shared_conn = Connection::open_in_memory().unwrap();
    shared_conn
        .execute_batch(include_str!("../../schema/sqlite-v0.2.sql"))
        .unwrap();
    let localizer =
        chinese_regional_localizer::UserControlledLocalizer::from_connections(shared_conn, conn);
    assert_eq!(
        localizer
            .localize("OpenAI", "zh-Hant", "zh-HK", None)
            .unwrap()
            .output,
        "OpenAI"
    );
    drop(localizer);
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("DROP TRIGGER fail_migration;").unwrap();
    drop(conn);
    assert!(PrivateStore::open(&path).is_ok());
}

#[test]
fn future_and_corrupt_store_rejected_untouched() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("future.sqlite");
    let conn = Connection::open(&path).unwrap();
    legacy(&conn);
    conn.execute("UPDATE user_metadata SET value='999'", [])
        .unwrap();
    drop(conn);
    assert!(PrivateStore::open(&path).is_err());
    let conn = Connection::open(&path).unwrap();
    let value: String = conn
        .query_row("SELECT value FROM user_metadata", [], |r| r.get(0))
        .unwrap();
    assert_eq!(value, "999");
    drop(conn);
    let corrupt = directory.path().join("corrupt.sqlite");
    std::fs::write(&corrupt, b"not SQLite").unwrap();
    assert!(PrivateStore::open(&corrupt).is_err());
    assert_eq!(std::fs::read(corrupt).unwrap(), b"not SQLite");
}

#[test]
fn csv_dictionary_operations_validation_and_atomic_commit() {
    let mut store = PrivateStore::from_connection(Connection::open_in_memory().unwrap()).unwrap();
    store.create_dictionary("a", "Audio").unwrap();
    store.rename_dictionary("a", "Hi-Fi Audio").unwrap();
    for id in ["personal", "legacy"] {
        assert!(store.rename_dictionary(id, "Wrong").is_err());
        assert!(store.delete_dictionary(id).is_err());
    }
    let mut preview=store.preview_csv("source_text,replacement,target_locale,note\r\n\"詞,一\",\"替代\n文\",zh-HK,\"引號\"\"備註\"\r\n",None);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    store.commit_csv("a", &preview).unwrap();
    let invalid = store.preview_csv("source_text,replacement\n有效,答案\n,錯誤\n", Some("zh-HK"));
    assert_eq!(invalid.errors[0].row, 3);
    assert!(store.commit_csv("a", &invalid).is_err());
    let mut bad = preference("bad", "X", None);
    bad.target.clear();
    preview.rows.push(bad);
    assert!(store.commit_csv("a", &preview).is_err());
    let count: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM user_terms", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let id: i64 = store
        .connection()
        .query_row("SELECT user_term_id FROM user_terms", [], |r| r.get(0))
        .unwrap();
    store.set_term_enabled(id, false).unwrap();
    store
        .edit_preference(id, &preference("新詞", "新答案", Some("general")))
        .unwrap();
    store.delete_term(id).unwrap();
    store.delete_dictionary("a").unwrap();
}

#[test]
fn stale_foreign_candidate_and_missing_context_never_write() {
    let directory = tempdir().unwrap();
    let s = directory.path().join("s.sqlite");
    let u = directory.path().join("u.sqlite");
    shared(&s);
    let mut store = PrivateStore::open(&u).unwrap();
    let runtime = Runtime::new(&s, Some(&u));
    let mut session = runtime
        .review(&request(
            "測試詞測試詞",
            "zh-Hant",
            "zh-HK",
            Some("general"),
        ))
        .unwrap();
    let before = session.snapshot();
    let first = &before.occurrences[0];
    let second = &before.occurrences[1];
    for (revision, candidate) in [
        (1, first.candidates[0].candidate_id.as_str()),
        (0, second.candidates[0].candidate_id.as_str()),
        (0, "foreign"),
    ] {
        assert!(matches!(
            session.apply_choice(
                revision,
                &first.occurrence_id,
                candidate,
                ChoiceIntent::RememberForAllContexts,
                Some(&mut store)
            ),
            Err(RememberError::Choice(_))
        ));
        assert_eq!(session.snapshot(), before);
    }
    let mut missing = runtime
        .review(&request("測試詞", "zh-Hant", "zh-HK", None))
        .unwrap();
    assert!(missing
        .apply_choice(
            0,
            "occurrence-1",
            "occurrence-1/candidate-1",
            ChoiceIntent::RememberForThisContext,
            Some(&mut store)
        )
        .is_err());
    let count: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM user_terms", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn partial_script_expansion_keeps_one_time_only_without_weakening_spans() {
    let directory = tempdir().unwrap();
    let s = directory.path().join("s.sqlite");
    let u = directory.path().join("u.sqlite");
    shared(&s);
    let mut store = PrivateStore::open(&u).unwrap();
    let conn = Connection::open(&s).unwrap();
    conn.execute("INSERT INTO term_rules(source_text,target_text,source_locale,target_locale,priority,rule_type,source_id,source_version_id) VALUES ('甲','測試詞測試詞','zh-CN','zh-Hant',0,'fixture','fixture',1)",[]).unwrap();
    drop(conn);
    let mut session = Runtime::new(&s, Some(&u))
        .review(&request("甲", "zh-CN", "zh-HK", Some("general")))
        .unwrap();
    let before = session.snapshot();
    let first = &before.occurrences[0];
    assert!(!first.rememberable);
    assert!(matches!(
        session.apply_choice(
            0,
            &first.occurrence_id,
            &first.candidates[1].candidate_id,
            ChoiceIntent::RememberForAllContexts,
            Some(&mut store)
        ),
        Err(RememberError::Choice(_))
    ));
    assert_eq!(session.snapshot(), before);
    assert_eq!(
        session
            .apply_choice(
                0,
                &first.occurrence_id,
                &first.candidates[1].candidate_id,
                ChoiceIntent::UseThisTimeOnly,
                None
            )
            .unwrap()
            .text,
        "另詞官方詞"
    );
    let count: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM user_terms", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
