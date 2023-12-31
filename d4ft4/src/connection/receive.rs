use crate::connection::{Connection, InitConnection};
use crate::encoding::{Decryptor, Encryptor};
use crate::{protocol, D4FTError, D4FTResult, FileListItem};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::net::tcp;

pub struct Receiver {
    encryptor: Encryptor<tcp::OwnedWriteHalf>,
    decryptor: Decryptor<tcp::OwnedReadHalf>,
}

impl Connection for Receiver {}

impl InitConnection for Receiver {
    const IS_SENDER: bool = false;
    fn init(
        encryptor: Encryptor<tcp::OwnedWriteHalf>,
        decryptor: Decryptor<tcp::OwnedReadHalf>,
    ) -> Self {
        Self {
            encryptor,
            decryptor,
        }
    }
}

impl Receiver {
    pub async fn receive_text(&mut self) -> D4FTResult<String> {
        let transfer = self.decryptor.decode::<protocol::InitTransfer>().await?;

        match transfer {
            protocol::InitTransfer::Text { text } => {
                self.encryptor.encode(&protocol::Response::Accept).await?;
                Ok(text)
            }
            protocol::InitTransfer::Files { .. } => {
                let reason = "got files, wanted text".to_string();
                self.encryptor
                    .encode(&protocol::Response::Reject {
                        reason: reason.clone(),
                    })
                    .await?;
                Err(D4FTError::RejectedTransfer { reason })
            }
        }
    }

    pub async fn receive_file_list(&mut self) -> D4FTResult<Vec<FileListItem>> {
        let transfer = self.decryptor.decode::<protocol::InitTransfer>().await?;

        match transfer {
            protocol::InitTransfer::Text { .. } => {
                let reason = "got text, wanted files".to_string();
                self.encryptor
                    .encode(&protocol::Response::Reject {
                        reason: reason.clone(),
                    })
                    .await?;
                Err(D4FTError::RejectedTransfer { reason })
            }
            protocol::InitTransfer::Files { files } => Ok(files),
        }
    }

    pub async fn receive_flat_files_fs(
        &mut self,
        mut allowlist: Vec<PathBuf>,
        out_dir: Option<&Path>,
    ) -> D4FTResult<()> {
        println!("receive_files start");
        self.accept_files(allowlist.clone()).await?;

        allowlist.sort();

        let out_dir = out_dir.unwrap_or(".".as_ref());

        println!("receive_files setup done");

        while !allowlist.is_empty() {
            let file_header = self.decryptor.decode::<protocol::FileHeader>().await?;

            println!("got a file header: {:?}", &file_header);

            if allowlist.contains(&file_header.path) {
                println!("receiving file");
                let handle =
                    File::create(out_dir.join(file_header.path.file_name().ok_or_else(|| {
                        D4FTError::CannotReadPath {
                            path: file_header.path.clone(),
                        }
                    })?))
                    .await
                    .map_err(|source| D4FTError::FileWriteError { source })?;
                self.decryptor.decode_file(handle).await?;
            } else {
                println!("ignoring file");
                self.decryptor.decode_file(tokio::io::sink()).await?;
            }
        }

        Ok(())
    }

    async fn accept_files(&mut self, allowlist: Vec<PathBuf>) -> D4FTResult<()> {
        self.encryptor
            .encode(&protocol::FileListResponse::Accept { allowlist })
            .await
    }
}
