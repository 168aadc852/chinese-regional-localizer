use chinese_regional_localizer::{Runtime, RuntimeRequest, RuntimeResponse};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug)]
pub struct AppState {
    pub shared_db: PathBuf,
    pub user_db: Option<PathBuf>,
}

impl AppState {
    pub fn development_defaults() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let shared_db = std::env::var_os("CRL_SHARED_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|| manifest_dir.join("../../build/regional-demo.sqlite"));
        let user_db = std::env::var_os("CRL_USER_DB").map(PathBuf::from);
        Self { shared_db, user_db }
    }
}

fn run_localization(state: &AppState, request: RuntimeRequest) -> Result<RuntimeResponse, String> {
    if !state.shared_db.exists() {
        return Err(format!(
            "Shared database not found: {}. Build the demo database first or set CRL_SHARED_DB.",
            state.shared_db.display()
        ));
    }
    if let Some(user_db) = &state.user_db {
        if !user_db.exists() {
            return Err(format!(
                "Configured user database not found: {}",
                user_db.display()
            ));
        }
    }

    Runtime::new(&state.shared_db, state.user_db.as_ref())
        .localize(&request)
        .map_err(|error| format!("{error:?}"))
}

#[tauri::command]
pub async fn localize_text(
    state: State<'_, AppState>,
    request: RuntimeRequest,
) -> Result<RuntimeResponse, String> {
    let shared_db = state.shared_db.clone();
    let user_db = state.user_db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_localization(
            &AppState {
                shared_db,
                user_db,
            },
            request,
        )
    })
    .await
    .map_err(|error| format!("Localization task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_shared_database_returns_clear_error() {
        let state = AppState {
            shared_db: PathBuf::from("definitely-missing.sqlite"),
            user_db: None,
        };
        let request = RuntimeRequest {
            api_version: "1".into(),
            text: "人工智能".into(),
            source_locale: "zh-CN".into(),
            target_locale: "zh-TW".into(),
            context: None,
        };
        let error = run_localization(&state, request).expect_err("missing DB should fail");
        assert!(error.contains("Shared database not found"));
    }
}
