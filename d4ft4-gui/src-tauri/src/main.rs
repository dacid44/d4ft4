// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::async_runtime::Mutex;

fn main() {
    tauri::Builder::default()
        .manage(Connections::new())
        .invoke_handler(tauri::generate_handler![server, client, setup_server])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct Connections {
    server: Mutex<Option<d4ft4::Connection>>,
    client: Mutex<Option<d4ft4::Connection>>,
}

impl Connections {
    fn new() -> Self {
        Self {
            server: Mutex::new(None),
            client: Mutex::new(None),
        }
    }
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

#[tauri::command]
async fn setup_server(connections: tauri::State<'_, Connections>, mode: d4ft4::TransferMode, password: String) -> Result<Option<String>, ()> {
    Ok(match d4ft4::Connection::listen("127.0.0.1:2581", mode, password).await {
        Ok(connection) => {
            *connections.server.lock().await = Some(connection);
            None
        },
        Err(error) => Some(format!("{}", error))
    })
}

#[tauri::command]
async fn setup_client(connections: tauri::State<'_, Connections>, mode: d4ft4::TransferMode, password: String) -> Result<Option<String>, ()> {
    Ok(match d4ft4::Connection::connect("127.0.0.1:2581", mode, password).await {
        Ok(connection) => {
            *connections.client.lock().await = Some(connection);
            None
        },
        Err(error) => Some(format!("{}", error))
    })
}
