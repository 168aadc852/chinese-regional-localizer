use chinese_regional_localizer::{Runtime, RuntimeRequest, RuntimeResponse};
use std::env;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug)]
struct AppState {
    shared_db: PathBuf,
    user_db: Option<PathBuf>,
}

fn default_shared_db() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/regional-demo.sqlite")
}

fn load_state() -> AppState {
    let shared_db = env::var_os("CRL_SHARED_DB")
        .map(PathBuf::from)
        .unwrap_or_else(default_shared_db);
    let user_db = env::var_os("CRL_USER_DB")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    AppState { shared_db, user_db }
}

#[tauri::command]
async fn localize_text(
    state: State<'_, AppState>,
    request: RuntimeRequest,
) -> Result<RuntimeResponse, String> {
    let shared_db = state.shared_db.clone();
    let user_db = state.user_db.clone();

    if !shared_db.is_file() {
        return Err(format!(
            "Shared database not found: {}. Build the demo database or set CRL_SHARED_DB.",
            shared_db.display()
        ));
    }
    if let Some(path) = &user_db {
        if !path.is_file() {
            return Err(format!(
                "User dictionary not found: {}. Remove CRL_USER_DB or point it to a valid user_dictionary.sqlite.",
                path.display()
            ));
        }
    }

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
        .manage(load_state())
        .invoke_handler(tauri::generate_handler![localize_text])
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
}
