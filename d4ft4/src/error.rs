use thiserror::Error;

use crate::TransferMode;

#[derive(Error, Debug)]
pub enum D4FTError {
    #[error("rejected handshake: {reason}")]
    RejectedHandshake { reason: String },

    #[error("incorrect transfer mode")]
    IncorrectTransferMode {
        required: TransferMode,
        actual: TransferMode,
    },

    #[error("JSON encode error")]
    JsonEncodeError { source: serde_json::Error },

    #[error("write error during encoding")]
    EncodeWriteError { source: std::io::Error },

    #[error("encryption error")]
    EncryptionError { source: aead::Error },

    #[error("JSON decode error")]
    JsonDecodeError { source: serde_json::Error },

    #[error("read error during decoding")]
    DecodeReadError { source: std::io::Error },

    #[error("tried to decode a malformed message: {msg}")]
    MalformedMessage { msg: String },

    #[error("socket error")]
    SocketError { source: std::io::Error },

    #[error("decryption error")]
    DecryptionError { source: aead::Error },

    #[error("hex decode error")]
    HexDecodeError { source: hex::FromHexError },

    #[error("file read error")]
    FileReadError { source: std::io::Error },

    #[error("file write error")]
    FileWriteError { source: std::io::Error },

    #[error("error opening file")]
    FileOpenError { source: std::io::Error },

    #[error("rejected file transfer")]
    RejectedFileTransfer { reason: String },
}

pub type D4FTResult<T> = Result<T, D4FTError>;
