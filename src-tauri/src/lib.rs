mod ai;
mod library;
mod models;
mod settings;

use ai::{list_ai_models, stream_ai, test_ai_connection};
use library::{
    choose_document_directory, export_markdown, import_markdown, load_store, save_store,
};
use settings::{clear_ai_api_key, load_settings, save_settings};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Config-based maximizing can be ignored during the first macOS
            // window layout. Reapply it after the webview window is created.
            #[cfg(desktop)]
            if let Some(window) = app.get_webview_window("main") {
                window.maximize()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_store,
            save_store,
            choose_document_directory,
            import_markdown,
            export_markdown,
            load_settings,
            save_settings,
            clear_ai_api_key,
            stream_ai,
            test_ai_connection,
            list_ai_models
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Orange Run Notes");
}
