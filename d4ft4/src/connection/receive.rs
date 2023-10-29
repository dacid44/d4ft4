use crate::connection::Connection;
use crate::encoding::{Decryptor, Encryptor};
use crate::{protocol, D4FTError, D4FTResult, FileList};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::net::TcpStream;

pub struct Receiver {
    socket: TcpStream,
    encryptor: Encryptor,
    decryptor: Decryptor,
}

impl Connection for Receiver {
    const IS_SENDER: bool = false;
    fn init(socket: TcpStream, encryptor: Encryptor, decryptor: Decryptor) -> Self {
        Self {
            socket,
            encryptor,
            decryptor,
        }
    }
}

impl Receiver {
    pub async fn receive_text(&mut self) -> D4FTResult<String> {
        let transfer = self
            .decryptor
            .decode::<protocol::InitTransfer, _>(&mut self.socket)
            .await?;

        match transfer {
            protocol::InitTransfer::Text(text) => {
                self.encryptor
                    .encode(&protocol::Response::Accept, &mut self.socket)
                    .await?;
                Ok(text)
            }
            protocol::InitTransfer::Files(_) => {
                let reason = "got files, wanted text".to_string();
                self.encryptor
                    .encode(
                        &protocol::Response::Reject {
                            reason: reason.clone(),
                        },
                        &mut self.socket,
                    )
                    .await?;
                Err(D4FTError::RejectedTransfer { reason })
            }
        }
    }

    pub async fn receive_file_list(&mut self) -> D4FTResult<FileList> {
        let transfer = self
            .decryptor
            .decode::<protocol::InitTransfer, _>(&mut self.socket)
            .await?;

        match transfer {
            protocol::InitTransfer::Text(_) => {
                let reason = "got text, wanted files".to_string();
                self.encryptor
                    .encode(
                        &protocol::Response::Reject {
                            reason: reason.clone(),
                        },
                        &mut self.socket,
                    )
                    .await?;
                Err(D4FTError::RejectedTransfer { reason })
            }
            protocol::InitTransfer::Files(file_list) => Ok(file_list),
        }
    }

    pub async fn receive_flat_files_fs(
        &mut self,
        mut allowlist: Vec<PathBuf>,
        out_dir: Option<&Path>,
    ) -> D4FTResult<()> {
        self.accept_files(allowlist.clone()).await?;

        allowlist.sort();

        let out_dir = out_dir.unwrap_or(".".as_ref());

        while !allowlist.is_empty() {
            let file_header = self
                .decryptor
                .decode::<protocol::FileHeader, _>(&mut self.socket)
                .await?;

            if allowlist.contains(&file_header.path) {
                let handle =
                    File::create(out_dir.join(file_header.path.file_name().ok_or_else(|| {
                        D4FTError::CannotReadPath {
                            path: file_header.path.clone(),
                        }
                    })?))
                    .await
                    .map_err(|source| D4FTError::FileWriteError { source })?;
                self.decryptor.decode_file(handle, &mut self.socket).await?;
            } else {
                self.decryptor
                    .decode_file(tokio::io::sink(), &mut self.socket)
                    .await?;
            }
        }

        Ok(())
    }

    async fn accept_files(&mut self, allowlist: Vec<PathBuf>) -> D4FTResult<()> {
        self.encryptor
            .encode(
                &protocol::FileListResponse::Accept(allowlist),
                &mut self.socket,
            )
            .await
    }
}
