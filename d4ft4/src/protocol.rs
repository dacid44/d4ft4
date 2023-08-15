use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Handshake {
    pub(crate) version: String,
    pub(crate) encryption: EncryptionVars,
    pub(crate) mode: TransferMode,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(crate) enum EncryptionVars {
    Plaintext,
    XChaChaPoly1305Psk {
        #[serde(rename = "client-server-nonce")]
        client_server_nonce: String,
        #[serde(rename = "client-server-salt")]
        client_server_salt: String,
        #[serde(rename = "server-client-nonce")]
        server_client_nonce: String,
        #[serde(rename = "server-client-salt")]
        server_client_salt: String,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum TransferMode {
    SendText,
    SendFile,
    ReceiveText,
    ReceiveFile,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "response")]
pub(crate) enum HandshakeResponse {
    Accept,
    Reject { reason: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SendText(pub(crate) String);