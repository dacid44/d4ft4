mod encoding;
mod error;
mod protocol;

use std::path::PathBuf;

use tokio::{
    fs,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use encoding::{decode_plaintext, encode_plaintext};
use error::{D4FTError, D4FTResult};

pub use protocol::TransferMode;

pub struct Connection {
    mode: TransferMode,
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

        let handshake = decode_plaintext::<protocol::Handshake, _>(&mut socket).await?;

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
            mode,
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
            mode,
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
                    length: metadata.len(),
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
            length: receiving_length,
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

        if receiving_path != path {
            self.encryptor
                .encode(
                    &protocol::Response::Reject {
                        reason: "unexpected file path".to_string(),
                    },
                    &mut self.socket,
                )
                .await?;
            return Err(D4FTError::RejectedFileTransfer {
                reason: "unexpected file path".to_string(),
            });
        }

        let file = fs::File::open(path.clone())
            .await
            .map_err(|source| D4FTError::FileOpenError { source })?;

        self.decryptor.decode_file(file, &mut self.socket).await
    }

    fn check_mode(&self, mode: TransferMode) -> D4FTResult<()> {
        if self.mode != mode {
            Err(D4FTError::IncorrectTransferMode {
                required: mode,
                actual: self.mode,
            })
        } else {
            Ok(())
        }
    }
}

pub async fn server(password: String, message: Option<String>) -> D4FTResult<Option<String>> {
    match message {
        Some(message) => {
            Connection::listen("127.0.0.1:2581", TransferMode::SendText, password)
                .await?
                .send_text(message)
                .await?;
            Ok(None)
        }
        None => Connection::listen("127.0.0.1:2581", TransferMode::ReceiveText, password)
            .await?
            .receive_text()
            .await
            .map(Some),
    }
}

pub async fn client(password: String, message: Option<String>) -> D4FTResult<Option<String>> {
    match message {
        Some(message) => {
            Connection::connect("127.0.0.1:2581", TransferMode::SendText, password)
                .await?
                .send_text(message)
                .await?;
            Ok(None)
        }
        None => Connection::connect("127.0.0.1:2581", TransferMode::ReceiveText, password)
            .await?
            .receive_text()
            .await
            .map(Some),
    }
}
