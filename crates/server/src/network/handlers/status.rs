use std::fmt::{Debug, Formatter};

use serde_json::json;
use tokio::net::TcpStream;
use tracing::instrument;
use serverx_protocol::io::{AsyncPacketReader, AsyncPacketWriter, PacketReadErr, PacketWriteErr};
use serverx_protocol::packet::ConnectionState::Status;
use serverx_protocol::packet::PacketDirection::{ClientBound, ServerBound};
use serverx_protocol::v765::clientbound::{StatusPingResponse, StatusResponse};
use serverx_protocol::v765::{PacketDecoderImpl, PacketEncoderImpl};
use serverx_protocol::v765::serverbound::{StatusPingRequest, StatusRequest};
use serverx_protocol::v765::{PROTO_NAME, PROTO_VER};
pub enum StatusErr {
    WriteErr(PacketWriteErr),
    ReadErr(PacketReadErr),
    UnexpectedPacket,
}

impl Debug for StatusErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusErr::WriteErr(err) => write!(f, "write error: {}", err),
            StatusErr::ReadErr(err) => write!(f, "read error: {}", err),
            StatusErr::UnexpectedPacket => write!(f, "unexpected packet"),
        }
    }
}

#[instrument(skip_all)]
pub async fn handle_status(
    socket: &mut TcpStream,
    reader: &mut AsyncPacketReader,
    writer: &mut AsyncPacketWriter,
) -> Result<(), StatusErr> {
    loop {
        let packet = reader
            .read::<TcpStream, PacketDecoderImpl>(socket, ServerBound, Status)
            .await
            .map_err(|err| StatusErr::ReadErr(err))?;
        match packet.id() {
            StatusRequest::ID => {
                let status = packet
                    .as_any()
                    .downcast_ref::<StatusRequest>()
                    .ok_or_else(|| StatusErr::UnexpectedPacket)?;
                tracing::trace!(packet = ?status, "received status request packet");
                let response = StatusResponse {
                    response: json!({
                        "version": {
                            "name": PROTO_NAME,
                            "protocol": PROTO_VER
                        },
                        "players": {
                            "max": 0,
                            "online": 0
                        },
                        "description": {
                            "text": "Hello world!"
                        }
                    }),
                };
                tracing::trace!(packet = ?response, "writing status response packet");
                writer
                    .write::<TcpStream, PacketEncoderImpl>(socket, ClientBound, Status, &response)
                    .await
                    .map_err(|err| StatusErr::WriteErr(err))?;
            }
            StatusPingRequest::ID => {
                let ping = packet
                    .as_any()
                    .downcast_ref::<StatusPingRequest>()
                    .ok_or_else(|| StatusErr::UnexpectedPacket)?;
                tracing::trace!(packet = ?ping, "received ping request packet");
                let pong = StatusPingResponse {
                    payload: ping.payload,
                };
                tracing::trace!(packet = ?pong, "writing pong packet");
                writer
                    .write::<TcpStream, PacketEncoderImpl>(socket, ClientBound, Status, &pong)
                    .await
                    .map_err(|err| StatusErr::WriteErr(err))?;
                return Ok(());
            }
            _ => return Err(StatusErr::UnexpectedPacket),
        }
    }
}
