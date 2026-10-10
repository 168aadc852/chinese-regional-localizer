//! Desktop-only preferences. Never serialize this document to the webview.
use super::{validate_shared_db, validate_user_db, DatabaseConfig};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

const SETTINGS_VERSION: u64 = 2;
const MAX_SETTINGS_BYTES: u64 = 64 * 1024;
const FALLBACK_MESSAGE: &str = "無法還原部分已儲存的設定；已使用可用的啟動預設設定。";
const FUTURE_MESSAGE: &str = "設定檔版本或欄位不受支援；已使用啟動預設設定，原檔不會被覆寫。";
const SAVE_MESSAGE: &str = "無法安全儲存設定；目前設定未變更。";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsV1 {
    schema_version: u64,
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
    user_enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum UiLocale {
    #[default]
    #[serde(rename = "zh-HK")]
    HongKong,
    #[serde(rename = "zh-TW")]
    Taiwan,
    #[serde(rename = "zh-CN")]
    ChineseMainland,
    #[serde(rename = "en")]
    English,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Appearance {
    #[default]
    System,
    Light,
    Dark,
    EinkMono,
}

/// Presentation only: never passed to the localization Runtime.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PresentationPreferences {
    pub(super) ui_locale: UiLocale,
    pub(super) appearance: Appearance,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsV2 {
    schema_version: u64,
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
    user_enabled: bool,
    presentation: PresentationPreferences,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum PresentationError {
    PreferencesUnavailable,
    SettingsUnsupported,
    SettingsSaveFailed,
}

enum DecodeError {
    Corrupt,
    Unsupported,
}

// Version dispatch is explicit: a future migration must return a complete,
// validated model. Never deserialize a newer document as an older version.
fn decode(bytes: &[u8]) -> Result<SettingsV2, DecodeError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| DecodeError::Corrupt)?;
    match value.get("schema_version").and_then(|v| v.as_u64()) {
        Some(version @ (1 | SETTINGS_VERSION)) => {
            let object = value.as_object().ok_or(DecodeError::Corrupt)?;
            if object.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "schema_version" | "shared_db" | "user_db" | "user_enabled"
                ) && !(version == SETTINGS_VERSION && key == "presentation")
            }) {
                return Err(DecodeError::Unsupported);
            }
            // Parse the original bytes so duplicate fields are rejected too.
            if version == 1 {
                let old: SettingsV1 =
                    serde_json::from_slice(bytes).map_err(|_| DecodeError::Corrupt)?;
                Ok(SettingsV2 {
                    schema_version: SETTINGS_VERSION,
                    shared_db: old.shared_db,
                    user_db: old.user_db,
                    user_enabled: old.user_enabled,
                    presentation: PresentationPreferences::default(),
                })
            } else {
                // Unknown nested fields are preserved, not silently discarded.
                if value
                    .get("presentation")
                    .and_then(|v| v.as_object())
                    .is_some_and(|p| {
                        p.keys()
                            .any(|k| !matches!(k.as_str(), "ui_locale" | "appearance"))
                    })
                {
                    return Err(DecodeError::Unsupported);
                }
                serde_json::from_slice(bytes).map_err(|_| DecodeError::Corrupt)
            }
        }
        Some(_) => Err(DecodeError::Unsupported),
        None => Err(DecodeError::Corrupt),
    }
}

#[derive(Debug)]
pub(super) struct SettingsStore {
    path: Option<PathBuf>,
    write_blocked: bool,
}

impl SettingsStore {
    pub(super) fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            write_blocked: false,
        }
    }

    pub(super) fn restore(&mut self, mut fallback: DatabaseConfig) -> DatabaseConfig {
        // An invalid environment dictionary must not prevent shared-only use.
        if fallback
            .user_db
            .as_deref()
            .is_some_and(|path| validate_user_db(path).is_err())
        {
            fallback.user_db = None;
            fallback.user_enabled = false;
            fallback.settings_message = Some(FALLBACK_MESSAGE.into());
        }
        let Some(path) = &self.path else {
            fallback.settings_message = Some(SAVE_MESSAGE.into());
            return fallback;
        };
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return fallback,
            Err(_) => {
                fallback.settings_message = Some(FALLBACK_MESSAGE.into());
                return fallback;
            }
        };
        let mut bytes = Vec::new();
        if file
            .take(MAX_SETTINGS_BYTES + 1)
            .read_to_end(&mut bytes)
            .is_err()
            || bytes.len() as u64 > MAX_SETTINGS_BYTES
        {
            fallback.settings_message = Some(FALLBACK_MESSAGE.into());
            return fallback;
        }
        let stored = match decode(&bytes) {
            Ok(stored) => stored,
            Err(DecodeError::Unsupported) => {
                self.write_blocked = true;
                fallback.settings_message = Some(FUTURE_MESSAGE.into());
                return fallback;
            }
            Err(DecodeError::Corrupt) => {
                fallback.settings_message = Some(FALLBACK_MESSAGE.into());
                return fallback;
            }
        };
        fallback.presentation = stored.presentation;
        if stored.shared_db.is_absolute() && validate_shared_db(&stored.shared_db).is_ok() {
            fallback.shared_db = stored.shared_db;
        } else {
            fallback.settings_message = Some(FALLBACK_MESSAGE.into());
        }
        match stored.user_db {
            Some(path) if path.is_absolute() && validate_user_db(&path).is_ok() => {
                fallback.user_db = Some(path);
                fallback.user_enabled = stored.user_enabled;
            }
            None if !stored.user_enabled => {
                fallback.user_db = None;
                fallback.user_enabled = false;
            }
            _ => {
                // A saved disabled state always wins over an environment default.
                // Never retain an invalid stored path, even while disabled.
                if !stored.user_enabled {
                    fallback.user_db = None;
                    fallback.user_enabled = false;
                }
                fallback.settings_message = Some(FALLBACK_MESSAGE.into());
            }
        }
        fallback
    }

    pub(super) fn save(&self, config: &DatabaseConfig) -> Result<(), String> {
        if self.write_blocked {
            return Err(FUTURE_MESSAGE.into());
        }
        let path = self.path.as_ref().ok_or(SAVE_MESSAGE)?;
        let parent = path.parent().ok_or(SAVE_MESSAGE)?;
        if !config.shared_db.is_absolute()
            || config.user_db.as_ref().is_some_and(|p| !p.is_absolute())
            || (config.user_enabled && config.user_db.is_none())
        {
            return Err(SAVE_MESSAGE.into());
        }
        let bytes = serde_json::to_vec_pretty(&SettingsV2 {
            schema_version: SETTINGS_VERSION,
            shared_db: config.shared_db.clone(),
            user_db: config.user_db.clone(),
            user_enabled: config.user_enabled,
            presentation: config.presentation.clone(),
        })
        .map_err(|_| SAVE_MESSAGE.to_string())?;
        if bytes.len() as u64 > MAX_SETTINGS_BYTES {
            return Err(SAVE_MESSAGE.into());
        }
        fs::create_dir_all(parent).map_err(|_| SAVE_MESSAGE.to_string())?;
        // Same-directory rename replaces the complete old document atomically,
        // including on Windows. Tempfile cleans up any failed staging write.
        let mut staged = NamedTempFile::new_in(parent).map_err(|_| SAVE_MESSAGE.to_string())?;
        staged
            .write_all(&bytes)
            .map_err(|_| SAVE_MESSAGE.to_string())?;
        staged
            .as_file()
            .sync_all()
            .map_err(|_| SAVE_MESSAGE.to_string())?;
        staged.persist(path).map_err(|_| SAVE_MESSAGE.to_string())?;
        Ok(())
    }

    pub(super) fn save_presentation(
        &self,
        config: &DatabaseConfig,
    ) -> Result<(), PresentationError> {
        if self.write_blocked {
            return Err(PresentationError::SettingsUnsupported);
        }
        self.save(config)
            .map_err(|_| PresentationError::SettingsSaveFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        accept_shared_database, accept_user_database, set_user_enabled, status_for, AppState,
    };
    use rusqlite::Connection;
    use std::sync::RwLock;
    use tempfile::{tempdir, TempDir};

    struct Fixture {
        temp: TempDir,
        fallback: DatabaseConfig,
        selected: DatabaseConfig,
        path: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = tempdir().unwrap();
            let database = |name: &str, user: bool| {
                let path = temp.path().join(name);
                let conn = Connection::open(&path).unwrap();
                conn.execute_batch(if user {
                    include_str!("../../../schema/user-dictionary-v0.1.sql")
                } else {
                    include_str!("../../../schema/sqlite-v0.2.sql")
                })
                .unwrap();
                path
            };
            let fallback = DatabaseConfig {
                shared_db: database("fallback.sqlite", false),
                user_db: Some(database("fallback-user.sqlite", true)),
                user_enabled: true,
                settings_message: None,
                presentation: PresentationPreferences::default(),
            };
            let selected = DatabaseConfig {
                shared_db: database("selected.sqlite", false),
                user_db: Some(database("selected-user.sqlite", true)),
                user_enabled: true,
                settings_message: None,
                presentation: PresentationPreferences::default(),
            };
            let path = temp.path().join("preferences").join("settings.json");
            Self {
                temp,
                fallback,
                selected,
                path,
            }
        }

        fn store(&self) -> SettingsStore {
            SettingsStore::new(Some(self.path.clone()))
        }

        fn restart(&self) -> DatabaseConfig {
            self.store().restore(self.fallback.clone())
        }

        fn state(&self, store: SettingsStore) -> AppState {
            AppState {
                config: RwLock::new(self.fallback.clone()),
                settings: store,
                update_offer: RwLock::new(None),
            }
        }

        fn write(&self, bytes: &[u8]) {
            fs::create_dir_all(self.path.parent().unwrap()).unwrap();
            fs::write(&self.path, bytes).unwrap();
        }
    }

    #[test]
    fn missing_settings_use_startup_defaults_without_writing() {
        let f = Fixture::new();
        let loaded = f.restart();
        assert_eq!(loaded.shared_db, f.fallback.shared_db);
        assert_eq!(loaded.user_db, f.fallback.user_db);
        assert!(loaded.user_enabled);
        assert!(!f.path.exists());
    }

    #[test]
    fn v1_migrates_in_memory_and_only_writes_v2_after_accepted_change() {
        let f = Fixture::new();
        let old = serde_json::to_vec(&SettingsV1 {
            schema_version: 1,
            shared_db: f.selected.shared_db.clone(),
            user_db: f.selected.user_db.clone(),
            user_enabled: false,
        })
        .unwrap();
        f.write(&old);
        let restored = f.restart();
        assert_eq!(restored.shared_db, f.selected.shared_db);
        assert_eq!(restored.user_db, f.selected.user_db);
        assert!(!restored.user_enabled);
        assert_eq!(restored.presentation, PresentationPreferences::default());
        assert_eq!(fs::read(&f.path).unwrap(), old);
        let state = AppState {
            config: RwLock::new(restored),
            ..f.state(f.store())
        };
        crate::change_presentation(
            &state,
            PresentationPreferences {
                ui_locale: UiLocale::English,
                appearance: Appearance::EinkMono,
            },
        )
        .unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&f.path).unwrap()).unwrap();
        assert_eq!(saved["schema_version"], 2);
        assert_eq!(saved["presentation"]["ui_locale"], "en");
        assert_eq!(saved["presentation"]["appearance"], "eink_mono");
        assert_eq!(f.restart().user_db, f.selected.user_db);
        assert!(!f.restart().user_enabled);
    }

    #[test]
    fn all_presentation_choices_round_trip_without_changing_database_choices() {
        let f = Fixture::new();
        let state = f.state(f.store());
        for ui_locale in [
            UiLocale::HongKong,
            UiLocale::Taiwan,
            UiLocale::ChineseMainland,
            UiLocale::English,
        ] {
            for appearance in [
                Appearance::System,
                Appearance::Light,
                Appearance::Dark,
                Appearance::EinkMono,
            ] {
                let preferences = PresentationPreferences {
                    ui_locale,
                    appearance,
                };
                crate::change_presentation(&state, preferences.clone()).unwrap();
                let restored = f.restart();
                assert_eq!(restored.presentation, preferences);
                assert_eq!(
                    fs::canonicalize(restored.shared_db).unwrap(),
                    fs::canonicalize(&f.fallback.shared_db).unwrap()
                );
                assert_eq!(restored.user_db, f.fallback.user_db);
                assert_eq!(restored.user_enabled, f.fallback.user_enabled);
                accept_shared_database(&state, f.selected.shared_db.clone()).unwrap();
                assert_eq!(f.restart().presentation, preferences);
                accept_shared_database(&state, f.fallback.shared_db.clone()).unwrap();
            }
        }
    }

    #[test]
    fn invalid_presentation_is_rejected_and_future_fields_are_write_protected() {
        let f = Fixture::new();
        for presentation in [
            serde_json::json!({"ui_locale":"fr", "appearance":"system"}),
            serde_json::json!({"ui_locale":"en", "appearance":"sepia"}),
            serde_json::json!({"ui_locale":"en"}),
            serde_json::json!({"ui_locale":"en", "appearance":"light", "future":true}),
        ] {
            let bytes = serde_json::to_vec(&serde_json::json!({
                "schema_version":2, "shared_db":f.selected.shared_db,
                "user_db":f.selected.user_db, "user_enabled":true,
                "presentation":presentation,
            }))
            .unwrap();
            f.write(&bytes);
            let mut store = f.store();
            let loaded = store.restore(f.fallback.clone());
            assert!(loaded.settings_message.is_some());
            assert_eq!(loaded.presentation, PresentationPreferences::default());
            assert_eq!(fs::read(&f.path).unwrap(), bytes);
            if presentation.get("future").is_some() {
                assert_eq!(
                    store.save_presentation(&loaded),
                    Err(PresentationError::SettingsUnsupported)
                );
                assert_eq!(fs::read(&f.path).unwrap(), bytes);
            }
        }
        assert!(serde_json::from_str::<PresentationPreferences>(
            r#"{"ui_locale":"en","ui_locale":"zh-HK","appearance":"light"}"#
        )
        .is_err());
    }

    #[test]
    fn failed_presentation_save_has_typed_path_free_error_and_no_session_change() {
        let f = Fixture::new();
        let blocked_parent = f.temp.path().join("blocked");
        fs::write(&blocked_parent, b"preserve").unwrap();
        let state = f.state(SettingsStore::new(Some(
            blocked_parent.join("settings.json"),
        )));
        let error = crate::change_presentation(
            &state,
            PresentationPreferences {
                ui_locale: UiLocale::Taiwan,
                appearance: Appearance::Dark,
            },
        )
        .unwrap_err();
        assert_eq!(serde_json::to_value(error).unwrap(), "settings_save_failed");
        assert_eq!(
            state.config.read().unwrap().presentation,
            PresentationPreferences::default()
        );
        assert_eq!(fs::read(blocked_parent).unwrap(), b"preserve");
    }

    #[test]
    fn concurrent_presentation_and_database_changes_keep_both() {
        let f = Fixture::new();
        let state = f.state(f.store());
        let preferences = PresentationPreferences {
            ui_locale: UiLocale::English,
            appearance: Appearance::Dark,
        };
        std::thread::scope(|scope| {
            scope.spawn(|| accept_shared_database(&state, f.selected.shared_db.clone()).unwrap());
            scope.spawn(|| crate::change_presentation(&state, preferences.clone()).unwrap());
        });
        assert_eq!(f.restart().presentation, preferences);
        assert_eq!(
            f.restart().shared_db,
            fs::canonicalize(f.selected.shared_db).unwrap()
        );
    }

    #[test]
    fn accepted_choices_survive_restart_and_repeated_atomic_replacement() {
        let f = Fixture::new();
        let state = f.state(f.store());
        accept_shared_database(&state, f.selected.shared_db.clone()).unwrap();
        accept_user_database(&state, f.selected.user_db.clone().unwrap()).unwrap();
        let restored = f.restart();
        assert_eq!(
            restored.shared_db,
            fs::canonicalize(&f.selected.shared_db).unwrap()
        );
        assert_eq!(
            restored.user_db,
            Some(fs::canonicalize(f.selected.user_db.unwrap()).unwrap())
        );
        assert!(restored.user_enabled);
        assert!(restored.settings_message.is_none());
        assert_eq!(fs::read_dir(f.path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn disabled_choice_survives_restart_over_environment_default_and_can_be_reenabled() {
        let f = Fixture::new();
        let state = f.state(f.store());
        accept_user_database(&state, f.selected.user_db.clone().unwrap()).unwrap();
        set_user_enabled(&state, false).unwrap();
        let restored = f.restart();
        assert!(!restored.user_enabled);
        assert!(restored.active_user_db().is_none());
        assert!(restored.user_db.is_some());
        let next = AppState {
            config: RwLock::new(restored),
            ..f.state(f.store())
        };
        set_user_enabled(&next, true).unwrap();
        assert!(f.restart().active_user_db().is_some());
    }

    #[test]
    fn disabled_without_selection_stays_disabled_over_environment_default() {
        let f = Fixture::new();
        let mut config = f.selected.clone();
        config.user_db = None;
        config.user_enabled = false;
        f.store().save(&config).unwrap();
        let restored = f.restart();
        assert!(restored.user_db.is_none());
        assert!(!restored.user_enabled);
    }

    #[test]
    fn missing_corrupt_and_incompatible_shared_paths_never_replace_working_fallback() {
        let f = Fixture::new();
        let corrupt = f.temp.path().join("corrupt.sqlite");
        fs::write(&corrupt, b"not sqlite").unwrap();
        let wrong = f.temp.path().join("wrong.sqlite");
        Connection::open(&wrong).unwrap();
        for path in [f.temp.path().join("missing.sqlite"), corrupt, wrong] {
            let mut config = f.selected.clone();
            config.shared_db = path;
            f.store().save(&config).unwrap();
            let restored = f.restart();
            assert_eq!(restored.shared_db, f.fallback.shared_db);
            assert_eq!(restored.user_db, f.selected.user_db);
            assert!(restored.settings_message.is_some());
        }
    }

    #[test]
    fn invalid_user_path_does_not_block_valid_shared_restore() {
        let f = Fixture::new();
        let mut config = f.selected.clone();
        config.user_db = Some(f.temp.path().join("missing-user.sqlite"));
        f.store().save(&config).unwrap();
        let restored = f.restart();
        assert_eq!(restored.shared_db, f.selected.shared_db);
        assert_eq!(restored.user_db, f.fallback.user_db);
        assert!(restored.user_enabled);
    }

    #[test]
    fn same_table_names_with_incompatible_columns_are_not_restored() {
        let f = Fixture::new();
        let wrong = f.temp.path().join("wrong-columns.sqlite");
        Connection::open(&wrong)
            .unwrap()
            .execute_batch(
                "CREATE TABLE concepts(id INTEGER); CREATE TABLE localized_names(id INTEGER);
             CREATE TABLE term_rules(id INTEGER); CREATE TABLE source_versions(id INTEGER);
             CREATE TABLE name_evidence(id INTEGER); CREATE TABLE external_ids(id INTEGER);
             CREATE TABLE user_terms(id INTEGER);",
            )
            .unwrap();
        let mut config = f.selected.clone();
        config.shared_db = wrong.clone();
        config.user_db = Some(wrong);
        f.store().save(&config).unwrap();
        let restored = f.restart();
        assert_eq!(restored.shared_db, f.fallback.shared_db);
        assert_eq!(restored.user_db, f.fallback.user_db);
        assert!(restored.settings_message.is_some());
    }

    #[test]
    fn restarted_dictionary_toggle_preserves_runtime_api_v1_localization() {
        let f = Fixture::new();
        Connection::open(f.selected.user_db.as_ref().unwrap()).unwrap().execute_batch(
            "INSERT INTO user_terms (kind, source_text, replacement, target_locale, created_at, updated_at)
             VALUES ('override', '詞', '偏好', 'zh-HK', '2026-10-08', '2026-10-08');"
        ).unwrap();
        let request = crate::RuntimeRequest {
            api_version: "1".into(),
            text: "詞".into(),
            source_locale: "zh-CN".into(),
            target_locale: "zh-HK".into(),
            context: None,
        };
        let localize = |config: DatabaseConfig| {
            crate::Runtime::new(&config.shared_db, config.active_user_db())
                .localize(&request)
                .unwrap()
        };
        f.store().save(&f.selected).unwrap();
        let enabled = localize(f.restart());
        assert_eq!(enabled.api_version, "1");
        assert_eq!(enabled.output, "偏好");
        assert!(enabled.user_dictionary_applied);
        let state = AppState {
            config: RwLock::new(f.restart()),
            ..f.state(f.store())
        };
        set_user_enabled(&state, false).unwrap();
        let disabled = localize(f.restart());
        assert_eq!(disabled.api_version, "1");
        assert_eq!(disabled.output, "詞");
        assert!(!disabled.user_dictionary_applied);
    }

    #[test]
    fn deleted_dictionary_cannot_be_reenabled_or_change_saved_disabled_state() {
        let f = Fixture::new();
        let state = f.state(f.store());
        accept_user_database(&state, f.selected.user_db.clone().unwrap()).unwrap();
        set_user_enabled(&state, false).unwrap();
        let before = fs::read(&f.path).unwrap();
        fs::remove_file(f.selected.user_db.unwrap()).unwrap();
        assert!(set_user_enabled(&state, true).is_err());
        assert!(state.config.read().unwrap().active_user_db().is_none());
        assert_eq!(fs::read(&f.path).unwrap(), before);
    }

    #[test]
    fn missing_disabled_user_is_not_replaced_by_enabled_environment_user() {
        let f = Fixture::new();
        let mut config = f.selected.clone();
        config.user_db = Some(f.temp.path().join("missing-user.sqlite"));
        config.user_enabled = false;
        f.store().save(&config).unwrap();
        let restored = f.restart();
        assert!(restored.user_db.is_none());
        assert!(!restored.user_enabled);
    }

    #[test]
    fn invalid_environment_user_falls_back_to_shared_only() {
        let f = Fixture::new();
        let mut fallback = f.fallback.clone();
        fallback.user_db = Some(f.temp.path().join("missing.sqlite"));
        let restored = f.store().restore(fallback);
        assert!(restored.active_user_db().is_none());
        assert!(restored.settings_message.is_some());
    }

    #[test]
    fn malformed_truncated_oversized_and_incomplete_settings_fall_back_without_rewriting() {
        let f = Fixture::new();
        for bytes in [
            b"{\"schema_version\":1,".to_vec(),
            b"null".to_vec(),
            b"{\"schema_version\":1}".to_vec(),
            b"{\"schema_version\":1,\"schema_version\":1}".to_vec(),
            vec![b' '; MAX_SETTINGS_BYTES as usize + 1],
        ] {
            f.write(&bytes);
            let restored = f.restart();
            assert_eq!(restored.shared_db, f.fallback.shared_db);
            assert!(restored.settings_message.is_some());
            assert_eq!(fs::read(&f.path).unwrap(), bytes);
        }
        // A deliberate new accepted selection can repair a corrupt v1 file.
        accept_shared_database(&f.state(f.store()), f.selected.shared_db.clone()).unwrap();
        assert!(f.restart().settings_message.is_none());
    }

    #[test]
    fn unsupported_versions_and_fields_are_preserved_for_explicit_future_migrations() {
        let f = Fixture::new();
        for bytes in [
            b"{\"schema_version\":0}".as_slice(),
            b"{\"schema_version\":3}",
            b"{\"schema_version\":1,\"future_preference\":true}",
        ] {
            f.write(bytes);
            let mut store = f.store();
            let restored = store.restore(f.fallback.clone());
            assert_eq!(restored.shared_db, f.fallback.shared_db);
            assert_eq!(restored.settings_message.as_deref(), Some(FUTURE_MESSAGE));
            assert!(store.save(&f.selected).is_err());
            assert_eq!(fs::read(&f.path).unwrap(), bytes);
        }
    }

    #[test]
    fn relative_stored_paths_are_rejected_and_defaults_are_retained() {
        let f = Fixture::new();
        f.write(b"{\"schema_version\":1,\"shared_db\":\"relative.sqlite\",\"user_db\":\"relative-user.sqlite\",\"user_enabled\":true}");
        let restored = f.restart();
        assert_eq!(restored.shared_db, f.fallback.shared_db);
        assert_eq!(restored.user_db, f.fallback.user_db);
    }

    #[test]
    fn rejected_selection_leaves_session_and_settings_bytes_unchanged() {
        let f = Fixture::new();
        f.store().save(&f.fallback).unwrap();
        let before = fs::read(&f.path).unwrap();
        let state = f.state(f.store());
        assert!(accept_shared_database(&state, f.temp.path().join("missing.sqlite")).is_err());
        assert!(accept_user_database(&state, f.fallback.shared_db.clone()).is_err());
        assert_eq!(state.config.read().unwrap().shared_db, f.fallback.shared_db);
        assert_eq!(state.config.read().unwrap().user_db, f.fallback.user_db);
        assert_eq!(fs::read(&f.path).unwrap(), before);
    }

    #[test]
    fn failed_save_never_changes_the_working_session_or_exposes_paths() {
        let f = Fixture::new();
        let blocked_parent = f.temp.path().join("not-a-directory");
        fs::write(&blocked_parent, b"keep me").unwrap();
        let state = f.state(SettingsStore::new(Some(
            blocked_parent.join("settings.json"),
        )));
        let error = accept_shared_database(&state, f.selected.shared_db.clone()).unwrap_err();
        assert_eq!(error, SAVE_MESSAGE);
        assert_eq!(state.config.read().unwrap().shared_db, f.fallback.shared_db);
        assert_eq!(fs::read(&blocked_parent).unwrap(), b"keep me");
        // Failure at final replacement also cleans up the staged file.
        fs::create_dir_all(&f.path).unwrap();
        let state = f.state(f.store());
        assert!(set_user_enabled(&state, false).is_err());
        assert!(state.config.read().unwrap().user_enabled);
        assert_eq!(fs::read_dir(f.path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn unavailable_config_directory_uses_defaults_and_reports_safe_error() {
        let f = Fixture::new();
        let mut store = SettingsStore::new(None);
        let restored = store.restore(f.fallback.clone());
        assert_eq!(restored.shared_db, f.fallback.shared_db);
        assert!(restored.settings_message.is_some());
        assert_eq!(store.save(&f.selected).unwrap_err(), SAVE_MESSAGE);
    }

    #[test]
    fn frontend_status_contains_only_names_state_and_path_free_messages() {
        let f = Fixture::new();
        f.write(b"broken");
        let restored = f.restart();
        let status = serde_json::to_value(status_for(&restored)).unwrap();
        assert_eq!(status["shared_name"], "fallback.sqlite");
        assert_eq!(status["user_name"], "fallback-user.sqlite");
        assert!(status.get("shared_db").is_none());
        assert!(status.get("user_db").is_none());
        assert!(!status.to_string().contains("settings.json"));
        assert!(!status
            .to_string()
            .contains(&f.temp.path().to_string_lossy().to_string()));
    }

    #[test]
    fn concurrent_changes_are_serialized_without_losing_either_preference() {
        let f = Fixture::new();
        let state = f.state(f.store());
        std::thread::scope(|scope| {
            scope.spawn(|| accept_shared_database(&state, f.selected.shared_db.clone()).unwrap());
            scope.spawn(|| {
                accept_user_database(&state, f.selected.user_db.clone().unwrap()).unwrap()
            });
        });
        let restored = f.restart();
        assert_eq!(
            restored.shared_db,
            fs::canonicalize(f.selected.shared_db).unwrap()
        );
        assert_eq!(
            restored.user_db,
            Some(fs::canonicalize(f.selected.user_db.unwrap()).unwrap())
        );
    }
}
