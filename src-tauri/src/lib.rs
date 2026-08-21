mod ai;
mod library;
mod models;
mod settings;

use ai::{stream_ai, test_ai_connection};
use library::{
    choose_document_directory, export_markdown, import_markdown, load_store, save_store,
};
use settings::{clear_ai_api_key, load_settings, save_settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            test_ai_connection
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Mojian Notes");
}
