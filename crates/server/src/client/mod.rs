pub mod profile;
pub mod status;
pub mod sync;
pub mod update;

use std::net::SocketAddr;

use flume::{Receiver, Sender};
use serverx_protocol::packet::{ConnectionState, Packet};
use slab::Slab;
use uuid::Uuid;

use crate::{
    client::{profile::Profile, status::ClientStatus},
};

pub struct Clients {
    pub clients: Slab<Client>,
    pub client_lookup: hashbrown::HashMap<Uuid, usize>,
    pub generation: u64,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: Slab::with_capacity(4096),
            client_lookup: hashbrown::HashMap::new(),
            generation: 0,
        }
    }

    pub fn add(
        &mut self,
        addr: SocketAddr,
        outgoing: Sender<Box<dyn Packet>>,
        incoming: Receiver<Box<dyn Packet>>,
        profile: Profile,
    ) -> &mut Client {
        let vacant_entry = self.clients.vacant_entry();
        let key = vacant_entry.key();
        let generation = self.generation;
        self.generation = generation.wrapping_add(1u64);
        self.client_lookup.insert(profile.uuid, key);
        vacant_entry.insert(Client {
            handle: ClientHandle {
                generation,
                slab_key: key,
            },
            status: ClientStatus::Init,
            state: ConnectionState::Configuration,
            addr,
            outgoing,
            incoming,
            profile,
        })
    }

    pub fn get(&self, handle: ClientHandle) -> Option<&Client> {
        if let Some(client) = self.clients.get(handle.slab_key) {
            if client.handle.generation == handle.generation {
                return Some(client);
            }
        }
        None
    }

    pub fn get_mut(&mut self, handle: ClientHandle) -> Option<&mut Client> {
        if let Some(client) = self.clients.get_mut(handle.slab_key) {
            if client.handle.generation == handle.generation {
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
    pub status: ClientStatus,
    pub state: ConnectionState,
    pub addr: SocketAddr,
    pub outgoing: Sender<Box<dyn Packet>>,
    pub incoming: Receiver<Box<dyn Packet>>,
    pub profile: Profile,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClientHandle {
    pub generation: u64,
    pub slab_key: usize,
}
