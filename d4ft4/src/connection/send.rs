use crate::connection::Connection;
use crate::encoding::{Decryptor, Encryptor};
use crate::{protocol, D4FTError, D4FTResult, FileList, FileListItem};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;

pub struct Sender {
    socket: TcpStream,
    encryptor: Encryptor,
    decryptor: Decryptor,
}

impl Connection for Sender {
    const IS_SENDER: bool = true;
    fn init(socket: TcpStream, encryptor: Encryptor, decryptor: Decryptor) -> Self {
        Self {
            socket,
            encryptor,
            decryptor,
        }
    }
}

impl Sender {
    pub async fn send_text(&mut self, text: String) -> D4FTResult<()> {
        self.encryptor
            .encode(&protocol::InitTransfer::Text(text), &mut self.socket)
            .await?;

        self.accept_response().await
    }

    /// Send files, without any directory structure. This function will trim file paths down to only the file name.
    pub async fn send_flat_files(&mut self, files: Vec<(PathBuf, &mut File)>) -> D4FTResult<()> {
        let file_list = FileList::from_items(
            futures::future::try_join_all(files.iter().map(|(path, f)| async {
                Ok(FileListItem::File {
                    path: path
                        .file_name()
                        .ok_or_else(|| D4FTError::CannotReadPath { path: path.clone() })
                        .map(Into::into)?,
                    size: f
                        .metadata()
                        .await
                        .map_err(|source| D4FTError::FileOpenError { source })?
                        .len(),
                }) as D4FTResult<FileListItem>
            }))
            .await?,
        );

        let mut allowlist = self.prepare_send_files(file_list.clone()).await?;
        allowlist.sort();

        for (handle, item) in files
            .into_iter()
            .map(|f| f.1)
            .zip(file_list.list.into_iter())
            .filter(|(_, item)| {
                allowlist
                    .binary_search_by_key(&item.path(), |p| p.as_ref())
                    .is_ok()
            })
        {
            if let FileListItem::File { path, size } = item {
                self.send_file(handle, path, size).await?;
            }
        }

        // TODO: Handle missing/corrupted files (optional)
        Ok(())
    }

    async fn prepare_send_files(&mut self, file_list: FileList) -> D4FTResult<Vec<PathBuf>> {
        self.encryptor
            .encode(&protocol::InitTransfer::Files(file_list), &mut self.socket)
            .await?;

        let response = self
            .decryptor
            .decode::<protocol::FileListResponse, _>(&mut self.socket)
            .await?;

        match response {
            protocol::FileListResponse::Accept(allowlist) => Ok(allowlist),
            protocol::FileListResponse::Reject { reason } => {
                Err(D4FTError::RejectedTransfer { reason })
            }
        }
    }

    async fn send_file(&mut self, handle: &mut File, path: PathBuf, size: u64) -> D4FTResult<()> {
        self.encryptor
            .encode(
                &protocol::FileHeader {
                    path,
                    size,
                    hash: None,
                },
                &mut self.socket,
            )
            .await?;

        handle
            .seek(std::io::SeekFrom::Start(0))
            .await
            .map_err(|source| D4FTError::FileReadError { source })?;
        self.encryptor.encode_file(handle, &mut self.socket).await
    }

    async fn accept_response(&mut self) -> D4FTResult<()> {
        let response = self
            .decryptor
            .decode::<protocol::Response, _>(&mut self.socket)
            .await?;

        match response {
            protocol::Response::Accept => Ok(()),
            protocol::Response::Reject { reason } => Err(D4FTError::RejectedTransfer { reason }),
        }
    }
}
