// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![add, server, client])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn add(a: i32, b: i32) -> i32 {
    d4ft4::add(a, b)
}

#[tauri::command]
async fn server(password: String, message: Option<String>) -> Option<String> {
    match d4ft4::server(password, message).await {
        Ok(msg) => msg,
        Err(e) => Some(format!("Error: {}", e)),
    }
}

#[tauri::command]
async fn client(password: String, message: Option<String>) -> Option<String> {
    match d4ft4::client(password, message).await {
        Ok(msg) => msg,
        Err(e) => Some(format!("Error: {}", e)),
    }
}
