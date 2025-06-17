// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod server;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Returns the HTTP server port.
#[tauri::command]
fn server_port() -> u16 {
    server::port()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            tauri::async_runtime::spawn(async {
                if let Err(e) = server::start().await {
                    eprintln!("failed to start server: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, server_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
