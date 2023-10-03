use std::{error::Error, path::PathBuf};

use d4ft4::D4FTResult;
use tauri::async_runtime::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // console_subscriber::init();

    tauri::Builder::default()
        .manage(Connections::new())
        .invoke_handler(tauri::generate_handler![
            setup,
            send_text,
            receive_text,
            send_file,
            receive_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct Connections([Mutex<Option<d4ft4::Connection>>; 2]);

impl Connections {
    fn new() -> Self {
        Self([Mutex::new(None), Mutex::new(None)])
    }
}

#[tauri::command]
async fn setup(
    connections: tauri::State<'_, Connections>,
    address: String,
    conn_id: usize,
    is_server: bool,
    mode: d4ft4::TransferMode,
    password: String,
) -> Result<(usize, Option<String>), ()> {
    let connection = if is_server {
        d4ft4::Connection::listen(address, mode, password).await
    } else {
        d4ft4::Connection::connect(address, mode, password).await
    };

    Ok(match connection {
        Ok(connection) => {
            *connections.0[conn_id].lock().await = Some(connection);
            (conn_id, None)
        }
        Err(error) => (
            conn_id,
            Some(format!("Error: {}, source: {:?}", error, error.source())),
        ),
    })
}

#[tauri::command]
async fn send_text(
    connections: tauri::State<'_, Connections>,
    conn_id: usize,
    text: String,
) -> Result<(usize, Option<String>), ()> {
    match &mut *connections.0[conn_id].lock().await {
        Some(connection) => handle_error(connection.send_text(text).await.map(|_| None), conn_id),
        None => Ok((
            conn_id,
            Some("Error: connection not initialized".to_string()),
        )),
    }
}

#[tauri::command]
async fn receive_text(
    connections: tauri::State<'_, Connections>,
    conn_id: usize,
) -> Result<(usize, String), ()> {
    match &mut *connections.0[conn_id].lock().await {
        Some(connection) => Ok((
            conn_id,
            match connection.receive_text().await {
                Ok(text) => text,
                Err(error) => format!("Error: {}, source: {:?}", error, error.source()),
            },
        )),
        None => Ok((conn_id, "Error: connection not initialized".to_string())),
    }
}

#[tauri::command]
async fn send_file(
    connections: tauri::State<'_, Connections>,
    conn_id: usize,
    path: PathBuf,
) -> Result<(usize, Option<String>), ()> {
    println!("send_file conn_id={conn_id}");
    match &mut *connections.0[conn_id].lock().await {
        Some(connection) => handle_error(connection.send_file(path).await.map(|_| None), conn_id),
        None => Ok((
            conn_id,
            Some("Error: connection not initialized".to_string()),
        )),
    }
}

#[tauri::command]
async fn receive_file(
    connections: tauri::State<'_, Connections>,
    conn_id: usize,
    path: PathBuf,
) -> Result<(usize, Option<String>), ()> {
    println!("send_file conn_id={conn_id}");
    match &mut *connections.0[conn_id].lock().await {
        Some(connection) => {
            handle_error(connection.receive_file(path).await.map(|_| None), conn_id)
        }
        None => Ok((
            conn_id,
            Some("Error: connection not initialized".to_string()),
        )),
    }
}

fn handle_error(
    result: d4ft4::D4FTResult<Option<String>>,
    conn_id: usize,
) -> Result<(usize, Option<String>), ()> {
    match result {
        Ok(text) => Ok((conn_id, text)),
        Err(error) => Ok((
            conn_id,
            Some(format!("Error: {}, source: {:?}", error, error.source())),
        )),
    }
}
