use std::io::{Read, Write};

use aead::rand_core::{RngCore, SeedableRng};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{D4FTError, D4FTResult};

const POLY1305_MAC_LENGTH: u64 = 16;

pub(crate) async fn encode_plaintext<T: Serialize, W: AsyncWriteExt + Unpin>(
    data: T,
    mut writer: W,
) -> D4FTResult<()> {
    let mut bytes = b"D4FT\0\0\0\0\0\0\0\0".to_vec();

    data.serialize(&mut serde_json::Serializer::new(&mut bytes))
        .map_err(|source| D4FTError::JsonEncodeError { source })?;

    let num_bytes = bytes.len() as u64 - 12;
    bytes[4..12].copy_from_slice(&num_bytes.to_be_bytes());

    writer
        .write_all(&bytes)
        .await
        .map_err(|source| D4FTError::EncodeWriteError { source })
}

pub(crate) async fn decode_plaintext<T: DeserializeOwned, R: AsyncReadExt + Unpin>(
    mut reader: R,
) -> D4FTResult<T> {
    let mut tag = [0u8; 4];
    reader
        .read_exact(&mut tag)
        .await
        .map_err(|source| D4FTError::DecodeReadError { source })?;
    if tag != *b"D4FT" {
        return Err(D4FTError::MalformedMessage {
            msg: "did not find 'D4FT' header tag".to_string(),
        });
    }

    let mut num_bytes = [0u8; 8];
    reader
        .read_exact(&mut num_bytes)
        .await
        .map_err(|source| D4FTError::DecodeReadError { source })?;
    let num_bytes = u64::from_be_bytes(num_bytes) as usize;

    let mut bytes = vec![0u8; num_bytes];
    reader
        .read_exact(&mut bytes)
        .await
        .map_err(|source| D4FTError::DecodeReadError { source })?;

    serde_json::from_slice(&bytes).map_err(|source| D4FTError::JsonDecodeError { source })
}

pub(crate) struct InitializationVectors {
    pub(crate) client_server_nonce: [u8; 19],
    pub(crate) client_server_salt: [u8; 32],
    pub(crate) server_client_nonce: [u8; 19],
    pub(crate) server_client_salt: [u8; 32],
}

impl InitializationVectors {
    pub(crate) fn generate() -> Self {
        let mut rng = rand_chacha::ChaCha20Rng::from_entropy();

        let mut ivs = InitializationVectors {
            client_server_nonce: [0u8; 19],
            client_server_salt: [0u8; 32],
            server_client_nonce: [0u8; 19],
            server_client_salt: [0u8; 32],
        };

        rng.fill_bytes(&mut ivs.client_server_nonce);
        rng.fill_bytes(&mut ivs.client_server_salt);
        rng.fill_bytes(&mut ivs.server_client_nonce);
        rng.fill_bytes(&mut ivs.server_client_salt);

        ivs
    }

    pub(crate) fn from_protocol(vars: crate::protocol::EncryptionVars) -> D4FTResult<Self> {
        match vars {
            crate::protocol::EncryptionVars::Plaintext => Err(D4FTError::WrongEncryptionMode {
                msg: "tried to retrieve encryption vars from plaintext encryption mode".to_string(),
            }),
            crate::protocol::EncryptionVars::XChaChaPoly1305Psk {
                client_server_nonce,
                client_server_salt,
                server_client_nonce,
                server_client_salt,
            } => {
                let mut ivs = InitializationVectors {
                    client_server_nonce: [0u8; 19],
                    client_server_salt: [0u8; 32],
                    server_client_nonce: [0u8; 19],
                    server_client_salt: [0u8; 32],
                };

                hex::decode_to_slice(client_server_nonce, &mut ivs.client_server_nonce)
                    .map_err(|source| D4FTError::HexDecodeError { source })?;
                hex::decode_to_slice(client_server_salt, &mut ivs.client_server_salt)
                    .map_err(|source| D4FTError::HexDecodeError { source })?;
                hex::decode_to_slice(server_client_nonce, &mut ivs.server_client_nonce)
                    .map_err(|source| D4FTError::HexDecodeError { source })?;
                hex::decode_to_slice(server_client_salt, &mut ivs.server_client_salt)
                    .map_err(|source| D4FTError::HexDecodeError { source })?;

                Ok(ivs)
            }
        }
    }

    pub(crate) fn to_protocol(&self) -> crate::protocol::EncryptionVars {
        crate::protocol::EncryptionVars::XChaChaPoly1305Psk {
            client_server_nonce: hex::encode_upper(self.client_server_nonce),
            client_server_salt: hex::encode_upper(self.client_server_salt),
            server_client_nonce: hex::encode_upper(self.server_client_nonce),
            server_client_salt: hex::encode_upper(self.server_client_salt),
        }
    }
}

fn derive_key(password: &str, salt: &[u8; 32]) -> [u8; 32] {
    let mut key = [69u8; 32];
    println!("starting key derive");
    scrypt::scrypt(
        password.as_bytes(),
        salt,
        &scrypt::Params::new(16, 8, 1, 32).expect("Scrypt should not error on hardcoded params"),
        &mut key,
    )
    .expect("Scrypt should not error on hardcoded output length");
    println!("key derive done");
    key
}

pub(crate) struct Encryptor {
    encryptor: aead::stream::EncryptorBE32<chacha20poly1305::XChaCha20Poly1305>,
}

impl Encryptor {
    pub(crate) fn new(password: &str, salt: &[u8; 32], nonce: &[u8; 19]) -> Self {
        let key = derive_key(password, salt);
        Self {
            encryptor: aead::stream::EncryptorBE32::new(
                &derive_key(password, salt).into(),
                nonce.into(),
            ),
        }
    }

    pub(crate) async fn encode<T: Serialize, W: AsyncWriteExt + Unpin>(
        &mut self,
        data: T,
        mut writer: W,
    ) -> D4FTResult<()> {
        let mut bytes =
            serde_json::to_vec(&data).map_err(|source| D4FTError::JsonEncodeError { source })?;

        let mut header = [0u8; 12];
        header[0..4].copy_from_slice(b"D4FT");
        header[4..12]
            .copy_from_slice(&(bytes.len() as u64 + POLY1305_MAC_LENGTH).to_be_bytes());

        self.encryptor
            .encrypt_next_in_place(&header, &mut bytes)
            .map_err(|source| D4FTError::EncryptionError { source })?;

        bytes.splice(0..0, header);

        writer
            .write_all(&bytes)
            .await
            .map_err(|source| D4FTError::EncodeWriteError { source })
    }
}

pub(crate) struct Decryptor {
    decryptor: aead::stream::DecryptorBE32<chacha20poly1305::XChaCha20Poly1305>,
}

impl Decryptor {
    pub(crate) fn new(password: &str, salt: &[u8; 32], nonce: &[u8; 19]) -> Self {
        Self {
            decryptor: aead::stream::DecryptorBE32::new(
                &derive_key(password, salt).into(),
                nonce.into(),
            ),
        }
    }

    pub(crate) async fn decode<T: DeserializeOwned, R: AsyncReadExt + Unpin>(
        &mut self,
        mut reader: R,
    ) -> D4FTResult<T> {
        let mut header = [0u8; 12];
        reader
            .read_exact(&mut header[0..4])
            .await
            .map_err(|source| D4FTError::DecodeReadError { source })?;
        if header[0..4] != *b"D4FT" {
            return Err(D4FTError::MalformedMessage {
                msg: "did not find 'D4FT' header tag".to_string(),
            });
        }

        let mut num_bytes = [0u8; 8];
        reader
            .read_exact(&mut num_bytes)
            .await
            .map_err(|source| D4FTError::DecodeReadError { source })?;
        header[4..12].copy_from_slice(&num_bytes);
        let num_bytes = u64::from_be_bytes(num_bytes) as usize;

        let mut bytes = vec![0u8; num_bytes];
        reader
            .read_exact(&mut bytes)
            .await
            .map_err(|source| D4FTError::DecodeReadError { source })?;

        self.decryptor
            .decrypt_next_in_place(&header, &mut bytes)
            .map_err(|source| D4FTError::DecryptionError { source })?;

        serde_json::from_slice(&bytes).map_err(|source| D4FTError::JsonDecodeError { source })
    }
}
