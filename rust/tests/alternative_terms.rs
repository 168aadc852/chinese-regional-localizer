use chinese_regional_localizer::alternative_terms::{
    ChoiceError, ChoiceIntent, ReviewSession, ReviewSnapshot,
};
use chinese_regional_localizer::context_profiles::{ContextProfile, ContextProfiles};
use chinese_regional_localizer::{LocalizerError, Runtime, RuntimeRequest, RuntimeResponse};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
fn fixtures() -> (Value, Value) {
    let base: Value = serde_json::from_str(include_str!(
        "../../data/fixtures/context_selection_cases.json"
    ))
    .unwrap();
    let extra: Value = serde_json::from_str(include_str!(
        "../../data/fixtures/alternative_terms_cases.json"
    ))
    .unwrap();
    (base, extra)
}
fn cases(base: &Value, extra: &Value) -> Vec<Value> {
    base["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(extra["cases"].as_array().unwrap())
        .cloned()
        .collect()
}
fn run_case(
    base: &Value,
    extra: &Value,
    case: &Value,
    reverse: bool,
) -> Result<RuntimeResponse, LocalizerError> {
    let temp = tempdir().unwrap();
    let shared = temp.path().join("shared.sqlite");
    let user = temp.path().join("user.sqlite");
    let conn = Connection::open(&shared).unwrap();
    conn.execute_batch(include_str!("../../schema/sqlite-v0.2.sql"))
        .unwrap();
    conn.execute("INSERT INTO sources(source_id,name,status,pack,ingest_allowed,review_record,manifest_version) VALUES ('fixture','Synthetic fixture','approved','core',0,'project-authored test only',1)", []).unwrap();
    conn.execute("INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (1,'fixture',1,'2026-10-10'),(2,'fixture',0,'2026-10-10')", []).unwrap();
    let mut rules = base["rules"].as_object().unwrap().clone();
    rules.extend(extra["rules"].as_object().unwrap().clone());
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
        let constraint = rule
            .get("raw_constraint")
            .map(|value| value.as_str().unwrap().to_owned())
            .or_else(|| rule.get("constraint").map(Value::to_string));
        conn.execute("INSERT INTO term_rules(rule_id,source_text,target_text,source_locale,target_locale,priority,context_constraint,rule_type,source_id,source_version_id,upstream_record_id,active) VALUES (?1,?2,?3,?4,?5,?6,?7,'fixture','fixture',?8,?9,?10)",params![id,rule["source"].as_str().unwrap(),rule["target"].as_str().unwrap(),rule["source_locale"].as_str().unwrap_or("zh-Hant"),rule["target_locale"].as_str().unwrap_or("zh-HK"),rule["priority"].as_i64().unwrap_or(0),constraint,if rule["stale"] == true {2} else {1},key,rule["active"].as_bool().unwrap_or(true)]).unwrap();
    }
    if case["entity"] == true {
        for id in 1..=if case["ambiguous_entity"] == true {
            2
        } else {
            1
        } {
            conn.execute("INSERT INTO concepts(concept_id,concept_type,domain) VALUES (?1,'person','entity')",[id]).unwrap();
            conn.execute("INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (?1,'zh-Hant','測試人物','preferred','entity',1,1),(?1,'zh-HK',?2,'preferred','entity',1,1)",params![id,case["entity_target"].as_str().unwrap_or("人物名稱")]).unwrap();
        }
        if case["ambiguous_target"] == true {
            conn.execute("INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (1,'zh-HK','另一名稱','preferred','entity',1,1)",[]).unwrap();
        }
    }
    drop(conn);
    let user_path = case.get("user_kind").map(|_| user.as_path());
    if let Some(kind) = case["user_kind"].as_str() {
        let conn = Connection::open(&user).unwrap();
        conn.execute_batch(include_str!("../../schema/user-dictionary-v0.1.sql"))
            .unwrap();
        conn.execute("INSERT INTO user_terms(kind,source_text,replacement,created_at,updated_at) VALUES (?1,'測試詞',?2,'2026-10-10','2026-10-10')",params![kind,(kind == "override").then_some(case["override_target"].as_str().unwrap_or("個人詞"))]).unwrap();
    }
    let profiles: Vec<ContextProfile> = serde_json::from_value(base["profiles"].clone()).unwrap();
    Runtime::new(&shared, user_path)
        .with_context_profiles(ContextProfiles::with_custom_profiles(profiles).unwrap())
        .localize(&RuntimeRequest {
            api_version: "1".into(),
            text: case["input"].as_str().unwrap_or("測試詞").into(),
            source_locale: case["source_locale"].as_str().unwrap_or("zh-Hant").into(),
            target_locale: case["target_locale"].as_str().unwrap_or("zh-HK").into(),
            context: case.get("context").cloned(),
        })
    // TempDir and every database handle are dropped before any choice/edit operation.
}
fn choice_values(response: &RuntimeResponse) -> Vec<Value> {
    response
        .changes
        .iter()
        .filter_map(|event| event.get("choice").cloned())
        .collect()
}
fn trace(case: &Value, response: &RuntimeResponse) -> Vec<ReviewSnapshot> {
    let mut session = ReviewSession::from_response(response).unwrap();
    let mut trace = vec![session.snapshot()];
    if let Some(edits) = case["edit_script"].as_array() {
        for edit in edits {
            let current = session.snapshot();
            let next = if edit["undo"] == true {
                session.undo(current.revision).unwrap()
            } else {
                let item = &current.occurrences[edit["occurrence"].as_u64().unwrap() as usize];
                let candidate = item
                    .candidates
                    .iter()
                    .find(|candidate| candidate.target_text == edit["target"].as_str().unwrap())
                    .unwrap();
                session
                    .apply_choice(
                        current.revision,
                        &item.occurrence_id,
                        &candidate.candidate_id,
                        ChoiceIntent::UseThisTimeOnly,
                    )
                    .unwrap()
            };
            trace.push(next);
        }
    }
    trace
}
fn observable(base: &Value, extra: &Value, case: &Value, reverse: bool) -> Value {
    let response = run_case(base, extra, case, reverse);
    if case["error"] == true {
        assert!(matches!(response, Err(LocalizerError::UsageContext(_))));
        return json!({"error":true});
    }
    let response = response.unwrap();
    json!({"output":response.output,"review_needed":response.review_needed,"choices":choice_values(&response),"trace":trace(case,&response)})
}

#[test]
fn golden_candidates_unicode_spans_and_sqlite_order() {
    let (base, extra) = fixtures();
    for case in cases(&base, &extra) {
        let actual = observable(&base, &extra, &case, false);
        if case["error"] != true {
            assert_eq!(actual["output"], case["expected"], "{}", case["id"]);
            assert_eq!(
                actual["review_needed"],
                case["review"].as_bool().unwrap_or(false),
                "{}",
                case["id"]
            );
            let choices = actual["choices"].as_array().unwrap();
            if let Some(expected) = case.get("expected_candidates") {
                assert!(!choices.is_empty());
                for item in choices {
                    let candidates: Vec<Value> = item["candidates"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|candidate| json!([candidate["target_text"], candidate["status"]]))
                        .collect();
                    assert_eq!(json!(candidates), *expected, "{}", case["id"]);
                }
            }
            for (expected_key, span_key) in [
                ("expected_source_spans", "source_span"),
                ("expected_output_spans", "output_span"),
            ] {
                if let Some(expected) = case.get(expected_key) {
                    let spans: Vec<_> = choices.iter().map(|item| item[span_key].clone()).collect();
                    assert_eq!(json!(spans), *expected, "{}", case["id"]);
                }
            }
            for (index, item) in choices.iter().enumerate() {
                assert_eq!(item["occurrence_id"], format!("occurrence-{}", index + 1));
                assert!(item["candidates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|candidate| candidate.as_object().unwrap().len() == 3));
            }
        }
        assert_eq!(
            observable(&base, &extra, &case, true),
            actual,
            "reverse {}",
            case["id"]
        );
        assert_eq!(
            observable(&base, &extra, &case, false),
            actual,
            "repeat {}",
            case["id"]
        );
    }
}

#[test]
fn edits_and_lifo_undo_match_independent_splice_model() {
    let (base, extra) = fixtures();
    for case in extra["cases"].as_array().unwrap() {
        let Some(edits) = case["edit_script"].as_array() else {
            continue;
        };
        let response = run_case(&base, &extra, case, false).unwrap();
        let trace = trace(case, &response);
        let mut expected = trace[0].clone();
        let mut history = Vec::new();
        for (index, (edit, actual)) in edits.iter().zip(&trace[1..]).enumerate() {
            if edit["undo"] == true {
                expected = history.pop().unwrap();
            } else {
                history.push(expected.clone());
                let occurrence = edit["occurrence"].as_u64().unwrap() as usize;
                let item = &mut expected.occurrences[occurrence];
                let [start, end] = item.output_span;
                let target = edit["target"].as_str().unwrap();
                expected.text = expected.text.chars().take(start).collect::<String>()
                    + target
                    + &expected.text.chars().skip(end).collect::<String>();
                let length = target.chars().count();
                item.output_span = [start, start + length];
                item.expected_text = target.into();
                item.state = "chosen".into();
                item.selected_candidate_id = Some(
                    item.candidates
                        .iter()
                        .find(|candidate| candidate.target_text == target)
                        .unwrap()
                        .candidate_id
                        .clone(),
                );
                for later in &mut expected.occurrences[occurrence + 1..] {
                    later.output_span = later.output_span.map(|p| p + length - (end - start));
                }
            }
            expected.revision = index as u64 + 1;
            assert_eq!(*actual, expected, "{}", case["id"]);
        }
        assert_eq!(trace.last().unwrap().text, response.output);
        assert_eq!(trace.last().unwrap().occurrences, trace[0].occurrences);
    }
}

#[test]
fn invalid_choices_are_atomic_and_returned_snapshots_are_not_live() {
    let (base, extra) = fixtures();
    let case = extra["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "unicode-adjacent-repeated")
        .unwrap();
    let response = run_case(&base, &extra, case, false).unwrap();
    let mut session = ReviewSession::from_response(&response).unwrap();
    let initial = session.snapshot();
    let first = &initial.occurrences[0];
    let second = &initial.occurrences[1];
    for (revision, occurrence, candidate, intent, error) in [
        (
            99,
            first.occurrence_id.as_str(),
            first.candidates[0].candidate_id.as_str(),
            ChoiceIntent::UseThisTimeOnly,
            ChoiceError::StaleRevision,
        ),
        (
            0,
            "unknown",
            first.candidates[0].candidate_id.as_str(),
            ChoiceIntent::UseThisTimeOnly,
            ChoiceError::UnknownOccurrence,
        ),
        (
            0,
            first.occurrence_id.as_str(),
            "unknown",
            ChoiceIntent::UseThisTimeOnly,
            ChoiceError::InvalidCandidate,
        ),
        (
            0,
            first.occurrence_id.as_str(),
            second.candidates[0].candidate_id.as_str(),
            ChoiceIntent::UseThisTimeOnly,
            ChoiceError::InvalidCandidate,
        ),
        (
            0,
            first.occurrence_id.as_str(),
            first.candidates[0].candidate_id.as_str(),
            ChoiceIntent::RememberForThisContext,
            ChoiceError::UnsupportedIntent,
        ),
        (
            0,
            first.occurrence_id.as_str(),
            first.candidates[0].candidate_id.as_str(),
            ChoiceIntent::RememberForAllContexts,
            ChoiceError::UnsupportedIntent,
        ),
    ] {
        assert_eq!(
            session
                .apply_choice(revision, occurrence, candidate, intent)
                .unwrap_err(),
            error
        );
        assert_eq!(session.snapshot(), initial);
    }
    assert_eq!(session.undo(0).unwrap_err(), ChoiceError::NoUndo);
    let mut copied = session.snapshot();
    copied.occurrences[0].candidates[0].target_text = "tampered".into();
    assert_eq!(session.snapshot(), initial);
    session
        .apply_choice(
            0,
            &first.occurrence_id,
            &first.candidates[0].candidate_id,
            ChoiceIntent::UseThisTimeOnly,
        )
        .unwrap();
    let changed = session.snapshot();
    assert_eq!(session.undo(0).unwrap_err(), ChoiceError::StaleRevision);
    assert_eq!(session.snapshot(), changed);
}

#[test]
fn live_python_rust_parity_for_candidates_ids_spans_edit_and_undo() {
    let python = Command::new(std::env::var("CRL_TEST_PYTHON").unwrap_or_else(|_| "python".into()))
        .arg(root().join("tests/test_alternative_terms.py"))
        .arg("--emit-parity")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .expect("repository Python required");
    assert!(
        python.status.success(),
        "{}",
        String::from_utf8_lossy(&python.stderr)
    );
    let expected: BTreeMap<String, Value> = serde_json::from_slice(&python.stdout).unwrap();
    let (base, extra) = fixtures();
    for case in cases(&base, &extra) {
        assert_eq!(
            observable(&base, &extra, &case, false),
            expected[case["id"].as_str().unwrap()],
            "{}",
            case["id"]
        );
    }
}

#[test]
fn legacy_requests_keep_runtime_v1_outer_shape_and_optional_metadata() {
    let (base, extra) = fixtures();
    let response = run_case(&base, &extra, &base["cases"][0], false).unwrap();
    let value = serde_json::to_value(&response).unwrap();
    let mut keys: Vec<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    assert_eq!(
        keys,
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
    assert_eq!(response.api_version, "1");
    assert_eq!(response.output, "中性詞");
    assert!(response
        .changes
        .iter()
        .all(|event| event.get("context_selection").is_none()));
    let choice = &choice_values(&response)[0];
    assert_eq!(choice["candidates"][0]["status"], "recommended");
}
