use thiserror::Error;

#[derive(Error, Debug)]
pub enum D4FTError {
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

    #[error("Wrong encryption mode: {msg}")]
    WrongEncryptionMode { msg: String },

    #[error("Hex decode error")]
    HexDecodeError { source: hex::FromHexError },
}

pub type D4FTResult<T> = Result<T, D4FTError>;