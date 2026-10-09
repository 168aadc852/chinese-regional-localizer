use chinese_regional_localizer::context_profiles::{ContextProfile, ContextProfiles};
use chinese_regional_localizer::{
    LocalizerEngine, LocalizerError, Runtime, RuntimeRequest, UserControlledLocalizer,
};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../data/fixtures/context_selection_cases.json"
    ))
    .unwrap()
}

fn profiles(fixture: &Value) -> ContextProfiles {
    let custom: Vec<ContextProfile> = serde_json::from_value(fixture["profiles"].clone()).unwrap();
    ContextProfiles::with_custom_profiles(custom).unwrap()
}

fn database(fixture: &Value, case: &Value, reverse: bool) -> (TempDir, PathBuf, PathBuf) {
    let temp = tempdir().unwrap();
    let shared = temp.path().join("shared.sqlite");
    let user = temp.path().join("user.sqlite");
    let conn = Connection::open(&shared).unwrap();
    conn.execute_batch(include_str!("../../schema/sqlite-v0.2.sql"))
        .unwrap();
    conn.execute("INSERT INTO sources(source_id,name,status,pack,ingest_allowed,review_record,manifest_version) VALUES ('fixture','Synthetic fixture','approved','core',0,'project-authored test only',1)", []).unwrap();
    conn.execute("INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (1,'fixture',1,'2026-10-09'),(2,'fixture',0,'2026-10-09')", []).unwrap();
    let rules = fixture["rules"].as_object().unwrap();
    let mut keys: Vec<_> = case["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|key| key.as_str().unwrap())
        .collect();
    if reverse {
        keys.reverse();
    }
    for key in keys {
        let rule = &rules[key];
        let id = rules.keys().position(|name| name == key).unwrap() as i64 + 1;
        insert_rule(&conn, id, key, rule);
    }
    if case["entity"] == true {
        for id in 1..=if case["ambiguous_entity"] == true {
            2
        } else {
            1
        } {
            conn.execute("INSERT INTO concepts(concept_id,concept_type,domain) VALUES (?1,'person','entity')", [id]).unwrap();
            conn.execute("INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (?1,'zh-Hant','測試人物','preferred','entity',1,1),(?1,'zh-HK','人物名稱','preferred','entity',1,1)", [id]).unwrap();
        }
        if case["ambiguous_target"] == true {
            conn.execute("INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (1,'zh-HK','另一名稱','preferred','entity',1,1)", []).unwrap();
        }
    }
    drop(conn);
    if let Some(kind) = case["user_kind"].as_str() {
        let conn = Connection::open(&user).unwrap();
        conn.execute_batch(include_str!("../../schema/user-dictionary-v0.1.sql"))
            .unwrap();
        conn.execute("INSERT INTO user_terms(kind,source_text,replacement,created_at,updated_at) VALUES (?1,'測試詞',?2,'2026-10-09','2026-10-09')", params![kind, (kind == "override").then_some("個人詞")]).unwrap();
    }
    (temp, shared, user)
}

fn insert_rule(conn: &Connection, id: i64, key: &str, rule: &Value) {
    let constraint = if let Some(raw) = rule.get("raw_constraint") {
        Some(raw.as_str().unwrap().to_owned())
    } else {
        rule.get("constraint").map(Value::to_string)
    };
    conn.execute("INSERT INTO term_rules(rule_id,source_text,target_text,source_locale,target_locale,priority,context_constraint,rule_type,source_id,source_version_id,upstream_record_id,active) VALUES (?1,?2,?3,?4,?5,?6,?7,'fixture','fixture',?8,?9,?10)", params![
        id, rule["source"].as_str().unwrap(), rule["target"].as_str().unwrap(), rule["source_locale"].as_str().unwrap_or("zh-Hant"), rule["target_locale"].as_str().unwrap_or("zh-HK"), rule["priority"].as_i64().unwrap_or(0), constraint,
        if rule["stale"] == true { 2 } else { 1 }, key, rule["active"].as_bool().unwrap_or(true)
    ]).unwrap();
}

fn request(case: &Value) -> RuntimeRequest {
    RuntimeRequest {
        api_version: "1".into(),
        text: case["input"].as_str().unwrap_or("測試詞").into(),
        source_locale: case["source_locale"].as_str().unwrap_or("zh-Hant").into(),
        target_locale: case["target_locale"].as_str().unwrap_or("zh-HK").into(),
        context: case.get("context").cloned(),
    }
}

fn observable(fixture: &Value, case: &Value, reverse: bool) -> Value {
    let (_temp, shared, user) = database(fixture, case, reverse);
    let user_path = case.get("user_kind").map(|_| user.as_path());
    let runtime = Runtime::new(&shared, user_path).with_context_profiles(profiles(fixture));
    let request = request(case);
    let response = runtime.localize(&request);
    if case["error"] == true {
        assert!(
            matches!(response, Err(LocalizerError::UsageContext(_))),
            "{}",
            case["id"]
        );
        // Direct shared/user entry points must not hide invalid requests either.
        if user_path.is_some() {
            assert!(matches!(
                UserControlledLocalizer::open(&shared, &user)
                    .unwrap()
                    .with_context_profiles(profiles(fixture))
                    .localize(
                        &request.text,
                        &request.source_locale,
                        &request.target_locale,
                        request.context.as_ref()
                    ),
                Err(LocalizerError::UsageContext(_))
            ));
        }
        return json!({"error": true});
    }
    let response = response.unwrap();
    for _ in 0..2 {
        assert_eq!(
            serde_json::to_vec(&response).unwrap(),
            serde_json::to_vec(&runtime.localize(&request).unwrap()).unwrap(),
            "repeat {}",
            case["id"]
        );
    }
    if user_path.is_none() {
        let engine = LocalizerEngine::open(&shared)
            .unwrap()
            .with_context_profiles(profiles(fixture));
        let core = engine
            .localize(
                &request.text,
                &request.source_locale,
                &request.target_locale,
                request.context.as_ref(),
            )
            .unwrap();
        assert_eq!(core.output, response.output);
        assert_eq!(core.review_needed, response.review_needed);
    }
    if let Some(winner) = case["winner"].as_str() {
        let event = response
            .changes
            .iter()
            .find(|event| event.get("context_selection").is_some())
            .unwrap();
        assert_eq!(
            event["upstream_record_id"], winner,
            "actual selected provenance"
        );
    }
    if case["review"] == true && case.get("level").is_some() {
        assert!(
            response
                .changes
                .iter()
                .filter(|event| event.get("context_selection").is_some())
                .all(|event| event.get("rule_id").is_none()),
            "conflict must not invent a chosen rule"
        );
    }
    let selections: Vec<Value> = response
        .changes
        .iter()
        .filter_map(|event| event.get("context_selection").cloned())
        .collect();
    json!({"output":response.output,"review_needed":response.review_needed,"context_selections":selections})
}

#[test]
fn shared_context_contract_cases_and_sqlite_order_are_deterministic() {
    let fixture = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let actual = observable(&fixture, case, false);
        if case["error"] != true {
            assert_eq!(actual["output"], case["expected"], "{}", case["id"]);
            assert_eq!(
                actual["review_needed"],
                case["review"].as_bool().unwrap_or(false),
                "{}",
                case["id"]
            );
            if case.get("level").is_some() {
                let expected = json!({"usage_context_id":case["context"]["usage_context_id"],"matched_usage_context_id":case["matched"],"context_distance":case["distance"],"context_level":case["level"]});
                let selections = actual["context_selections"].as_array().unwrap();
                assert!(!selections.is_empty(), "{}", case["id"]);
                assert!(
                    selections.iter().all(|value| value == &expected),
                    "{}",
                    case["id"]
                );
            }
        }
        assert_eq!(
            observable(&fixture, case, true),
            actual,
            "reverse {}",
            case["id"]
        );
    }
}

#[test]
fn live_python_reference_and_rust_runtime_have_context_selection_parity() {
    let python = Command::new(std::env::var("CRL_TEST_PYTHON").unwrap_or_else(|_| "python".into()))
        .arg(root().join("tests/test_context_selection.py"))
        .arg("--emit-parity")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .expect("installed Python 3.11 is required by repository CI");
    assert!(
        python.status.success(),
        "{}",
        String::from_utf8_lossy(&python.stderr)
    );
    let expected: BTreeMap<String, Value> = serde_json::from_slice(&python.stdout).unwrap();
    let fixture = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert_eq!(
            observable(&fixture, case, false),
            expected[id],
            "Python/Rust parity: {id}"
        );
    }
}

#[test]
fn malformed_context_rule_constraints_fail_closed() {
    let fixture = fixture();
    let case = json!({"rules":[]});
    let mut invalid = vec![
        Value::Null,
        json!([]),
        json!("technology-software"),
        json!(5),
        json!({"constraints":null}),
        json!({"usage_context_id":"technology-software"}),
    ];
    for value in [
        Value::Null,
        json!([]),
        json!({}),
        json!(true),
        json!(3),
        json!(""),
        json!("*"),
        json!("unknown"),
        json!(["technology-software"]),
    ] {
        invalid.push(json!({"constraints":{"usage_context_id":value}}));
    }
    for metadata in invalid {
        let (_temp, shared, _) = database(&fixture, &case, false);
        let conn = Connection::open(&shared).unwrap();
        insert_rule(
            &conn,
            10000,
            "bad",
            &json!({"source":"測試詞","target":"不應使用","constraint":metadata}),
        );
        drop(conn);
        let result = LocalizerEngine::open(&shared)
            .unwrap()
            .localize(
                "測試詞",
                "zh-Hant",
                "zh-HK",
                Some(&json!({"usage_context_id":"technology-software"})),
            )
            .unwrap();
        assert_eq!(result.output, "測試詞");
    }
    for raw in ["{", "", "{\"constraints\":{\"usage_context_id\":null}}"] {
        let (_temp, shared, _) = database(&fixture, &case, false);
        let conn = Connection::open(&shared).unwrap();
        insert_rule(
            &conn,
            10000,
            "bad",
            &json!({"source":"測試詞","target":"不應使用","raw_constraint":raw}),
        );
        drop(conn);
        assert_eq!(
            LocalizerEngine::open(&shared)
                .unwrap()
                .localize(
                    "測試詞",
                    "zh-Hant",
                    "zh-HK",
                    Some(&json!({"usage_context_id":"technology-software"}))
                )
                .unwrap()
                .output,
            "測試詞"
        );
    }
}

#[test]
fn runtime_v1_outer_shape_and_legacy_records_are_unchanged() {
    let fixture = fixture();
    let case = &fixture["cases"][0];
    let (_temp, shared, _) = database(&fixture, case, false);
    let runtime = Runtime::new(&shared, Option::<&Path>::None);
    let response = serde_json::to_value(runtime.localize(&request(case)).unwrap()).unwrap();
    let mut fields: Vec<_> = response
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    fields.sort();
    assert_eq!(
        fields,
        [
            "api_version",
            "changes",
            "input",
            "output",
            "review_needed",
            "route",
            "source_locale",
            "target_locale",
            "user_dictionary_applied"
        ]
    );
    assert_eq!(response["api_version"], "1");
    assert!(response["changes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|event| event.get("context_selection").is_none()));
    assert_eq!(
        runtime
            .localize(&RuntimeRequest {
                context: Some(json!({})),
                ..request(case)
            })
            .unwrap()
            .output,
        "中性詞"
    );
}
