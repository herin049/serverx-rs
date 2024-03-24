use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use flume::{Receiver, Sender};
use rayon::{ThreadPool, ThreadPoolBuilder};
use smallvec::SmallVec;
use tokio::{
    runtime::Runtime,
    time,
    time::{Instant, MissedTickBehavior},
};
use tracing::instrument;

use crate::{
    client,
    client::{status::ClientStatus, ClientHandle, Clients},
    config::ServerConfig,
    network,
    network::event::NetworkEvent,
    resources::Resources,
};

pub struct Server {
    pub config: ServerConfig,
    pub thread_pool: ThreadPool,
    pub resources: Resources,
    pub net_send: Sender<NetworkEvent>,
    pub net_recv: Receiver<NetworkEvent>,
}

impl Server {
    pub fn new(config: ServerConfig, resources: Resources) -> Self {
        let (net_send, net_recv) = flume::unbounded::<NetworkEvent>();
        Self {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .expect("unable to create rayon threadpool"),
            config,
            resources,
            net_send,
            net_recv,
        }
    }

    #[instrument(skip_all)]
    pub fn process_events(&mut self, clients: &mut Clients) {
        while let Ok(event) = self.net_recv.try_recv() {
            match event {
                NetworkEvent::Connected {
                    addr,
                    outgoing,
                    incoming,
                    profile,
                } => {
                    tracing::debug!(?profile, "player has joined the game");
                    clients.add(addr, outgoing, incoming, profile);
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub fn update_clients(&mut self, clients: &mut Clients) {
        let mut disconnected: SmallVec<[ClientHandle; 4]> = SmallVec::new();
        for (_, client) in clients.clients.iter_mut() {
            client::update::update_client(client, self);
            if client.status == ClientStatus::Disconnected {
                disconnected.push(client.handle);
            }
        }

        for handle in disconnected {
            let _ = clients.remove(handle);
        }
    }

    #[instrument(skip_all)]
    pub fn sync_clients(&self, clients: &mut Clients) {
        for (_, client) in clients.clients.iter_mut() {
            client::sync::sync_client(client, self);
        }
    }

    #[instrument(skip_all)]
    pub async fn tick(&mut self, clients: &mut Clients) {
        let start = Instant::now();
        self.process_events(clients);
        self.update_clients(clients);
        self.sync_clients(clients);
        let elapsed = start.elapsed();
        tracing::info!(?elapsed, "finished tick");
    }

    #[instrument(skip_all)]
    pub fn start(mut self) {
        let rt = Runtime::new().unwrap();
        let _enter_guard = rt.enter();
        let listener_addr = SocketAddr::new(
            IpAddr::from_str(self.config.ip.as_str()).expect("invalid ip address"),
            self.config.port,
        );
        let net_send_clone = self.net_send.clone();
        tokio::spawn(async move {
            network::listen::listen(listener_addr, net_send_clone).await;
        });
        let mut clients = Clients::new();
        rt.block_on(async move {
            loop {
                let mut interval = time::interval(Duration::from_millis(50));
                interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
                loop {
                    self.tick(&mut clients).await;
                    interval.tick().await;
                }
            }
        })
    }
}
