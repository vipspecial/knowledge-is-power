mod ai;
mod library;
mod mcp;
mod models;
mod settings;

use ai::{abort_ai_stream, list_ai_models, stream_ai, test_ai_connection};
use library::{
    choose_document_directory, export_markdown, import_markdown, load_store, save_store,
};
use mcp::{get_mcp_setup_info, set_mcp_enabled};
use settings::{clear_ai_api_key, load_settings, save_settings};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

const MAIN_WINDOW_LABEL: &str = "main";

/// Run the bundled executable as a read-only stdio MCP server.
pub fn run_mcp(args: &[String]) -> Result<(), String> {
    mcp::run(args)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            settings::migrate_legacy_app_data(app.handle()).map_err(std::io::Error::other)?;
            setup_tray(app)?;
            setup_close_to_tray(app)?;
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
            get_mcp_setup_info,
            set_mcp_enabled,
            stream_ai,
            abort_ai_stream,
            test_ai_connection,
            list_ai_models
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Orange Run Notes");
}

/// 创建系统托盘：左键单击恢复窗口，右键菜单提供「显示主窗口 / 退出」。
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let mut tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("拿了桔子跑啊")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 关闭主窗口时隐藏到托盘而不是退出，避免误关丢上下文；退出走托盘菜单。
fn setup_close_to_tray(app: &tauri::App) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };
    let app_handle = app.handle().clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.hide();
                // 提示用户应用仍在托盘运行，避免误以为已退出。
                let _ = app_handle.emit("window-hidden-to-tray", ());
            }
        }
    });
    Ok(())
}
