use crate::{encoding, protocol, D4FTError, D4FTResult};
use tokio::net::{tcp, TcpListener, TcpStream, ToSocketAddrs};

mod receive;
mod send;

pub use receive::Receiver;
pub use send::Sender;

pub trait Connection {}

trait InitConnection: Connection {
    const IS_SENDER: bool;
    fn init(
        encryptor: encoding::Encryptor<tcp::OwnedWriteHalf>,
        decryptor: encoding::Decryptor<tcp::OwnedReadHalf>,
    ) -> Self;
}

pub async fn init_send<A: ToSocketAddrs>(
    listen: bool,
    address: A,
    password: String,
) -> D4FTResult<Sender> {
    if listen {
        init_listen(address, password).await
    } else {
        init_connect(address, password).await
    }
}

pub async fn init_receive<A: ToSocketAddrs>(
    listen: bool,
    address: A,
    password: String,
) -> D4FTResult<Receiver> {
    if listen {
        init_listen(address, password).await
    } else {
        init_connect(address, password).await
    }
}

async fn init_listen<A: ToSocketAddrs, Conn: InitConnection>(
    address: A,
    password: String,
) -> D4FTResult<Conn> {
    let (mut socket, _) = TcpListener::bind(address)
        .await
        .map_err(|source| D4FTError::SocketError { source })?
        .accept()
        .await
        .map_err(|source| D4FTError::SocketError { source })?;

    let handshake = encoding::decode_plaintext::<protocol::Handshake, _>(&mut socket).await?;

    let ivs = encoding::InitializationVectors::from_protocol(handshake.encryption)?;
    let (rx_sock, tx_sock) = socket.into_split();
    let (mut encryptor, decryptor) = tokio::join!(
        encoding::Encryptor::new(
            password.clone(),
            ivs.server_client_salt,
            &ivs.server_client_nonce,
            tx_sock,
        ),
        encoding::Decryptor::new(
            password,
            ivs.client_server_salt,
            &ivs.client_server_nonce,
            rx_sock
        ),
    );

    if handshake.version != "4" {
        encryptor
            .encode(&protocol::Response::Reject {
                reason: "incompatible version".to_string(),
            })
            .await?;
        return Err(D4FTError::RejectedHandshake {
            reason: "incompatible version".to_string(),
        });
    }

    if handshake.is_sender == Conn::IS_SENDER {
        let reason = format!(
            "both ends are {}",
            if handshake.is_sender {
                "sender"
            } else {
                "receiver"
            }
        );
        encryptor
            .encode(&protocol::Response::Reject {
                reason: reason.clone(),
            })
            .await?;
        return Err(D4FTError::RejectedHandshake { reason });
    }

    encryptor.encode(&protocol::Response::Accept).await?;

    Ok(Conn::init(encryptor, decryptor))
}

async fn init_connect<A: ToSocketAddrs, Conn: InitConnection>(
    address: A,
    password: String,
) -> D4FTResult<Conn> {
    let mut socket = TcpStream::connect(address)
        .await
        .map_err(|source| D4FTError::SocketError { source })?;

    let ivs = encoding::InitializationVectors::generate();

    encoding::encode_plaintext(
        protocol::Handshake {
            version: "4".to_string(),
            encryption: ivs.to_protocol(),
            is_sender: Conn::IS_SENDER,
        },
        &mut socket,
    )
    .await?;

    let (tx_sock, rx_sock) = socket.into_split();
    let (mut decryptor, encryptor) = tokio::join!(
        encoding::Decryptor::new(
            password.clone(),
            ivs.server_client_salt,
            &ivs.server_client_nonce,
            tx_sock,
        ),
        encoding::Encryptor::new(
            password,
            ivs.client_server_salt,
            &ivs.client_server_nonce,
            rx_sock
        ),
    );

    if let protocol::Response::Reject { reason } = decryptor.decode::<protocol::Response>().await? {
        return Err(D4FTError::RejectedHandshake { reason });
    }

    Ok(Conn::init(encryptor, decryptor))
}
