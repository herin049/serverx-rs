use std::net::SocketAddr;

use flume::{Receiver, Sender};
use serverx_protocol::{
    io::{AsyncPacketReader, AsyncPacketWriter, PacketReadErr},
    packet::{
        ConnectionState, Packet,
        PacketDirection::{ClientBound, ServerBound},
    },
    v765::{
        clientbound::{ServerFinishConfiguration, StartConfiguration},
        serverbound::{ClientFinishConfiguration, ConfigurationAck},
        types::HandshakeNextState,
        PacketDecoderImpl, PacketEncoderImpl,
    },
};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

use crate::network::{
    event::NetworkEvent,
    handlers::{handshake::handle_handshake, login::handle_login, status::handle_status},
};

pub fn spawn_write_loop(
    mut state: ConnectionState,
    mut sock: OwnedWriteHalf,
    mut writer: AsyncPacketWriter,
    packets: Receiver<Box<dyn Packet>>,
) {
    tokio::spawn(async move {
        'outer: loop {
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
                            break 'outer;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
}

pub fn spawn_read_loop(
    mut state: ConnectionState,
    mut sock: OwnedReadHalf,
    mut reader: AsyncPacketReader,
    packets: Sender<Box<dyn Packet>>,
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
                    if let PacketReadErr::IoErr(_) = err {
                        break;
                    } else {
                        tracing::warn!(?err, "error while reading packet");
                    }
                }
            }
        }
    });
}

pub async fn accept_client(mut socket: TcpStream, addr: SocketAddr, events: Sender<NetworkEvent>) {
    let _ = socket.set_nodelay(true);
    let mut reader = AsyncPacketReader::new();
    let mut writer = AsyncPacketWriter::new();
    let handshake_result = match handle_handshake(&mut socket, &mut reader).await {
        Ok(result) => result,
        Err(error) => {
            tracing::debug!(?error, "unable to process handshake request");
            return;
        }
    };
    match handshake_result.next {
        HandshakeNextState::Status => {
            match handle_status(&mut socket, &mut reader, &mut writer).await {
                Ok(()) => {
                    tracing::trace!("successfully handled status request");
                }
                Err(err) => {
                    tracing::error!(?err, "unable to handle status request");
                }
            }
        }
        HandshakeNextState::Login => {
            match handle_login(&mut socket, &mut reader, &mut writer).await {
                Ok(login_result) => {
                    tracing::trace!("successfully handled connect request");
                    let (outgoing_tx, outgoing_rx) = flume::unbounded::<Box<dyn Packet>>();
                    let (incoming_tx, incoming_rx) = flume::unbounded::<Box<dyn Packet>>();
                    let (sock_read, sock_write) = socket.into_split();
                    spawn_read_loop(
                        ConnectionState::Configuration,
                        sock_read,
                        reader,
                        incoming_tx,
                    );
                    spawn_write_loop(
                        ConnectionState::Configuration,
                        sock_write,
                        writer,
                        outgoing_rx,
                    );
                    if let Err(_) = events.send(NetworkEvent::Connected {
                        addr,
                        outgoing: outgoing_tx,
                        incoming: incoming_rx,
                        profile: login_result.profile,
                    }) {
                        tracing::error!("error while sending connected event to queue")
                    }
                }
                Err(err) => {
                    tracing::error!(?err, "unable to handle connect");
                }
            }
        }
    }
}
