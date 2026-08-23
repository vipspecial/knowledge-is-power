// 防止 Windows 发布版启动时附带控制台窗口，关闭该窗口会终止整个应用，勿删。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    orange_run_notes_lib::run();
}
