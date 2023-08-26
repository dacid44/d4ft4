mod encoding;
mod error;
mod protocol;

use std::{path::{PathBuf, Path}, ops::Deref, cmp::Ordering};

use protocol::FileListItem;
use tokio::{
    fs,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub use error::{D4FTError, D4FTResult};

pub use protocol::TransferMode;

pub struct Connection {
    stage: TransferStage,
    socket: TcpStream,
    encryptor: encoding::Encryptor,
    decryptor: encoding::Decryptor,
}

impl Connection {
    pub async fn listen<A: ToSocketAddrs>(
        address: A,
        mode: TransferMode,
        password: String,
    ) -> D4FTResult<Self> {
        let (mut socket, _) = TcpListener::bind(address)
            .await
            .map_err(|source| D4FTError::SocketError { source })?
            .accept()
            .await
            .map_err(|source| D4FTError::SocketError { source })?;

        let handshake = encoding::decode_plaintext::<protocol::Handshake, _>(&mut socket).await?;

        let ivs = encoding::InitializationVectors::from_protocol(handshake.encryption)?;
        let (mut encryptor, mut decryptor) = tokio::join!(
            encoding::Encryptor::new(
                password.clone(),
                ivs.server_client_salt,
                &ivs.server_client_nonce
            ),
            encoding::Decryptor::new(password, ivs.client_server_salt, &ivs.client_server_nonce),
        );

        if handshake.version != "4" {
            encryptor
                .encode(
                    &protocol::Response::Reject {
                        reason: "incompatible version".to_string(),
                    },
                    &mut socket,
                )
                .await?;
            return Err(D4FTError::RejectedHandshake {
                reason: "incompatible version".to_string(),
            });
        }

        if handshake.mode != mode.corresponding() {
            encryptor
                .encode(
                    &protocol::Response::Reject {
                        reason: "transfer mode does not match".to_string(),
                    },
                    &mut socket,
                )
                .await?;
            return Err(D4FTError::RejectedHandshake {
                reason: "transfer mode does not match".to_string(),
            });
        }

        encryptor
            .encode(&protocol::Response::Accept, &mut socket)
            .await?;

        Ok(Self {
            stage: TransferStage::from_mode(mode),
            socket,
            encryptor,
            decryptor,
        })
    }

    pub async fn connect<A: ToSocketAddrs>(
        address: A,
        mode: TransferMode,
        password: String,
    ) -> D4FTResult<Self> {
        let mut socket = TcpStream::connect(address)
            .await
            .map_err(|source| D4FTError::SocketError { source })?;

        let ivs = encoding::InitializationVectors::generate();

        encoding::encode_plaintext(
            protocol::Handshake {
                version: "4".to_string(),
                encryption: ivs.to_protocol(),
                mode,
            },
            &mut socket,
        )
        .await?;

        let (mut decryptor, mut encryptor) = tokio::join!(
            encoding::Decryptor::new(
                password.clone(),
                ivs.server_client_salt,
                &ivs.server_client_nonce
            ),
            encoding::Encryptor::new(password, ivs.client_server_salt, &ivs.client_server_nonce),
        );

        if let protocol::Response::Reject { reason } = decryptor
            .decode::<protocol::Response, _>(&mut socket)
            .await?
        {
            return Err(D4FTError::RejectedHandshake { reason });
        }

        Ok(Self {
            stage: TransferStage::from_mode(mode),
            socket,
            encryptor,
            decryptor,
        })
    }

    pub async fn send_text(&mut self, text: String) -> D4FTResult<()> {
        self.check_mode(TransferMode::SendText)?;

        self.encryptor
            .encode(&protocol::SendText(text), &mut self.socket)
            .await
    }

    pub async fn receive_text(&mut self) -> D4FTResult<String> {
        self.check_mode(TransferMode::ReceiveText)?;

        self.decryptor
            .decode::<protocol::SendText, _>(&mut self.socket)
            .await
            .map(|response| response.0)
    }

    pub async fn send_file(&mut self, path: PathBuf) -> D4FTResult<()> {
        self.check_mode(TransferMode::SendFile)?;

        let file = fs::File::open(path.clone())
            .await
            .map_err(|source| D4FTError::FileOpenError { source })?;

        let metadata = file
            .metadata()
            .await
            .map_err(|source| D4FTError::FileOpenError { source })?;

        self.encryptor
            .encode(
                &protocol::SendFile::File {
                    path,
                    size: metadata.len(),
                    hash: None,
                },
                &mut self.socket,
            )
            .await?;

        if let protocol::Response::Reject { reason } = self
            .decryptor
            .decode::<protocol::Response, _>(&mut self.socket)
            .await?
        {
            return Err(D4FTError::RejectedFileTransfer { reason });
        }

        self.encryptor.encode_file(file, &mut self.socket).await
    }

    pub async fn receive_file(&mut self, path: PathBuf) -> D4FTResult<()> {
        self.check_mode(TransferMode::ReceiveFile)?;

        let file_definition = self
            .decryptor
            .decode::<protocol::SendFile, _>(&mut self.socket)
            .await?;

        let protocol::SendFile::File {
            path: receiving_path,
            size: receiving_length,
            hash: receiving_hash
        } = file_definition else {
            self.encryptor
                .encode(
                    &protocol::Response::Reject {
                        reason: "exppected file, got directory".to_string(),
                    },
                    &mut self.socket,
                )
                .await?;
            return Err(D4FTError::RejectedFileTransfer {
                reason: "expected file, got directory".to_string(),
            });
        };

        // if receiving_path != path {
        //     self.encryptor
        //         .encode(
        //             &protocol::Response::Reject {
        //                 reason: "unexpected file path".to_string(),
        //             },
        //             &mut self.socket,
        //         )
        //         .await?;
        //     return Err(D4FTError::RejectedFileTransfer {
        //         reason: "unexpected file path".to_string(),
        //     });
        // }

        self.encryptor.encode(&protocol::Response::Accept, &mut self.socket).await?;

        let file = fs::File::create(path.clone())
            .await
            .map_err(|source| D4FTError::FileOpenError { source })?;

        self.decryptor.decode_file(file, &mut self.socket).await
    }

    /// Recursively send files from the given paths.
    pub async fn prepare_send_files<P: Deref<Target = Path>>(&mut self, paths: &[P]) -> D4FTResult<()> {
        self.check_mode(TransferMode::SendFile)?;
        // check for existing prepare
        let stored_paths = match &mut self.stage {
            TransferStage::SendFile(paths) => if paths.is_none() { paths } else {
                return Err(D4FTError::ExistingFileTransferPrepared);
            }
            _ => return Err(D4FTError::IncorrectTransferMode { required: TransferMode::SendFile, actual: self.stage.to_mode() }),
        };



        todo!()
    }

    fn check_mode(&self, mode: TransferMode) -> D4FTResult<()> {
        let actual_mode = self.stage.to_mode();
        if actual_mode != mode {
            Err(D4FTError::IncorrectTransferMode {
                required: mode,
                actual: actual_mode,
            })
        } else {
            Ok(())
        }
    }
}

enum TransferStage {
    SendText,
    SendFile(Option<Vec<protocol::FileListItem>>),
    ReceiveText,
    ReceiveFile(Option<Vec<protocol::FileListItem>>),
}

impl TransferStage {
    fn from_mode(mode: TransferMode) -> Self {
        match mode {
            TransferMode::SendText => Self::SendText,
            TransferMode::SendFile => Self::SendFile(None),
            TransferMode::ReceiveText => Self::ReceiveText,
            TransferMode::ReceiveFile => Self::ReceiveFile(None),
        }
    }

    fn to_mode(&self) -> TransferMode {
        match self {
            Self::SendText => TransferMode::SendText,
            Self::SendFile(_) => TransferMode::SendFile,
            Self::ReceiveText => TransferMode::ReceiveText,
            Self::ReceiveFile(_) => TransferMode::ReceiveFile,
        }
    }
}

impl PartialOrd for FileListItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileListItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_path = self.path();
        let other_path = other.path();
        
        if self_path == other_path {
            return match (self, other) {
                (Self::Directory(_), Self::File { .. }) => Ordering::Less,
                (Self::File { .. }, Self::Directory(_)) => Ordering::Greater,
                (Self::Directory(_), Self::Directory(_)) => Ordering::Equal,
                (Self::File { size: self_size, .. }, Self::File { size: other_size, .. }) => self_size.cmp(other_size),
            };
        }

        self_path.cmp(other_path)
    }
}