use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fmt::Debug, io::Read};

use d4ft4::D4FTResult;
use tauri::async_runtime::{channel, Mutex, Receiver, Sender};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FileResponse};
use tokio::fs::File;

#[cfg(target_os = "android")]
mod android;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // console_subscriber::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .manage(State::new())
        .invoke_handler(tauri::generate_handler![
            handle_message,
            receive_response,
            open_file_dialog
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct State {
    sender: Mutex<Option<d4ft4::Sender>>,
    receiver: Mutex<Option<d4ft4::Receiver>>,
    response_tx: Sender<Message<Response>>,
    response_rx: Mutex<Receiver<Message<Response>>>,
    files: Mutex<Vec<LoadedFile>>,
}

impl State {
    fn new() -> Self {
        let (tx, rx) = channel(16);
        Self {
            sender: Mutex::new(None),
            receiver: Mutex::new(None),
            response_tx: tx,
            response_rx: Mutex::new(rx),
            files: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Debug)]
struct LoadedFile {
    name: String,
    handle: FileHandle,
}

#[derive(Debug)]
enum FileHandle {
    Path(PathBuf),
    File(File),
}

impl FileHandle {
    async fn open(&mut self) -> std::io::Result<&mut File> {
        loop {
            match self {
                Self::Path(path) => {
                    *self = Self::File(File::open(path).await?);
                }
                Self::File(f) => break Ok(f),
            }
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
    // #[serde(rename_all = "kebab-case")]
    // Setup {
    //     conn_id: usize,
    //     address: String,
    //     is_server: bool,
    //     mode: d4ft4::TransferMode,
    //     password: String,
    // },
    SetupSender(SetupParams),
    SetupReceiver(SetupParams),
    #[serde(rename_all = "kebab-case")]
    SendText {
        text: String,
    },
    ReceiveText,
    ChooseFile,
    DropFiles {
        names: Vec<String>,
    },
    SendFiles {
        names: Vec<String>,
    },
    ReceiveFileList,
    #[serde(rename_all = "kebab-case")]
    ReceiveFiles {
        allowlist: Vec<String>,
        out_dir: Option<String>,
    },
    // SendFile { conn_id: usize, path: String },
    // ReceiveFile { conn_id: usize, path: String },
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SetupParams {
    address: String,
    is_server: bool,
    password: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "name")]
enum Response {
    SetupComplete(Result<(), String>),
    TextSent(Result<(), String>),
    TextReceived(Result<String, String>),
    FileSelected(Result<String, String>),
    FilesSent(Result<(), String>),
    ReceivedFileList(Result<d4ft4::FileList, String>),
    ReceivedFiles(Result<(), String>),
}

#[tauri::command]
async fn handle_message(
    window: tauri::Window,
    state: tauri::State<'_, State>,
    call: Message<Call>,
) -> Result<(), String> {
    dbg!(&call);
    let message: Option<Response> = match call.message {
        Call::SetupSender(SetupParams {
            address,
            is_server,
            password,
        }) => Some(Response::SetupComplete(
            match d4ft4::init_send(is_server, address, password).await {
                Ok(sender) => {
                    *state.sender.lock().await = Some(sender);
                    Ok(())
                }
                Err(err) => Err(format!("{err:?}")),
            },
        )),
        Call::SetupReceiver(SetupParams {
            address,
            is_server,
            password,
        }) => Some(Response::SetupComplete(
            match d4ft4::init_receive(is_server, address, password).await {
                Ok(receiver) => {
                    *state.receiver.lock().await = Some(receiver);
                    Ok(())
                }
                Err(err) => Err(format!("{err:?}")),
            },
        )),
        Call::SendText { text } => Some(Response::TextSent(
            if let Some(connection) = state.sender.lock().await.as_mut() {
                connection
                    .send_text(text)
                    .await
                    .map_err(|err| format!("{:?}", err))
            } else {
                Err("connection not initialized".to_string())
            },
        )),
        Call::ReceiveText => Some(Response::TextReceived(
            if let Some(connection) = state.receiver.lock().await.as_mut() {
                connection
                    .receive_text()
                    .await
                    .map_err(|err| format!("{:?}", err))
            } else {
                Err("connection not initialized".to_string())
            },
        )),
        Call::ChooseFile => {
            #[cfg(not(target_os = "android"))]
            let handle_pick_file_response = {
                let window = window.clone();
                let return_path = call.return_path.clone();
                move |response: Option<FileResponse>| {
                    let state = window.state::<State>();
                    if let Some(response) = response {
                        let mut files = tauri::async_runtime::block_on(state.files.lock());
                        let message = Response::FileSelected(
                            if let Some(filename) =
                                response.name.and_then(|name| dedup_filename(&name, &files))
                            {
                                files.push(LoadedFile {
                                    name: filename.clone(),
                                    handle: FileHandle::Path(response.path),
                                });
                                Ok(filename)
                            } else {
                                Err("could not find filename".to_string())
                            },
                        );

                        tauri::async_runtime::block_on(state.response_tx.send(Message {
                            return_path: return_path.clone(),
                            message,
                        }))
                        .expect("channel send error");
                    }
                }
            };

            #[cfg(target_os = "android")]
            let handle_pick_file_response = {
                let window = window.clone();
                let return_path = call.return_path.clone();
                move |response: Option<FileResponse>| {
                    if let Some(response) = response {
                        let window_clone = window.clone();
                        let return_path_clone = return_path.clone();
                        let with_webview_result = window.with_webview(move |webview| {
                            android::get_file(
                                webview.jni_handle(),
                                response.path.to_string_lossy().to_string(),
                                "r",
                                move |handle| {
                                    let state = window_clone.state::<State>();
                                    let message = Response::FileSelected(match handle {
                                        Ok(handle) => {
                                            let mut files =
                                                tauri::async_runtime::block_on(state.files.lock());
                                            if let Some(filename) = response
                                                .name
                                                .as_ref()
                                                .and_then(|name| dedup_filename(&name, &files))
                                            {
                                                files.push(LoadedFile {
                                                    name: filename.clone(),
                                                    handle: FileHandle::File(File::from_std(
                                                        handle,
                                                    )),
                                                });
                                                Ok(filename)
                                            } else {
                                                Err("could not find filename".to_string())
                                            }
                                        }
                                        Err(err) => Err(format!("{err:?}")),
                                    });

                                    tauri::async_runtime::block_on(state.response_tx.send(
                                        Message {
                                            return_path: return_path_clone.clone(),
                                            message,
                                        },
                                    ))
                                    .expect("channel send error");
                                },
                            )
                        });
                        if let Err(err) = with_webview_result {
                            let state = window.state::<State>();
                            tauri::async_runtime::block_on(state.response_tx.send(Message {
                                return_path: return_path.clone(),
                                message: Response::FileSelected(Err(format!(
                                    "could not access Android API: {err:?}"
                                ))),
                            }))
                            .expect("channel send error");
                        }
                    }
                }
            };

            window.dialog().file().pick_file(handle_pick_file_response);
            None
        }
        Call::DropFiles { names } => {
            state
                .files
                .lock()
                .await
                .retain(|f| !names.contains(&f.name));
            dbg!(&state.files.lock().await[..]);
            None
        }
        Call::SendFiles { names } => Some(Response::FilesSent({
            let mut files = state.files.lock().await;
            let sending_files = futures::future::try_join_all(
                files
                    .iter_mut()
                    .filter(|f| names.contains(&f.name))
                    .map(|f| async {
                        Ok((
                            PathBuf::from(&f.name),
                            f.handle
                                .open()
                                .await
                                .map_err(|_| format!("could not open file `{:?}`", f.name))?,
                        )) as Result<_, String>
                    }),
            )
            .await;
            match (sending_files, state.sender.lock().await.as_mut()) {
                (Ok(files), Some(sender)) => sender
                    .send_flat_files(files)
                    .await
                    .map_err(|err| format!("{err:?}")),
                (Ok(_), None) => Err("connection not initialized".to_string()),
                (Err(_), _) => Err("could not open file".to_string()),
            }
        })),
        Call::ReceiveFileList => Some(Response::ReceivedFileList({
            match state.receiver.lock().await.as_mut() {
                Some(receiver) => receiver
                    .receive_file_list()
                    .await
                    .map_err(|err| format!("{err:?}")),
                None => Err("connection not initialized".to_string()),
            }
        })),
        Call::ReceiveFiles { allowlist, out_dir } => Some(Response::ReceivedFiles({
            let allowlist = allowlist
                .iter()
                .map(|name| PathBuf::from(name))
                .collect::<Vec<_>>();
            match state.receiver.lock().await.as_mut() {
                Some(receiver) => receiver
                    .receive_flat_files_fs(allowlist, out_dir.as_ref().map(AsRef::as_ref))
                    .await
                    .map_err(|err| format!("{err:?}")),
                None => Err("connection not initialized".to_string()),
            }
        })),
    };

    if let Some(response) = message {
        state
            .response_tx
            .send(Message {
                return_path: call.return_path,
                message: response,
            })
            .await
            .map_err(|_| "channel send error".to_string())
    } else {
        Ok(())
    }
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

fn dedup_filename(filename: &str, files: &[LoadedFile]) -> Option<String> {
    if !files.iter().any(|f| f.name == filename) {
        return Some(filename.to_string());
    }

    let path: &Path = filename.as_ref();
    let stem = path.file_stem()?.to_string_lossy();
    let extension = path
        .extension()
        .map(OsStr::to_string_lossy)
        .unwrap_or_default();
    for i in 1usize.. {
        let new_filename = format!(
            "{stem} ({i}){}{extension}",
            if extension.is_empty() { "" } else { "." }
        );
        if !files.iter().any(|f| f.name == new_filename) {
            return Some(new_filename);
        }
    }
    // If this is reached something has gone very wrong (reached usize max value)
    None
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
async fn open_file_dialog(
    window: tauri::Window,
    app: tauri::AppHandle,
    save: bool,
) -> Result<Option<String>, ()> {
    Ok(if save {
        // app.dialog().file().save_file(|_| ());
        None
    } else {
        app.dialog().file().blocking_pick_file().map(|response| {
            let path = response.path.to_string_lossy().to_string();
            let mut buf = [0u8; 4];
            let (tx, rx) = std::sync::mpsc::channel::<Result<std::fs::File, jni::errors::Error>>();
            window
                .with_webview(|webview| {
                    android::get_file(webview.jni_handle(), path, "r", move |file| {
                        tx.send(file).unwrap()
                    });
                })
                .unwrap();
            let result: Result<_, Box<dyn std::error::Error>> = rx
                .recv()
                .unwrap()
                .map_err(Into::into)
                .and_then(|mut f| f.read_exact(&mut buf).map(|_| f).map_err(Into::into));
            format!(
                "response: {:?}, {}",
                response,
                match result {
                    Ok(f) => format!(
                        "buf: {buf:?}, metadata: {:?}, len: {:?}",
                        f.metadata(),
                        f.metadata().map(|m| m.len())
                    ),
                    Err(err) => format!("error: {err:?}"),
                }
            )
        })
    })
}
