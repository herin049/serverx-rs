use std::net::SocketAddr;

use serverx_core::protocol::io::{AsyncPacketReader, AsyncPacketWriter};
use tokio::net::TcpStream;

use crate::{
    client::{ClientHandle, Profile},
    server::Server,
};

pub enum NetEvent {
    Connected {
        socket: TcpStream,
        addr: SocketAddr,
        reader: AsyncPacketReader,
        writer: AsyncPacketWriter,
        profile: Profile,
    },
    Disconnected {
        client: ClientHandle,
    },
}

pub fn process_net_events(server: &mut Server) {
    while let Ok(event) = server.net_recv.try_recv() {
        match event {
            NetEvent::Connected {
                socket,
                addr,
                reader,
                writer,
                profile,
            } => {
                let (mut sock_read, mut sock_write) = socket.into_split();
            }
            NetEvent::Disconnected { client } => {}
        }
    }
}
