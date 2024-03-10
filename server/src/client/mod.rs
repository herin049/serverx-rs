pub mod profile;
pub mod state;

use std::net::SocketAddr;

use flume::{Receiver, Sender};
pub use profile::*;
use serverx_core::protocol::packet::Packet;
use uuid::Uuid;
use serverx_slab::Slab;

use crate::client::state::ClientState;

pub struct Clients {
    pub clients: Slab<Client>,
    pub client_id: u64,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: Slab::with_capacity(4096),
            client_id: 0,
        }
    }

    pub fn add(
        &mut self,
        addr: SocketAddr,
        outgoing: Sender<Box<dyn Packet>>,
        incoming: Receiver<Box<dyn Packet>>,
        uuid: Uuid,
        username: String,
    ) -> &mut Client {
        let entry = self.clients.vacant_entry();
        let key = entry.key();
        let client_id = self.client_id;
        self.client_id = client_id.wrapping_add(1u64);
        entry.insert(Client {
            handle: ClientHandle {
                id: client_id,
                slab_key: key,
            },
            state: ClientState::Init,
            addr,
            outgoing,
            incoming,
            uuid,
            username,
        })
    }

    pub fn get(&self, handle: ClientHandle) -> Option<&Client> {
        if let Some(client) = self.clients.get(handle.slab_key) {
            if client.handle.id == handle.id {
                return Some(client);
            }
        }
        None
    }

    pub fn get_mut(&mut self, handle: ClientHandle) -> Option<&mut Client> {
        if let Some(client) = self.clients.get_mut(handle.slab_key) {
            if client.handle.id == handle.id {
                return Some(client);
            }
        }
        None
    }

    pub fn remove(&mut self, handle: ClientHandle) -> Option<Client> {
        let should_remove = self.get_mut(handle).is_some();
        if should_remove {
            self.clients.try_remove(handle.slab_key)
        } else {
            None
        }
    }
}

pub struct Client {
    pub handle: ClientHandle,
    pub state: ClientState,
    pub addr: SocketAddr,
    pub outgoing: Sender<Box<dyn Packet>>,
    pub incoming: Receiver<Box<dyn Packet>>,
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClientHandle {
    pub id: u64,
    pub slab_key: usize,
}
