use chinese_regional_localizer::{
    package_is_runtime_compatible, AuthenticatedCatalog, PackageStore, ReleaseCatalogPackage,
    ReqwestTransport, Runtime, RuntimeRequest, RuntimeResponse, TrustedKey, TrustedKeySet,
    UpdateClient, UpdateConfig,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Clone)]
struct DatabaseConfig {
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct UpdateRuntimeConfig {
    base_url: String,
    trusted_keys_file: PathBuf,
    store_root: PathBuf,
    package_id: String,
}

#[derive(Debug, Clone)]
struct UpdateOffer {
    catalog: AuthenticatedCatalog,
    package: ReleaseCatalogPackage,
}

#[derive(Debug)]
struct AppState {
    config: RwLock<DatabaseConfig>,
    update_offer: RwLock<Option<UpdateOffer>>,
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

#[derive(Debug, Clone, Serialize)]
struct UpdateStatus {
    configured: bool,
    current_version: Option<String>,
    offered_version: Option<String>,
    update_available: bool,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TrustedKeyDocument {
    List(Vec<TrustedKey>),
    Object { keys: Vec<TrustedKey> },
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

fn load_update_config() -> Result<Option<UpdateRuntimeConfig>, String> {
    let base_url = env::var("CRL_UPDATE_BASE_URL")
        .ok()
        .filter(|v| !v.is_empty());
    let trusted_keys_file = env::var_os("CRL_TRUSTED_KEYS_FILE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);
    let store_root = env::var_os("CRL_UPDATE_STORE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);
    let package_id = env::var("CRL_UPDATE_PACKAGE_ID")
        .ok()
        .filter(|v| !v.is_empty());

    if base_url.is_none()
        && trusted_keys_file.is_none()
        && store_root.is_none()
        && package_id.is_none()
    {
        return Ok(None);
    }
    match (base_url, trusted_keys_file, store_root, package_id) {
        (Some(base_url), Some(trusted_keys_file), Some(store_root), Some(package_id)) => {
            Ok(Some(UpdateRuntimeConfig {
                base_url,
                trusted_keys_file,
                store_root,
                package_id,
            }))
        }
        _ => Err("更新服務設定不完整；需要 CRL_UPDATE_BASE_URL、CRL_TRUSTED_KEYS_FILE、CRL_UPDATE_STORE 及 CRL_UPDATE_PACKAGE_ID。".into()),
    }
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

fn read_trusted_keys(path: &Path) -> Result<TrustedKeySet, String> {
    let bytes = fs::read(path).map_err(|_| "無法讀取已配置的 trusted-key file。".to_string())?;
    let document: TrustedKeyDocument = serde_json::from_slice(&bytes)
        .map_err(|_| "Trusted-key file 不是有效 JSON。".to_string())?;
    let keys = match document {
        TrustedKeyDocument::List(keys) | TrustedKeyDocument::Object { keys } => keys,
    };
    TrustedKeySet::from_entries(&keys).map_err(|error| format!("Trusted-key 設定無效：{error:?}"))
}

fn build_update_client(
    config: &UpdateRuntimeConfig,
) -> Result<UpdateClient<ReqwestTransport>, String> {
    let transport =
        ReqwestTransport::new().map_err(|error| format!("無法建立安全更新連線：{error:?}"))?;
    let update_config = UpdateConfig::production(config.base_url.clone())
        .map_err(|error| format!("更新來源設定無效：{error:?}"))?;
    let trusted_keys = read_trusted_keys(&config.trusted_keys_file)?;
    UpdateClient::new(
        transport,
        update_config,
        trusted_keys,
        PackageStore::new(&config.store_root),
    )
    .map_err(|error| format!("無法初始化更新服務：{error:?}"))
}

fn unix_now() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系統時間無效，無法驗證更新 catalog。".to_string())?
        .as_secs();
    i64::try_from(seconds).map_err(|_| "系統時間超出支援範圍。".to_string())
}

fn current_package_version(config: &UpdateRuntimeConfig) -> Result<Option<String>, String> {
    let state = PackageStore::new(&config.store_root)
        .state()
        .map_err(|error| format!("無法讀取本機 package 狀態：{error:?}"))?;
    Ok(state
        .current
        .filter(|item| item.package_id == config.package_id)
        .map(|item| item.version))
}

fn recommended_package(
    catalog: &AuthenticatedCatalog,
    package_id: &str,
    current_version: Option<&str>,
) -> Option<ReleaseCatalogPackage> {
    catalog
        .catalog()
        .packages
        .iter()
        .rev()
        .find(|package| {
            package.package_id == package_id
                && package_is_runtime_compatible(package)
                && current_version != Some(package.version.as_str())
        })
        .cloned()
}

fn update_status_for(state: &State<'_, AppState>) -> Result<UpdateStatus, String> {
    let Some(config) = load_update_config()? else {
        return Ok(UpdateStatus {
            configured: false,
            current_version: None,
            offered_version: None,
            update_available: false,
            message: Some("更新服務尚未設定；離線地區化功能不受影響。".into()),
        });
    };
    let current_version = current_package_version(&config)?;
    let offered_version = state
        .update_offer
        .read()
        .map_err(|_| "Update state is unavailable.".to_string())?
        .as_ref()
        .map(|offer| offer.package.version.clone());
    Ok(UpdateStatus {
        configured: true,
        update_available: offered_version.is_some(),
        current_version,
        offered_version,
        message: None,
    })
}

#[tauri::command]
fn database_status(state: State<'_, AppState>) -> Result<DatabaseStatus, String> {
    Ok(status_for(&current_config(&state)?))
}

#[tauri::command]
fn update_status(state: State<'_, AppState>) -> Result<UpdateStatus, String> {
    update_status_for(&state)
}

#[tauri::command]
async fn check_data_update(state: State<'_, AppState>) -> Result<UpdateStatus, String> {
    let config = load_update_config()?.ok_or_else(|| {
        "更新服務尚未設定；請由應用程式發行設定提供受信任的更新來源。".to_string()
    })?;
    let current_version = current_package_version(&config)?;
    let config_for_task = config.clone();
    let catalog = tauri::async_runtime::spawn_blocking(move || {
        let client = build_update_client(&config_for_task)?;
        client
            .discover(unix_now()?)
            .map_err(|error| format!("檢查更新失敗：{error:?}"))
    })
    .await
    .map_err(|error| format!("更新檢查工作失敗：{error}"))??;

    let package = recommended_package(&catalog, &config.package_id, current_version.as_deref());
    {
        let mut offer = state
            .update_offer
            .write()
            .map_err(|_| "Update state is unavailable.".to_string())?;
        *offer = package.map(|package| UpdateOffer { catalog, package });
    }
    let mut status = update_status_for(&state)?;
    if status.update_available {
        status.message = Some("已找到經簽章驗證的資料更新。".into());
    } else {
        status.message = Some("目前沒有可用的新資料版本。".into());
    }
    Ok(status)
}

#[tauri::command]
async fn install_data_update(state: State<'_, AppState>) -> Result<UpdateStatus, String> {
    let config = load_update_config()?.ok_or_else(|| "更新服務尚未設定。".to_string())?;
    let offer = state
        .update_offer
        .read()
        .map_err(|_| "Update state is unavailable.".to_string())?
        .clone()
        .ok_or_else(|| "沒有待安裝的已驗證更新；請先檢查更新。".to_string())?;
    let config_for_task = config.clone();
    let package_id = offer.package.package_id.clone();
    let version = offer.package.version.clone();
    let active_db = tauri::async_runtime::spawn_blocking(move || {
        let client = build_update_client(&config_for_task)?;
        client
            .install_from_catalog(&offer.catalog, &package_id, &version)
            .map_err(|error| format!("安裝更新失敗：{error:?}"))?;
        PackageStore::new(&config_for_task.store_root)
            .active_database_path()
            .map_err(|error| format!("無法取得已啟用 database：{error:?}"))?
            .ok_or_else(|| "更新安裝完成但找不到已啟用 database。".to_string())
    })
    .await
    .map_err(|error| format!("更新安裝工作失敗：{error}"))??;

    validate_shared_db(&active_db)?;
    {
        let mut database = state
            .config
            .write()
            .map_err(|_| "Database configuration state is unavailable.".to_string())?;
        database.shared_db = active_db;
    }
    {
        let mut pending = state
            .update_offer
            .write()
            .map_err(|_| "Update state is unavailable.".to_string())?;
        *pending = None;
    }
    let mut status = update_status_for(&state)?;
    status.message = Some("資料更新已驗證、安裝並啟用。".into());
    Ok(status)
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
            update_offer: RwLock::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            localize_text,
            database_status,
            update_status,
            check_data_update,
            install_data_update,
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
        fs::create_dir_all(shared.parent().expect("parent")).expect("mkdir");
        create_shared_db(&shared);
        let status = status_for(&DatabaseConfig {
            shared_db: shared,
            user_db: None,
        });
        assert_eq!(status.shared_name, "shared.sqlite");
        assert!(status.shared_ready);
        assert!(status.user_name.is_none());
    }

    #[test]
    fn recommended_package_uses_signed_catalog_order_and_compatibility() {
        use chinese_regional_localizer::ReleaseCatalog;
        let catalog = AuthenticatedCatalog::from_test_parts(
            ReleaseCatalog {
                catalog_version: 1,
                sequence: 2,
                generated_at_unix: 10,
                expires_at_unix: 20,
                packages: vec![
                    ReleaseCatalogPackage {
                        package_id: "core".into(),
                        version: "1".into(),
                        pack_type: "core".into(),
                        min_runtime_api: "1".into(),
                        manifest_sha256: "a".repeat(64),
                    },
                    ReleaseCatalogPackage {
                        package_id: "core".into(),
                        version: "2".into(),
                        pack_type: "core".into(),
                        min_runtime_api: "1".into(),
                        manifest_sha256: "b".repeat(64),
                    },
                ],
            },
            "test-key".into(),
        );
        let selected = recommended_package(&catalog, "core", Some("1")).expect("offer");
        assert_eq!(selected.version, "2");
    }
}
