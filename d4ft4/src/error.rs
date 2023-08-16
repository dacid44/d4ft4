use thiserror::Error;

use crate::TransferMode;

#[derive(Error, Debug)]
pub enum D4FTError {
    #[error("Rejected handshake: {reason}")]
    RejectedHandshake { reason: String },

    #[error("Incorrect transfer mode")]
    IncorrectTransferMode {
        required: TransferMode,
        actual: TransferMode,
    },

    #[error("JSON encode error")]
    JsonEncodeError { source: serde_json::Error },

    #[error("Write error during encoding")]
    EncodeWriteError { source: std::io::Error },

    #[error("Encryption error")]
    EncryptionError { source: aead::Error },

    #[error("JSON decode error")]
    JsonDecodeError { source: serde_json::Error },

    #[error("Read error during decoding")]
    DecodeReadError { source: std::io::Error },

    #[error("Tried to decode a malformed message: {msg}")]
    MalformedMessage { msg: String },

    #[error("Socket Error")]
    SocketError { source: std::io::Error },

    #[error("Decryption error")]
    DecryptionError { source: aead::Error },

    #[error("Hex decode error")]
    HexDecodeError { source: hex::FromHexError },

    #[error("File read error")]
    FileReadError { source: std::io::Error },

    #[error("File write error")]
    FileWriteError { source: std::io::Error },

    #[error("Error opening file")]
    FileOpenError { source: std::io::Error },

    #[error("Rejected file transfer")]
    RejectedFileTransfer { reason: String },
}

pub type D4FTResult<T> = Result<T, D4FTError>;
