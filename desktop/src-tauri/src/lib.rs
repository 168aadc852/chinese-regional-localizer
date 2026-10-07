use chinese_regional_localizer::{Runtime, RuntimeRequest, RuntimeResponse};
use rusqlite::Connection;
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Clone)]
struct DatabaseConfig {
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
}

#[derive(Debug)]
struct AppState {
    config: RwLock<DatabaseConfig>,
}

#[derive(Debug, Clone, Serialize)]
struct DatabaseStatus {
    shared_name: String,
    shared_ready: bool,
    shared_message: Option<String>,
    user_name: Option<String>,
    user_ready: bool,
    user_message: Option<String>,
}

fn default_shared_db() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/regional-demo.sqlite")
}

fn load_config() -> DatabaseConfig {
    let shared_db = env::var_os("CRL_SHARED_DB")
        .map(PathBuf::from)
        .unwrap_or_else(default_shared_db);
    let user_db = env::var_os("CRL_USER_DB")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    DatabaseConfig { shared_db, user_db }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("database")
        .to_owned()
}

fn has_table(conn: &Connection, table: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [table],
        |row| row.get(0),
    )
}

fn validate_shared_db(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("選擇的 shared database 檔案不存在。".into());
    }
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "無法以 SQLite database 開啟所選檔案。".to_string())?;
    for table in [
        "concepts",
        "localized_names",
        "term_rules",
        "source_versions",
    ] {
        if !has_table(&conn, table).map_err(|_| "無法檢查 shared database schema。".to_string())?
        {
            return Err(format!(
                "所選檔案不是相容的 shared database：缺少 {table} table。"
            ));
        }
    }
    Ok(())
}

fn validate_user_db(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("選擇的 user dictionary 檔案不存在。".into());
    }
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "無法以 SQLite database 開啟所選 user dictionary。".to_string())?;
    if !has_table(&conn, "user_terms")
        .map_err(|_| "無法檢查 user dictionary schema。".to_string())?
    {
        return Err("所選檔案不是相容的 user dictionary：缺少 user_terms table。".into());
    }
    Ok(())
}

fn status_for(config: &DatabaseConfig) -> DatabaseStatus {
    let shared_check = validate_shared_db(&config.shared_db);
    let (shared_ready, shared_message) = match shared_check {
        Ok(()) => (true, None),
        Err(message) => (false, Some(message)),
    };
    let (user_name, user_ready, user_message) = match &config.user_db {
        Some(path) => match validate_user_db(path) {
            Ok(()) => (Some(display_name(path)), true, None),
            Err(message) => (Some(display_name(path)), false, Some(message)),
        },
        None => (None, true, None),
    };
    DatabaseStatus {
        shared_name: display_name(&config.shared_db),
        shared_ready,
        shared_message,
        user_name,
        user_ready,
        user_message,
    }
}

fn current_config(state: &State<'_, AppState>) -> Result<DatabaseConfig, String> {
    state
        .config
        .read()
        .map(|config| config.clone())
        .map_err(|_| "Database configuration state is unavailable.".to_string())
}

#[tauri::command]
fn database_status(state: State<'_, AppState>) -> Result<DatabaseStatus, String> {
    Ok(status_for(&current_config(&state)?))
}

#[tauri::command]
async fn choose_shared_database(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DatabaseStatus, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("SQLite database", &["sqlite", "db"])
        .blocking_pick_file();
    let Some(selected) = selected else {
        return database_status(state);
    };
    let path = selected
        .into_path()
        .map_err(|_| "所選項目不是可用的本機檔案。".to_string())?;
    validate_shared_db(&path)?;
    {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Database configuration state is unavailable.".to_string())?;
        config.shared_db = path;
    }
    database_status(state)
}

#[tauri::command]
async fn choose_user_database(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DatabaseStatus, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("SQLite database", &["sqlite", "db"])
        .blocking_pick_file();
    let Some(selected) = selected else {
        return database_status(state);
    };
    let path = selected
        .into_path()
        .map_err(|_| "所選項目不是可用的本機檔案。".to_string())?;
    validate_user_db(&path)?;
    {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Database configuration state is unavailable.".to_string())?;
        config.user_db = Some(path);
    }
    database_status(state)
}

#[tauri::command]
fn clear_user_database(state: State<'_, AppState>) -> Result<DatabaseStatus, String> {
    {
        let mut config = state
            .config
            .write()
            .map_err(|_| "Database configuration state is unavailable.".to_string())?;
        config.user_db = None;
    }
    database_status(state)
}

#[tauri::command]
async fn localize_text(
    state: State<'_, AppState>,
    request: RuntimeRequest,
) -> Result<RuntimeResponse, String> {
    let config = current_config(&state)?;
    validate_shared_db(&config.shared_db)?;
    if let Some(path) = &config.user_db {
        validate_user_db(path)?;
    }
    let shared_db = config.shared_db;
    let user_db = config.user_db;

    tauri::async_runtime::spawn_blocking(move || {
        Runtime::new(&shared_db, user_db.as_ref())
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
        .manage(AppState {
            config: RwLock::new(load_config()),
        })
        .invoke_handler(tauri::generate_handler![
            localize_text,
            database_status,
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
    use tempfile::tempdir;

    fn create_shared_db(path: &Path) {
        let conn = Connection::open(path).expect("create shared db");
        conn.execute_batch(
            "CREATE TABLE concepts(id INTEGER);\n             CREATE TABLE localized_names(id INTEGER);\n             CREATE TABLE term_rules(id INTEGER);\n             CREATE TABLE source_versions(id INTEGER);",
        )
        .expect("create schema");
    }

    fn create_user_db(path: &Path) {
        let conn = Connection::open(path).expect("create user db");
        conn.execute_batch("CREATE TABLE user_terms(id INTEGER);")
            .expect("create user schema");
    }

    #[test]
    fn default_database_path_targets_repo_build_directory() {
        assert!(default_shared_db().ends_with("build/regional-demo.sqlite"));
    }

    #[test]
    fn shared_database_validation_accepts_expected_schema() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("shared.sqlite");
        create_shared_db(&path);
        assert!(validate_shared_db(&path).is_ok());
    }

    #[test]
    fn shared_database_validation_rejects_wrong_schema() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("wrong.sqlite");
        Connection::open(&path).expect("create wrong db");
        assert!(validate_shared_db(&path).is_err());
    }

    #[test]
    fn user_database_validation_accepts_user_terms_schema() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("user.sqlite");
        create_user_db(&path);
        assert!(validate_user_db(&path).is_ok());
    }

    #[test]
    fn status_does_not_expose_full_paths() {
        let temp = tempdir().expect("tempdir");
        let shared = temp.path().join("private-folder").join("shared.sqlite");
        std::fs::create_dir_all(shared.parent().expect("parent")).expect("mkdir");
        create_shared_db(&shared);
        let status = status_for(&DatabaseConfig {
            shared_db: shared,
            user_db: None,
        });
        assert_eq!(status.shared_name, "shared.sqlite");
        assert!(status.shared_ready);
        assert!(status.user_name.is_none());
    }
}
