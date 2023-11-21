use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fmt::Debug, io::Read};

use d4ft4::{Connection, D4FTError, D4FTResult};
use futures::{future, FutureExt, TryFutureExt};
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
        .invoke_handler(tauri::generate_handler![handle_message, receive_response,])
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
#[serde(tag = "name", content = "content")]
enum Response {
    SetupComplete,
    TextSent,
    TextReceived(String),
    FileSelected(String),
    FilesSent,
    ReceivedFileList(Vec<d4ft4::FileListItem>),
    ReceivedFiles,
    Error(String),
}

impl From<Result<Response, String>> for Response {
    fn from(value: Result<Response, String>) -> Self {
        match value {
            Ok(response) => response,
            Err(message) => Response::Error(message),
        }
    }
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
        }) => Some(match d4ft4::init_send(is_server, address, password).await {
            Ok(sender) => {
                *state.sender.lock().await = Some(sender);
                dbg!(Response::SetupComplete)
            }
            Err(err) => dbg!(Response::Error(format!("{err:?}"))),
        }),
        Call::SetupReceiver(SetupParams {
            address,
            is_server,
            password,
        }) => Some({
            let mut receiver_lock = state.receiver.lock().await;
            // drop the existing receiver so we don't get "address already in use"
            // TODO: Figure out how to do this if the sender was listening before
            *receiver_lock = None;
            match d4ft4::init_receive(is_server, address, password).await {
                Ok(receiver) => {
                    *receiver_lock = Some(receiver);
                    Response::SetupComplete
                }
                Err(err) => Response::Error(format!("{err:?}")),
            }
        }),
        Call::SendText { text } => Some(
            with_locked_conn(&state.sender, |sender| {
                sender
                    .send_text(text)
                    .map_ok(|_| Response::TextSent)
                    .boxed()
            })
            .await,
        ),
        Call::ReceiveText => Some(
            with_locked_conn(&state.receiver, |receiver| {
                receiver
                    .receive_text()
                    .map_ok(Response::TextReceived)
                    .boxed()
            })
            .await,
        ),
        Call::ChooseFile => {
            #[cfg(not(target_os = "android"))]
            let handle_pick_file_response = {
                let window = window.clone();
                let return_path = call.return_path.clone();
                move |response: Option<FileResponse>| {
                    let state = window.state::<State>();
                    if let Some(response) = response {
                        let mut files = tauri::async_runtime::block_on(state.files.lock());
                        let message = if let Some(filename) =
                            response.name.and_then(|name| dedup_filename(&name, &files))
                        {
                            files.push(LoadedFile {
                                name: filename.clone(),
                                handle: FileHandle::Path(response.path),
                            });
                            Response::FileSelected(filename)
                        } else {
                            Response::Error("could not find filename".to_string())
                        };

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
                                    let message = match handle {
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
                                                Response::FileSelected(filename)
                                            } else {
                                                Response::Error(
                                                    "could not find filename".to_string(),
                                                )
                                            }
                                        }
                                        Err(err) => Response::Error(format!("{err:?}")),
                                    };

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
                                message: Response::Error(format!(
                                    "could not access Android API: {err:?}"
                                )),
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
        Call::SendFiles { names } => Some({
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
                    .map(|_| Response::FilesSent)
                    .map_err(|err| format!("{err:?}")),
                (Ok(_), None) => Err("connection not initialized".to_string()),
                (Err(_), _) => Err("could not open file".to_string()),
            }
            .into()
        }),
        Call::ReceiveFileList => Some(
            with_locked_conn(&state.receiver, |receiver| {
                receiver
                    .receive_file_list()
                    .map_ok(Response::ReceivedFileList)
                    .boxed()
            })
            .await,
        ),
        Call::ReceiveFiles { allowlist, out_dir } => Some({
            let allowlist = allowlist
                .iter()
                .map(|name| PathBuf::from(name))
                .collect::<Vec<_>>();

            with_locked_conn(&state.receiver, |receiver| {
                async move {
                    receiver
                        .receive_flat_files_fs(allowlist, out_dir.as_ref().map(AsRef::as_ref))
                        .await
                        .map(|_| Response::ReceivedFiles)
                }
                .boxed()
            })
            .await
        }),
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

/// Calls an async function on the contained connection handle, and handles converting the result.
async fn with_locked_conn<Conn, Op>(conn: &Mutex<Option<Conn>>, op: Op) -> Response
where
    Conn: Connection,
    Op: FnOnce(&mut Conn) -> future::BoxFuture<'_, D4FTResult<Response>>,
{
    let mut conn_lock = conn.lock().await;
    let Some(conn_handle) = conn_lock.as_mut() else {
        return Response::Error("connection not initialized".to_string());
    };

    op(conn_handle)
        .await
        .map_err(|err| format!("{err:?}"))
        .into()
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
