mod commands;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::development_defaults())
        .invoke_handler(tauri::generate_handler![commands::localize_text])
        .run(tauri::generate_context!())
        .expect("error while running Chinese Regional Localizer");
}
