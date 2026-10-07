use chinese_regional_localizer::{Runtime, RuntimeRequest, RuntimeResponse, RUNTIME_API_VERSION};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;

const SETTINGS_FILE: &str = "settings.json";
const SHARED_DB_KEY: &str = "shared_db";
const USER_DB_KEY: &str = "user_db";

#[derive(Debug, Clone)]
struct RuntimeConfig {
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
}

#[derive(Debug)]
struct AppState {
    config: RwLock<RuntimeConfig>,
}

impl AppState {
    fn new(config: RuntimeConfig) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    fn snapshot(&self) -> Result<RuntimeConfig, String> {
        self.config
            .read()
            .map(|config| config.clone())
            .map_err(|_| "Runtime settings lock is unavailable".to_owned())
    }

    fn replace(&self, config: RuntimeConfig) -> Result<(), String> {
        let mut current = self
            .config
            .write()
            .map_err(|_| "Runtime settings lock is unavailable".to_owned())?;
        *current = config;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
struct DatabaseStatus {
    api_version: String,
    shared_db: String,
    shared_exists: bool,
    user_db: Option<String>,
    user_exists: bool,
    user_dictionary_enabled: bool,
    runtime_ready: bool,
    validation_error: Option<String>,
}

fn default_shared_db() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/regional-demo.sqlite")
}

fn environment_config() -> RuntimeConfig {
    let shared_db = env::var_os("CRL_SHARED_DB")
        .map(PathBuf::from)
        .unwrap_or_else(default_shared_db);
    let user_db = env::var_os("CRL_USER_DB")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    RuntimeConfig { shared_db, user_db }
}

fn load_persisted_config(app: &tauri::App) -> Result<RuntimeConfig, Box<dyn std::error::Error>> {
    let fallback = environment_config();
    let store = app.store(SETTINGS_FILE)?;

    let shared_db = store
        .get(SHARED_DB_KEY)
        .and_then(|value| value.as_str().map(PathBuf::from))
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(fallback.shared_db);

    let user_db = match store.get(USER_DB_KEY) {
        Some(Value::String(path)) if !path.is_empty() => Some(PathBuf::from(path)),
        Some(Value::Null) => None,
        Some(_) => None,
        None => fallback.user_db,
    };

    Ok(RuntimeConfig { shared_db, user_db })
}

fn validation_request() -> RuntimeRequest {
    RuntimeRequest {
        api_version: RUNTIME_API_VERSION.to_owned(),
        text: String::new(),
        source_locale: "zh-CN".to_owned(),
        target_locale: "zh-HK".to_owned(),
        context: None,
    }
}

fn validate_config(config: &RuntimeConfig) -> Result<(), String> {
    if !config.shared_db.is_file() {
        return Err(format!(
            "Shared database not found: {}",
            config.shared_db.display()
        ));
    }
    if let Some(path) = &config.user_db {
        if !path.is_file() {
            return Err(format!("User dictionary not found: {}", path.display()));
        }
    }

    Runtime::new(&config.shared_db, config.user_db.as_ref())
        .localize(&validation_request())
        .map(|_| ())
        .map_err(|error| format!("Database validation failed: {error:?}"))
}

fn status_for(config: &RuntimeConfig) -> DatabaseStatus {
    let validation_error = validate_config(config).err();
    DatabaseStatus {
        api_version: RUNTIME_API_VERSION.to_owned(),
        shared_db: config.shared_db.to_string_lossy().into_owned(),
        shared_exists: config.shared_db.is_file(),
        user_db: config
            .user_db
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        user_exists: config.user_db.as_ref().is_some_and(|path| path.is_file()),
        user_dictionary_enabled: config.user_db.is_some(),
        runtime_ready: validation_error.is_none(),
        validation_error,
    }
}

fn persist_path(app: &AppHandle, key: &str, path: Option<&Path>) -> Result<(), String> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| format!("Could not open settings store: {error}"))?;
    let value = path
        .map(|path| json!(path.to_string_lossy().into_owned()))
        .unwrap_or(Value::Null);
    store.set(key, value);
    store
        .save()
        .map_err(|error| format!("Could not save settings: {error}"))
}

fn picked_path(app: &AppHandle, title: &str) -> Result<Option<PathBuf>, String> {
    app.dialog()
        .file()
        .set_title(title)
        .add_filter("SQLite database", &["sqlite", "sqlite3", "db"])
        .blocking_pick_file()
        .map(|path| {
            path.into_path()
                .map_err(|error| format!("Selected file is not a local path: {error}"))
        })
        .transpose()
}

#[tauri::command]
async fn runtime_status(state: State<'_, AppState>) -> Result<DatabaseStatus, String> {
    let config = state.snapshot()?;
    tauri::async_runtime::spawn_blocking(move || status_for(&config))
        .await
        .map_err(|error| format!("Status task failed: {error}"))
}

#[tauri::command]
async fn choose_shared_database(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DatabaseStatus, String> {
    let Some(path) = picked_path(&app, "Choose regional database")? else {
        return Ok(status_for(&state.snapshot()?));
    };

    let mut candidate = state.snapshot()?;
    candidate.shared_db = path.clone();
    validate_config(&candidate)?;
    persist_path(&app, SHARED_DB_KEY, Some(&path))?;
    state.replace(candidate.clone())?;
    Ok(status_for(&candidate))
}

#[tauri::command]
async fn choose_user_database(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DatabaseStatus, String> {
    let Some(path) = picked_path(&app, "Choose private user dictionary")? else {
        return Ok(status_for(&state.snapshot()?));
    };

    let mut candidate = state.snapshot()?;
    candidate.user_db = Some(path.clone());
    validate_config(&candidate)?;
    persist_path(&app, USER_DB_KEY, Some(&path))?;
    state.replace(candidate.clone())?;
    Ok(status_for(&candidate))
}

#[tauri::command]
async fn clear_user_database(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DatabaseStatus, String> {
    let mut candidate = state.snapshot()?;
    candidate.user_db = None;
    validate_config(&candidate)?;
    persist_path(&app, USER_DB_KEY, None)?;
    state.replace(candidate.clone())?;
    Ok(status_for(&candidate))
}

#[tauri::command]
async fn localize_text(
    state: State<'_, AppState>,
    request: RuntimeRequest,
) -> Result<RuntimeResponse, String> {
    let config = state.snapshot()?;
    validate_config(&config)?;

    tauri::async_runtime::spawn_blocking(move || {
        Runtime::new(&config.shared_db, config.user_db.as_ref())
            .localize(&request)
            .map_err(|error| format!("Localization failed: {error:?}"))
    })
    .await
    .map_err(|error| format!("Localization task failed: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            app.manage(AppState::new(load_persisted_config(app)?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            localize_text,
            runtime_status,
            choose_shared_database,
            choose_user_database,
            clear_user_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running Chinese Regional Localizer desktop app");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_database_path_targets_repo_build_directory() {
        assert!(default_shared_db().ends_with("build/regional-demo.sqlite"));
    }

    #[test]
    fn validation_rejects_missing_shared_database() {
        let config = RuntimeConfig {
            shared_db: PathBuf::from("definitely-not-a-real-regional-database.sqlite"),
            user_db: None,
        };
        let error = validate_config(&config).expect_err("missing DB must fail");
        assert!(error.contains("Shared database not found"));
    }

    #[test]
    fn status_reports_disabled_user_dictionary() {
        let config = RuntimeConfig {
            shared_db: default_shared_db(),
            user_db: None,
        };
        let status = status_for(&config);
        assert!(!status.user_dictionary_enabled);
        assert!(status.user_db.is_none());
    }
}
