use crate::{encoding, protocol, D4FTError, D4FTResult};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

mod receive;
mod send;

pub use receive::Receiver;
pub use send::Sender;

trait Connection {
    const IS_SENDER: bool;
    fn init(
        socket: TcpStream,
        encryptor: encoding::Encryptor,
        decryptor: encoding::Decryptor,
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

async fn init_listen<A: ToSocketAddrs, Conn: Connection>(
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
    let (mut encryptor, mut decryptor) = tokio::join!(
        encoding::Encryptor::new(
            password.clone(),
            ivs.server_client_salt,
            &ivs.server_client_nonce
        ),
        encoding::Decryptor::new(password, ivs.client_server_salt, &ivs.client_server_nonce),
    );

    if handshake.version != "4" {
        encryptor
            .encode(
                &protocol::Response::Reject {
                    reason: "incompatible version".to_string(),
                },
                &mut socket,
            )
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
            .encode(
                &protocol::Response::Reject {
                    reason: reason.clone(),
                },
                &mut socket,
            )
            .await?;
        return Err(D4FTError::RejectedHandshake { reason });
    }

    encryptor
        .encode(&protocol::Response::Accept, &mut socket)
        .await?;

    Ok(Conn::init(socket, encryptor, decryptor))
}

async fn init_connect<A: ToSocketAddrs, Conn: Connection>(
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

    let (mut decryptor, mut encryptor) = tokio::join!(
        encoding::Decryptor::new(
            password.clone(),
            ivs.server_client_salt,
            &ivs.server_client_nonce
        ),
        encoding::Encryptor::new(password, ivs.client_server_salt, &ivs.client_server_nonce),
    );

    if let protocol::Response::Reject { reason } = decryptor
        .decode::<protocol::Response, _>(&mut socket)
        .await?
    {
        return Err(D4FTError::RejectedHandshake { reason });
    }

    Ok(Conn::init(socket, encryptor, decryptor))
}
