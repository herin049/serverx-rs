use std::net::SocketAddr;

use flume::Sender;
use serverx_core::protocol::{
    io::{AsyncPacketReader, AsyncPacketWriter},
    v765::types::HandshakeNextState,
};
use tokio::net::{TcpListener, TcpStream};
use tracing::instrument;

use crate::network::{
    event::NetEvent,
    handlers::{handshake::handle_handshake, login::handle_login, status::handle_status},
};

pub fn accept_client(mut socket: TcpStream, addr: SocketAddr, event_queue: Sender<NetEvent>) {
    let _ = socket.set_nodelay(true);
    tokio::spawn(async move {
        let mut reader = AsyncPacketReader::new();
        let mut writer = AsyncPacketWriter::new();
        let handshake_result = match handle_handshake(&mut socket, &mut reader, &mut writer).await {
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
                        if let Err(_) = event_queue.send(NetEvent::Connected {
                            socket,
                            addr,
                            reader,
                            writer,
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
    });
}
