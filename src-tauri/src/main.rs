// 防止 Windows 发布版启动时附带控制台窗口，关闭该窗口会终止整个应用，勿删。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--mcp") {
        if let Err(error) = orange_run_notes_lib::run_mcp(&arguments[1..]) {
            eprintln!("MCP 启动失败：{error}");
            std::process::exit(1);
        }
        return;
    }
    orange_run_notes_lib::run();
}
