mod protocol;
mod encoding;
mod error;

use tokio::net::{TcpListener, TcpStream};

use error::{D4FTResult, D4FTError};
use encoding::{decode_plaintext, encode_plaintext};


pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

pub async fn server(password: String, message: String) -> D4FTResult<String> {
    // let mac_length = dbg!(<<chacha20poly1305::XChaCha20Poly1305 as aead::AeadCore>::CiphertextOverhead as typenum::marker_traits::Unsigned>::to_usize());
    // let mut message_bytes = message.clone().bytes().collect::<Vec<_>>();
    // println!("message length: {}", message_bytes.len());
    // let cipher = chacha20poly1305::XChaCha20Poly1305::new(&[0u8; 32].into());
    // cipher.encrypt_in_place(&[0; 24].into(), b"D4FTD4FT", &mut message_bytes);
    // dbg!(&message_bytes);
    // dbg!(String::from_utf8_lossy(&message_bytes));
    // println!("message length after cipher: {}", message_bytes.len());

    let listener = TcpListener::bind("127.0.0.1:2581").await
        .map_err(|source| D4FTError::SocketError { source })?;

    let (mut socket, _) = listener.accept().await
        .map_err(|source| D4FTError::SocketError { source })?;

    let handshake = decode_plaintext::<protocol::Handshake, _>(&mut socket).await?;

    if handshake.version != "4"
        || handshake.mode != protocol::TransferMode::SendText
    {
        encode_plaintext(protocol::HandshakeResponse::Reject { reason: "invalid handshake".to_string() }, &mut socket).await?;
        return Ok("invalid handshake".to_string());
    }

    let ivs = encoding::InitializationVectors::from_protocol(handshake.encryption)?;
    let mut encryptor = encoding::Encryptor::new(&password, &ivs.server_client_salt, &ivs.server_client_nonce);

    let received_message = handshake.message;

    encryptor.encode(protocol::HandshakeResponse::Accept { message }, &mut socket).await?;

    Ok(received_message)
}

pub async fn client(password: String, message: String) -> D4FTResult<String> {
    let mut socket = TcpStream::connect("127.0.0.1:2581").await
        .map_err(|source| D4FTError::SocketError { source })?;

    let mut ivs = encoding::InitializationVectors::generate();

    encode_plaintext(protocol::Handshake {
        version: "4".to_string(),
        encryption: ivs.to_protocol(),
        mode: protocol::TransferMode::SendText,
        message
    }, &mut socket).await?;

    let mut decryptor = encoding::Decryptor::new(&password, &ivs.server_client_salt, &ivs.server_client_nonce);

    let response = decryptor.decode::<protocol::HandshakeResponse, _>(&mut socket).await?;

    match response {
        protocol::HandshakeResponse::Accept { message } => Ok(message),
        protocol::HandshakeResponse::Reject { reason } => Ok(format!("handshake rejected: {reason}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
