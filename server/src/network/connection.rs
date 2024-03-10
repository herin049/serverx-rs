use flume::{Receiver, RecvError, Sender};
use serverx_core::protocol::{
    io::{AsyncPacketReader, AsyncPacketWriter, PacketReadErr, PacketWriteErr},
    packet::{
        ConnectionState, Packet,
        PacketDirection::{ClientBound, ServerBound},
    },
    v765::{
        clientbound::{ServerFinishConfiguration, StartConfiguration},
        serverbound::{ClientFinishConfiguration, ConfigurationAck},
        PacketDecoderImpl, PacketEncoderImpl,
    },
};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

use crate::{client::ClientHandle, network::event::NetEvent};

pub fn spawn_write_loop(
    client: ClientHandle,
    mut state: ConnectionState,
    mut sock: OwnedWriteHalf,
    mut writer: AsyncPacketWriter,
    packets: Receiver<Box<dyn Packet>>,
    events: Sender<NetEvent>,
) {
    tokio::spawn(async move {
        loop {
            match packets.recv_async().await {
                Ok(packet) => {
                    let write_result = writer
                        .write::<OwnedWriteHalf, PacketEncoderImpl>(
                            &mut sock,
                            ClientBound,
                            state,
                            packet.as_ref(),
                        )
                        .await;
                    match write_result {
                        Ok(()) => {
                            if state == ConnectionState::Configuration
                                && packet.id() == ServerFinishConfiguration::ID
                            {
                                state = ConnectionState::Play;
                            } else if state == ConnectionState::Play
                                && packet.id() == StartConfiguration::ID
                            {
                                state = ConnectionState::Configuration;
                            }
                        }
                        Err(_) => {
                            let _ = events.send(NetEvent::Disconnected { client });
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
}

pub fn spawn_read_loop(
    client: ClientHandle,
    mut state: ConnectionState,
    mut sock: OwnedReadHalf,
    mut reader: AsyncPacketReader,
    packets: Sender<Box<dyn Packet>>,
    events: Sender<NetEvent>,
) {
    tokio::spawn(async move {
        loop {
            let read_result = reader
                .read::<OwnedReadHalf, PacketDecoderImpl>(&mut sock, ServerBound, state)
                .await;
            match read_result {
                Ok(packet) => {
                    tracing::trace!(?packet, "read packet");
                    if state == ConnectionState::Configuration
                        && packet.id() == ClientFinishConfiguration::ID
                    {
                        state = ConnectionState::Play;
                    } else if state == ConnectionState::Play && packet.id() == ConfigurationAck::ID
                    {
                        state = ConnectionState::Configuration;
                    }
                    if packets.send(packet).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    tracing::debug!(?err, "error while reading packet");
                    // let _ = events.send(NetEvent::Disconnected { client });
                    break;
                }
            }
        }
    });
}
