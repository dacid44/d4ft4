use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Handshake {
    pub(crate) version: String,
    pub(crate) encryption: EncryptionVars,
    pub(crate) mode: TransferMode,
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
pub(crate) struct SendText(pub(crate) String);

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FileList {
    list: Vec<FileListItem>,
    total_size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum FileListItem {
    File {
        path: PathBuf,
        size: u64,
    },
    Directory(PathBuf),
}

// hashing should be optional
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SendFile {
    File {
        path: PathBuf,
        size: u64,
        hash: Option<String>,
    },
    Directory(PathBuf),
}
