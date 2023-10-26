use std::{error::Error, fmt::Debug, io::Read};

use d4ft4::D4FTResult;
use tauri::async_runtime::{channel, Mutex, Receiver, Sender};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[cfg(target_os = "android")]
mod android;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // console_subscriber::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(State::new())
        .invoke_handler(tauri::generate_handler![handle_message, receive_response, open_file_dialog])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct State {
    connections: [Mutex<Option<d4ft4::Connection>>; 2],
    response_tx: Sender<Message<Response>>,
    response_rx: Mutex<Receiver<Message<Response>>>,
}

impl State {
    fn new() -> Self {
        let (tx, rx) = channel(16);
        Self {
            connections: [Mutex::new(None), Mutex::new(None)],
            response_tx: tx,
            response_rx: Mutex::new(rx),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Message<T> {
    return_path: Vec<String>,
    message: T,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "name", content = "args")]
enum Call {
    #[serde(rename_all = "kebab-case")]
    Setup {
        conn_id: usize,
        address: String,
        is_server: bool,
        mode: d4ft4::TransferMode,
        password: String,
    },
    #[serde(rename_all = "kebab-case")]
    SendText { conn_id: usize, text: String },
    #[serde(rename_all = "kebab-case")]
    ReceiveText { conn_id: usize },
    // SendFile { conn_id: usize, path: String },
    // ReceiveFile { conn_id: usize, path: String },
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "name")]
enum Response {
    SetupComplete(Result<(), String>),
    TextSent(Result<(), String>),
    TextReceived(Result<String, String>),
}

#[tauri::command]
async fn handle_message(
    state: tauri::State<'_, State>,
    call: Message<Call>,
) -> Result<(), String> {
    dbg!(&call);
    let response = Message {
        return_path: call.return_path,
        message: match call.message {
            Call::Setup {
                conn_id,
                address,
                is_server,
                mode,
                password,
            } => {
                let connection = if is_server {
                    d4ft4::Connection::listen(address, mode, password).await
                } else {
                    d4ft4::Connection::connect(address, mode, password).await
                };
                Response::SetupComplete(match connection {
                    Ok(connection) => {
                        *state.connections[conn_id].lock().await = Some(connection);
                        Ok(())
                    }
                    Err(err) => Err(format!("{:?}", err)),
                })
            }
            Call::SendText { conn_id, text } => Response::TextSent(
                if let Some(connection) = state.connections[conn_id].lock().await.as_mut() {
                    connection
                        .send_text(text)
                        .await
                        .map_err(|err| format!("{:?}", err))
                } else {
                    Err("connection not initialized".to_string())
                },
            ),
            Call::ReceiveText { conn_id } => Response::TextReceived(
                if let Some(connection) = state.connections[conn_id].lock().await.as_mut() {
                    connection
                        .receive_text()
                        .await
                        .map_err(|err| format!("{:?}", err))
                } else {
                    Err("connection not initialized".to_string())
                },
            ),
        },
    };
    state.response_tx.send(response).await
        .map_err(|_| "channel send error".to_string())
}

#[tauri::command]
async fn receive_response(state: tauri::State<'_, State>) -> Result<Message<Response>, String> {
    state
        .response_rx
        .lock()
        .await
        .recv()
        .await
        .ok_or("channel closed".to_string())
}

// #[tauri::command]
// async fn send_file(
//     connections: tauri::State<'_, State>,
//     conn_id: usize,
//     path: PathBuf,
// ) -> Result<(usize, Option<String>), ()> {
//     println!("send_file conn_id={conn_id}");
//     match &mut *connections.0[conn_id].lock().await {
//         Some(connection) => handle_error(connection.send_file(path).await.map(|_| None), conn_id),
//         None => Ok((
//             conn_id,
//             Some("Error: connection not initialized".to_string()),
//         )),
//     }
// }

// #[tauri::command]
// async fn receive_file(
//     connections: tauri::State<'_, State>,
//     conn_id: usize,
//     path: PathBuf,
// ) -> Result<(usize, Option<String>), ()> {
//     println!("send_file conn_id={conn_id}");
//     match &mut *connections.0[conn_id].lock().await {
//         Some(connection) => {
//             handle_error(connection.receive_file(path).await.map(|_| None), conn_id)
//         }
//         None => Ok((
//             conn_id,
//             Some("Error: connection not initialized".to_string()),
//         )),
//     }
// }

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

#[cfg(not(target_os = "android"))]
#[tauri::command]
async fn open_file_dialog(app: tauri::AppHandle, save: bool) -> Result<Option<String>, ()> {
    Ok(if save {
        // app.dialog().file().save_file(|_| ());
        None
    } else {
        app.dialog().file().blocking_pick_file().map(|response| {
            let path = response.path;
            let mut buf = [0u8; 4];
            let result =
                std::fs::File::open(path.clone().join(response.name.clone().unwrap_or_default()))
                    .and_then(|mut f| f.read_exact(&mut buf));
            format!(
                "path: {:?}, name: {:?}, {}",
                path,
                response.name,
                match result {
                    Ok(_) => format!("{buf:?}"),
                    Err(err) => format!("{err:?}"),
                }
            )
        })
    })
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn open_file_dialog(window: tauri::Window, app: tauri::AppHandle, save: bool) -> Result<Option<String>, ()> {
    Ok(if save {
        // app.dialog().file().save_file(|_| ());
        None
    } else {
        app.dialog().file().blocking_pick_file().map(|response| {
            let path = response.path.to_string_lossy().to_string();
            let mut buf = [0u8; 4];
            let (tx, rx) = std::sync::mpsc::channel::<Result<std::fs::File, jni::errors::Error>>();
            window.with_webview(|webview| {
                android::get_file(webview.jni_handle(), path, "r", move |file| tx.send(file).unwrap() );
            }).unwrap();
            let result: Result<_, Box<dyn std::error::Error>> = rx.recv().unwrap()
                .map_err(Into::into)
                .and_then(|mut f| f.read_exact(&mut buf).map(|_| f).map_err(Into::into));
            format!(
                "response: {:?}, {}",
                response,
                match result {
                    Ok(f) => format!("buf: {buf:?}, metadata: {:?}, len: {:?}", f.metadata(), f.metadata().map(|m| m.len())),
                    Err(err) => format!("error: {err:?}"),
                }
            )
        })
    })
}