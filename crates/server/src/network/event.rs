use std::net::SocketAddr;

use flume::{Receiver, Sender};
use serverx_protocol::packet::Packet;

use crate::client::profile::Profile;

pub enum NetworkEvent {
    Connected {
        addr: SocketAddr,
        outgoing: Sender<Box<dyn Packet>>,
        incoming: Receiver<Box<dyn Packet>>,
        profile: Profile,
    },
}
