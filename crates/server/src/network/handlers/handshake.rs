use std::fmt::{Debug, Display, Formatter};

use serverx_protocol::{
    io::{AsyncPacketReader, PacketReadErr},
    packet::{ConnectionState::Handshake, PacketDirection::ServerBound},
    v765::{
        serverbound::HandshakeRequest, types::HandshakeNextState, PacketDecoderImpl, PROTO_VER,
    },
};
use tokio::net::TcpStream;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct HandshakeResult {
    pub next: HandshakeNextState,
}

pub enum HandshakeErr {
    ReadErr(PacketReadErr),
    MismatchedVersion(i32, i32),
    UnexpectedPacket,
}

impl Debug for HandshakeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeErr::ReadErr(err) => write!(f, "read error: {:?}", err),
            HandshakeErr::MismatchedVersion(client_ver, server_ver) => write!(
                f,
                "client protocol version {} does not match server version {}",
                client_ver, server_ver
            ),
            HandshakeErr::UnexpectedPacket => write!(f, "unexpected packet"),
        }
    }
}

#[instrument(skip_all)]
pub async fn handle_handshake(
    socket: &mut TcpStream,
    reader: &mut AsyncPacketReader,
) -> Result<HandshakeResult, HandshakeErr> {
    let packet = reader
        .read::<TcpStream, PacketDecoderImpl>(socket, ServerBound, Handshake)
        .await
        .map_err(|err| HandshakeErr::ReadErr(err))?;
    if let Ok(handshake) = packet.into_any().downcast::<HandshakeRequest>() {
        return if handshake.version != PROTO_VER {
            tracing::debug!(
                client_ver = handshake.version,
                server_vec = PROTO_VER,
                "protocol version mismatch"
            );
            Err(HandshakeErr::MismatchedVersion(
                handshake.version,
                PROTO_VER,
            ))
        } else {
            Ok(HandshakeResult {
                next: handshake.next_state,
            })
        };
    }
    Err(HandshakeErr::UnexpectedPacket)
}
