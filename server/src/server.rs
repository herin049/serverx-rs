use std::{
    cell::RefCell,
    net::{IpAddr, SocketAddr},
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    ThreadPool, ThreadPoolBuilder,
};
use serverx_core::protocol::packet::{ConnectionState, Packet};
use tokio::{
    runtime::Runtime,
    time,
    time::{Instant, MissedTickBehavior},
};
use tracing::instrument;

use crate::{
    client::{state::ClientState, ClientHandle, Clients},
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
    pub thread_pool: ThreadPool,
    pub resources: Resources,
    pub net_send: Sender<NetEvent>,
    pub net_recv: Receiver<NetEvent>,
}

type ServerHandle = Arc<RwLock<Server>>;

impl Server {
    pub fn new(config: ServerConfig, resources: Resources) -> Server {
        let (net_send, net_recv) = flume::unbounded::<NetEvent>();
        Self {
            clients: Clients::new(),
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .expect("unable to create rayon thread pool"),
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
                    tracing::trace!("starting client read loop");
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

    #[instrument(skip_all)]
    pub async fn update_clients(server_handle: &ServerHandle) {
        let mut client_info: Vec<(ClientHandle, ClientState)> = {
            let server = &*server_handle.read();
            let mut result = Vec::with_capacity(server.clients.clients.len());
            for (_, c) in server.clients.clients.iter() {
                result.push((c.handle, c.state));
            }
            result
        };
        let (send, recv) = tokio::sync::oneshot::channel();
        {
            let mut handle_copy = Arc::clone(server_handle);
            let server = &*server_handle.read();
            server.thread_pool.spawn(move || {
                let _ = send.send(
                    client_info
                        .par_iter()
                        .with_min_len(64)
                        .map(|(c, s)| {
                            let server = &*handle_copy.read();
                            (*c, (*s).update_client(server, *c))
                        })
                        .collect::<Vec<(ClientHandle, ClientState)>>(),
                );
            });
        }
        let mut updated_states = recv.await.expect("panic in rayon thread pool");
        {
            let mut server = &mut *server_handle.write();
            updated_states.iter_mut().for_each(|e| {
                e.1 = e.1.update_client_mut(&mut server, e.0);
            });
            for (h, s) in updated_states {
                let state = if let Some(client) = server.clients.get(h) {
                    client.state
                } else {
                    continue;
                };
                if s != state {
                    if s == ClientState::Disconnected {
                        if let Some(client) = server.clients.remove(h) {
                            tracing::trace!(username = client.username, "removed client");
                        }
                    } else if let Some(client) = server.clients.get_mut(h) {
                        client.state = s;
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn tick(server_handle: &ServerHandle) {
        let start = Instant::now();
        {
            let mut server = &mut *server_handle.write();
            server.process_net_events();
        }
        Self::update_clients(server_handle).await;
        let elapsed = start.elapsed();
        tracing::trace!(?elapsed, "finished tick");
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
        let server_handle = Arc::new(RwLock::new(self));
        rt.block_on(async move {
            loop {
                let mut interval = time::interval(Duration::from_millis(50));
                interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
                loop {
                    Self::tick(&server_handle).await;
                    interval.tick().await;
                }
            }
        })
    }
}
