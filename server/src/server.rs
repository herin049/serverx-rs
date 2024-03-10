use std::{
    cell::RefCell,
    net::{IpAddr, SocketAddr},
    rc::Rc,
    str::FromStr,
    time::Duration,
};

use flume::{Receiver, Sender};
use serverx_core::protocol::packet::{ConnectionState, Packet};
use tokio::{runtime::Runtime, time, time::MissedTickBehavior};
use tracing::instrument;

use crate::{
    client::{
        state::{update_client, ClientState},
        Clients,
    },
    config::ServerConfig,
    network::{
        connection::{spawn_read_loop, spawn_write_loop},
        event::NetEvent,
        listen::listen,
    },
    resources::Resources,
};

pub struct Server {
    pub clients: Clients,
    pub config: ServerConfig,
    pub resources: Resources,
    pub net_send: Sender<NetEvent>,
    pub net_recv: Receiver<NetEvent>,
}

impl Server {
    pub fn new(config: ServerConfig, resources: Resources) -> Self {
        let (net_send, net_recv) = flume::unbounded::<NetEvent>();
        Self {
            clients: Clients::new(),
            config,
            resources,
            net_send,
            net_recv,
        }
    }

    pub fn process_net_events(&mut self) {
        while let Ok(event) = self.net_recv.try_recv() {
            match event {
                NetEvent::Connected {
                    socket,
                    addr,
                    reader,
                    writer,
                    profile,
                } => {
                    let (mut sock_read, mut sock_write) = socket.into_split();
                    let (out_send, out_recv) = flume::unbounded::<Box<dyn Packet>>();
                    let (in_send, in_recv) = flume::unbounded::<Box<dyn Packet>>();
                    let client =
                        self.clients
                            .add(addr, out_send, in_recv, profile.uuid, profile.name);
                    tracing::trace!("starting client write loop");
                    spawn_write_loop(
                        client.handle,
                        ConnectionState::Configuration,
                        sock_write,
                        writer,
                        out_recv,
                        self.net_send.clone(),
                    );
                    tracing::trace!("starting client write loop");
                    spawn_read_loop(
                        client.handle,
                        ConnectionState::Configuration,
                        sock_read,
                        reader,
                        in_send,
                        self.net_send.clone(),
                    );
                }
                NetEvent::Disconnected {
                    client: client_handle,
                } => {
                    if let Some(client) = self.clients.get_mut(client_handle) {
                        tracing::trace!(username = client.username, "client disconnected");
                        client.state = ClientState::Disconnecting;
                    }
                }
            }
        }
    }

    pub fn update_clients(&mut self) {
        for i in 0..4096 {
            let (handle, state) = if let Some(client) = self.clients.clients.get(i) {
                (client.handle, client.state)
            } else {
                continue;
            };
            if let Some(next_state) = update_client(self, handle, state) {
                if let Some(client) = self.clients.get_mut(handle) {
                    client.state = next_state;
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn tick(&mut self) {
        self.process_net_events();
        self.update_clients();
    }

    #[instrument(skip_all)]
    pub fn start(mut self) {
        let rt = Runtime::new().unwrap();
        let _enter_guard = rt.enter();
        let listener_addr = SocketAddr::new(
            IpAddr::from_str(self.config.ip.as_str()).expect("invalid ip address"),
            self.config.port,
        );
        listen(listener_addr, self.net_send.clone());
        rt.block_on(async move {
            loop {
                let mut interval = time::interval(Duration::from_millis(50));
                interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
                loop {
                    self.tick().await;
                    interval.tick().await;
                }
            }
        })
    }
}
