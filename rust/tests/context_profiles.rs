use chinese_regional_localizer::context_profiles::{
    ContextProfile, ContextProfileError, ContextProfileStore, ContextProfiles,
    CONTEXT_PROFILE_FILE, CONTEXT_PROFILE_SCHEMA_VERSION, MAX_CONTEXT_PROFILES,
    MAX_CONTEXT_PROFILE_BYTES,
};
use serde_json::{json, Value};
use std::fs;
use tempfile::tempdir;

fn chain(profiles: &ContextProfiles, id: &str) -> Vec<String> {
    profiles
        .resolve_chain(id)
        .unwrap()
        .iter()
        .map(|profile| profile.context_id.clone())
        .collect()
}

fn document() -> Value {
    serde_json::from_slice(&ContextProfiles::default().to_json().unwrap()).unwrap()
}

fn definitions() -> Vec<ContextProfile> {
    ContextProfiles::default().profiles().cloned().collect()
}

#[test]
fn seven_built_ins_have_stable_unique_ids_and_names() {
    let profiles = ContextProfiles::default();
    let expected = [
        ("general", "General"),
        ("technology-software", "Technology / Software"),
        ("banking-finance", "Banking / Finance"),
        ("business-marketing", "Business / Marketing"),
        ("legal", "Legal"),
        ("education", "Education"),
        (
            "government-public-administration",
            "Government / Public Administration",
        ),
    ];
    assert_eq!(profiles.profiles().count(), expected.len());
    for (id, name) in expected {
        let profile = profiles.get(id).unwrap();
        assert_eq!(profile.display_name, name);
        assert!(profile.built_in && profile.enabled);
        assert_eq!(
            chain(&profiles, id),
            if id == "general" {
                vec![id]
            } else {
                vec![id, "general"]
            }
        );
    }
    assert_eq!(profiles.get("general").unwrap().parent_context_id, None);
}

#[test]
fn custom_defaults_to_general_and_can_explicitly_choose_another_parent_or_none() {
    let default = ContextProfile::custom("hi-fi-audio", "Hi-Fi Audio");
    assert_eq!(default.parent_context_id.as_deref(), Some("general"));
    let mut specific = ContextProfile::custom("esg", "ESG");
    specific.parent_context_id = Some("business-marketing".into());
    let mut standalone = ContextProfile::custom("standalone", "Standalone");
    standalone.parent_context_id = None;
    let profiles =
        ContextProfiles::with_custom_profiles(vec![default, specific, standalone]).unwrap();
    assert_eq!(chain(&profiles, "hi-fi-audio"), ["hi-fi-audio", "general"]);
    assert_eq!(
        chain(&profiles, "esg"),
        ["esg", "business-marketing", "general"]
    );
    assert_eq!(chain(&profiles, "standalone"), ["standalone"]);
}

#[test]
fn imported_custom_parent_defaults_only_when_omitted_not_missing_or_null() {
    let mut doc = document();
    let profile =
        json!({"context_id": "audio", "display_name": "Audio", "built_in": false, "enabled": true});
    doc["profiles"].as_array_mut().unwrap().push(profile);
    let profiles = ContextProfiles::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap();
    assert_eq!(chain(&profiles, "audio"), ["audio", "general"]);
    let last = doc["profiles"].as_array().unwrap().len() - 1;
    doc["profiles"][last]["parent_context_id"] = Value::Null;
    let profiles = ContextProfiles::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap();
    assert_eq!(chain(&profiles, "audio"), ["audio"]);
    doc["profiles"][last]["parent_context_id"] = json!("missing");
    assert!(matches!(
        ContextProfiles::from_json(&serde_json::to_vec(&doc).unwrap()),
        Err(ContextProfileError::UnknownContext(_))
    ));
}

#[test]
fn multi_level_single_parent_chain_and_storage_order_are_deterministic() {
    let parent = ContextProfile::custom("audio", "Audio");
    let mut child = ContextProfile::custom("hi-fi-audio", "Hi-Fi Audio");
    child.parent_context_id = Some("audio".into());
    let a = ContextProfiles::with_custom_profiles(vec![parent.clone(), child.clone()]).unwrap();
    let b = ContextProfiles::with_custom_profiles(vec![child, parent]).unwrap();
    for _ in 0..10 {
        assert_eq!(
            chain(&a, "hi-fi-audio"),
            ["hi-fi-audio", "audio", "general"]
        );
        assert_eq!(chain(&a, "hi-fi-audio"), chain(&b, "hi-fi-audio"));
        assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
    }
}

#[test]
fn duplicate_ids_are_rejected_without_publishing_an_edit() {
    let profile = ContextProfile::custom("audio", "Audio");
    let old = ContextProfiles::with_custom_profiles(vec![profile.clone()]).unwrap();
    assert!(matches!(
        ContextProfiles::with_custom_profiles(vec![profile.clone(), profile]),
        Err(ContextProfileError::Invalid(_))
    ));
    assert_eq!(old.profiles().count(), 8);
    assert_eq!(chain(&old, "audio"), ["audio", "general"]);
}

#[test]
fn self_parent_is_rejected() {
    let mut profile = ContextProfile::custom("audio", "Audio");
    profile.parent_context_id = Some("audio".into());
    assert!(matches!(
        ContextProfiles::with_custom_profiles(vec![profile]),
        Err(ContextProfileError::Invalid(_))
    ));
}

#[test]
fn indirect_cycles_are_rejected_even_when_disabled() {
    let mut a = ContextProfile::custom("a", "A");
    let mut b = ContextProfile::custom("b", "B");
    a.parent_context_id = Some("b".into());
    b.parent_context_id = Some("a".into());
    for enabled in [true, false] {
        a.enabled = enabled;
        assert!(matches!(
            ContextProfiles::with_custom_profiles(vec![b.clone(), a.clone()]),
            Err(ContextProfileError::Invalid(_))
        ));
    }
}

#[test]
fn missing_parents_are_rejected_not_defaulted_to_general() {
    let mut profile = ContextProfile::custom("audio", "Audio");
    profile.parent_context_id = Some("missing".into());
    assert!(
        matches!(ContextProfiles::with_custom_profiles(vec![profile]), Err(ContextProfileError::UnknownContext(id)) if id == "missing")
    );
    assert!(matches!(
        ContextProfiles::default().resolve_chain("missing"),
        Err(ContextProfileError::UnknownContext(_))
    ));
}

#[test]
fn disabled_profiles_are_retained_but_unavailable_including_as_ancestors() {
    let mut parent = ContextProfile::custom("audio", "Audio");
    parent.enabled = false;
    let mut child = ContextProfile::custom("hi-fi-audio", "Hi-Fi Audio");
    child.parent_context_id = Some("audio".into());
    let profiles = ContextProfiles::with_custom_profiles(vec![parent, child]).unwrap();
    assert!(
        matches!(profiles.resolve_chain("audio"), Err(ContextProfileError::DisabledContext(id)) if id == "audio")
    );
    assert!(
        matches!(profiles.resolve_chain("hi-fi-audio"), Err(ContextProfileError::DisabledContext(id)) if id == "audio")
    );
    assert_eq!(
        ContextProfiles::from_json(&profiles.to_json().unwrap()).unwrap(),
        profiles
    );
    assert_eq!(chain(&profiles, "general"), ["general"]);
    let mut profiles = definitions();
    profiles
        .iter_mut()
        .find(|profile| profile.context_id == "general")
        .unwrap()
        .enabled = false;
    let profiles = ContextProfiles::from_profiles(profiles).unwrap();
    assert!(
        matches!(profiles.resolve_chain("legal"), Err(ContextProfileError::DisabledContext(id)) if id == "general")
    );
}

#[test]
fn built_in_ids_flags_and_parent_links_are_protected() {
    assert!(
        ContextProfiles::with_custom_profiles(vec![ContextProfile::custom(
            "general",
            "Replacement"
        )])
        .is_err()
    );
    let mut impostor = ContextProfile::custom("audio", "Audio");
    impostor.built_in = true;
    assert!(ContextProfiles::with_custom_profiles(vec![impostor.clone()]).is_err());
    let mut profiles = definitions();
    profiles.push(impostor);
    assert!(ContextProfiles::from_profiles(profiles).is_err());
    for id in ["general", "legal"] {
        let mut profiles = definitions();
        let profile = profiles
            .iter_mut()
            .find(|profile| profile.context_id == id)
            .unwrap();
        profile.parent_context_id = if id == "general" {
            Some("legal".into())
        } else {
            None
        };
        assert!(ContextProfiles::from_profiles(profiles).is_err());
    }
    let profiles: Vec<_> = definitions()
        .into_iter()
        .filter(|profile| profile.context_id != "general")
        .collect();
    assert!(ContextProfiles::from_profiles(profiles).is_err());
}

#[test]
fn display_name_changes_preserve_identity_and_resolution() {
    let mut profiles = definitions();
    profiles
        .iter_mut()
        .find(|profile| profile.context_id == "legal")
        .unwrap()
        .display_name = "法律".into();
    let profiles = ContextProfiles::from_profiles(profiles).unwrap();
    assert_eq!(profiles.get("legal").unwrap().display_name, "法律");
    assert_eq!(chain(&profiles, "legal"), ["legal", "general"]);
}

#[test]
fn invalid_ids_names_and_parent_ids_fail_validation() {
    for id in [
        "",
        "General",
        "a b",
        "a/b",
        "-audio",
        "audio-",
        "a--b",
        "中文",
        "1audio",
        &"a".repeat(65),
    ] {
        assert!(
            ContextProfiles::with_custom_profiles(vec![ContextProfile::custom(id, "Audio")])
                .is_err(),
            "{id}"
        );
    }
    for name in ["", "  ", " Audio", "Audio ", "Au\ndio", &"a".repeat(129)] {
        assert!(
            ContextProfiles::with_custom_profiles(vec![ContextProfile::custom("audio", name)])
                .is_err()
        );
    }
    let mut profile = ContextProfile::custom("audio", "Audio");
    profile.parent_context_id = Some("General".into());
    assert!(ContextProfiles::with_custom_profiles(vec![profile]).is_err());
}

#[test]
fn malformed_incomplete_unknown_and_multi_parent_documents_fail_safely() {
    for bytes in [
        b"{".as_slice(),
        b"null",
        b"{}",
        b"{\"schema_version\":1}",
        b"{\"schema_version\":-1}",
        b"{\"schema_version\":\"1\",\"profiles\":[]}",
    ] {
        assert!(ContextProfiles::from_json(bytes).is_err());
    }
    for field in ["context_id", "display_name", "built_in", "enabled"] {
        let mut value = document();
        value["profiles"][0].as_object_mut().unwrap().remove(field);
        assert!(ContextProfiles::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    for (field, value) in [
        ("extra", json!(true)),
        ("parent_context_id", json!(["general", "legal"])),
        ("enabled", json!("true")),
    ] {
        let mut doc = document();
        doc["profiles"][0][field] = value;
        assert!(ContextProfiles::from_json(&serde_json::to_vec(&doc).unwrap()).is_err());
    }
    let mut value = document();
    value["terms"] = json!([{"source": "test", "replacement": "unapproved"}]);
    assert!(ContextProfiles::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    value = document();
    let repeated = value["profiles"][0].clone();
    value["profiles"].as_array_mut().unwrap().push(repeated);
    assert!(ContextProfiles::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn duplicate_json_fields_are_rejected() {
    let bytes = String::from_utf8(ContextProfiles::default().to_json().unwrap()).unwrap();
    let repeated_version = bytes.replacen(
        "\"schema_version\": 1",
        "\"schema_version\": 1, \"schema_version\": 1",
        1,
    );
    assert!(ContextProfiles::from_json(repeated_version.as_bytes()).is_err());
    let repeated_id = bytes.replacen(
        "\"context_id\":",
        "\"context_id\": \"repeated\", \"context_id\":",
        1,
    );
    assert!(ContextProfiles::from_json(repeated_id.as_bytes()).is_err());
}

#[test]
fn version_dispatch_rejects_future_and_unsupported_old_versions() {
    assert_eq!(CONTEXT_PROFILE_SCHEMA_VERSION, 1);
    for version in [0, 2, u64::MAX] {
        let bytes = serde_json::to_vec(&json!({"schema_version": version})).unwrap();
        assert!(
            matches!(ContextProfiles::from_json(&bytes), Err(ContextProfileError::UnsupportedVersion(v)) if v == version)
        );
    }
}

#[test]
fn v1_identity_migration_and_resaving_are_deterministic() {
    let profiles =
        ContextProfiles::with_custom_profiles(vec![ContextProfile::custom("audio", "音響")])
            .unwrap();
    let mut value: Value = serde_json::from_slice(&profiles.to_json().unwrap()).unwrap();
    value["profiles"].as_array_mut().unwrap().reverse();
    let input = serde_json::to_vec(&value).unwrap();
    let first = ContextProfiles::from_json(&input).unwrap();
    let bytes = first.to_json().unwrap();
    let second = ContextProfiles::from_json(&bytes).unwrap();
    assert_eq!(first, profiles);
    assert_eq!(second.to_json().unwrap(), bytes);
    assert_eq!(chain(&second, "audio"), ["audio", "general"]);
}

#[test]
fn file_and_profile_count_limits_fail_safely() {
    assert!(matches!(
        ContextProfiles::from_json(&vec![b' '; MAX_CONTEXT_PROFILE_BYTES + 1]),
        Err(ContextProfileError::TooLarge)
    ));
    let profiles = (0..MAX_CONTEXT_PROFILES)
        .map(|n| ContextProfile::custom(format!("audio-{n}"), "Audio"))
        .collect();
    assert!(matches!(
        ContextProfiles::with_custom_profiles(profiles),
        Err(ContextProfileError::TooLarge)
    ));
}

#[test]
fn store_roundtrip_replaces_complete_snapshot_and_missing_load_has_no_side_effects() {
    let temp = tempdir().unwrap();
    let directory = temp.path().join("profiles");
    let store = ContextProfileStore::new(&directory);
    assert_eq!(store.path(), directory.join(CONTEXT_PROFILE_FILE));
    assert_eq!(store.load().unwrap(), ContextProfiles::default());
    assert!(!directory.exists());
    store.save(&ContextProfiles::default()).unwrap();
    let profiles =
        ContextProfiles::with_custom_profiles(vec![ContextProfile::custom("audio", "Audio")])
            .unwrap();
    store.save(&profiles).unwrap();
    let restarted = ContextProfileStore::new(&directory);
    assert_eq!(restarted.load().unwrap(), profiles);
    assert_eq!(fs::read_dir(directory).unwrap().count(), 1);
}

#[test]
fn bad_existing_storage_is_preserved_on_load_and_save() {
    let temp = tempdir().unwrap();
    let store = ContextProfileStore::new(temp.path());
    for bytes in [
        b"broken".to_vec(),
        b"{\"schema_version\":2}".to_vec(),
        vec![b' '; MAX_CONTEXT_PROFILE_BYTES + 1],
    ] {
        fs::write(store.path(), &bytes).unwrap();
        assert!(store.load().is_err());
        assert!(store.save(&ContextProfiles::default()).is_err());
        assert_eq!(fs::read(store.path()).unwrap(), bytes);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }
}

#[test]
#[cfg(windows)]
fn failed_windows_atomic_replacement_preserves_old_file_and_cleans_staging() {
    use std::os::windows::fs::OpenOptionsExt;
    let temp = tempdir().unwrap();
    let store = ContextProfileStore::new(temp.path());
    store.save(&ContextProfiles::default()).unwrap();
    let before = fs::read(store.path()).unwrap();
    // Permit reads/writes but not deletion/replacement while this handle is open.
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(store.path())
        .unwrap();
    let changed =
        ContextProfiles::with_custom_profiles(vec![ContextProfile::custom("audio", "Audio")])
            .unwrap();
    assert!(store.save(&changed).is_err());
    assert_eq!(fs::read(store.path()).unwrap(), before);
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    drop(held);
    store.save(&changed).unwrap();
    assert_eq!(store.load().unwrap(), changed);
}

#[test]
fn storage_io_failures_are_errors_without_repair_or_staged_leftovers() {
    let temp = tempdir().unwrap();
    let blocked = temp.path().join("not-a-directory");
    fs::write(&blocked, b"keep me").unwrap();
    let store = ContextProfileStore::new(&blocked);
    assert!(store.save(&ContextProfiles::default()).is_err());
    assert_eq!(fs::read(blocked).unwrap(), b"keep me");
    let store = ContextProfileStore::new(temp.path());
    fs::create_dir(store.path()).unwrap();
    assert!(store.load().is_err());
    assert!(store.save(&ContextProfiles::default()).is_err());
    assert!(store.path().is_dir());
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
}

#[test]
fn store_does_not_touch_desktop_settings_private_dictionary_or_runtime_contract() {
    let temp = tempdir().unwrap();
    let settings = temp.path().join("settings.json");
    let user = temp.path().join("user.sqlite");
    let settings_bytes =
        br#"{"schema_version":1,"shared_db":"unchanged","user_db":null,"user_enabled":false}"#;
    fs::write(&settings, settings_bytes).unwrap();
    let conn = rusqlite::Connection::open(&user).unwrap();
    conn.execute_batch(include_str!("../../schema/user-dictionary-v0.1.sql"))
        .unwrap();
    drop(conn);
    let user_before = fs::read(&user).unwrap();
    let store = ContextProfileStore::new(temp.path());
    let profiles =
        ContextProfiles::with_custom_profiles(vec![ContextProfile::custom("audio", "Audio")])
            .unwrap();
    store.save(&profiles).unwrap();
    assert_eq!(store.load().unwrap(), profiles);
    assert_eq!(fs::read(settings).unwrap(), settings_bytes);
    assert_eq!(fs::read(user).unwrap(), user_before);
    assert_eq!(chinese_regional_localizer::RUNTIME_API_VERSION, "1");
    let request: chinese_regional_localizer::RuntimeRequest = serde_json::from_value(json!({
        "text": "測試", "source_locale": "zh-CN", "target_locale": "zh-HK"
    }))
    .unwrap();
    assert_eq!(request.api_version, "1");
    assert!(request.context.is_none());
}
