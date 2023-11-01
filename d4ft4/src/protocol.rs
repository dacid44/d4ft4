use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Handshake {
    pub(crate) version: String,
    pub(crate) encryption: EncryptionVars,
    pub(crate) is_sender: bool,
    // pub(crate) mode: TransferMode,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "kebab-case")]
pub(crate) struct EncryptionVars {
    #[serde(rename = "client-server-nonce")]
    pub(crate) client_server_nonce: String,
    #[serde(rename = "client-server-salt")]
    pub(crate) client_server_salt: String,
    #[serde(rename = "server-client-nonce")]
    pub(crate) server_client_nonce: String,
    #[serde(rename = "server-client-salt")]
    pub(crate) server_client_salt: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum TransferMode {
    SendText,
    SendFile,
    ReceiveText,
    ReceiveFile,
}

impl TransferMode {
    pub fn corresponding(&self) -> Self {
        match self {
            Self::SendText => Self::ReceiveText,
            Self::SendFile => Self::ReceiveFile,
            Self::ReceiveText => Self::SendText,
            Self::ReceiveFile => Self::SendFile,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "response")]
pub(crate) enum Response {
    Accept,
    Reject { reason: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(crate) enum InitTransfer {
    Text { text: String },
    Files { files: Vec<FileListItem> },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileListItem {
    File { path: PathBuf, size: u64 },
    Directory { path: PathBuf },
}

impl FileListItem {
    pub fn path(&self) -> &Path {
        match self {
            Self::File { path, .. } => path,
            Self::Directory { path } => path,
        }
    }

    pub fn size(&self) -> Option<u64> {
        match self {
            Self::File { size, .. } => Some(*size),
            Self::Directory { .. } => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "response")]
pub(crate) enum FileListResponse {
    Accept { allowlist: Vec<PathBuf> },
    Reject { reason: String },
}

// hashing should be optional
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FileHeader {
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
    pub(crate) hash: Option<String>,
}
