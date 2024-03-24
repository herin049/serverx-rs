use std::{
    io,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use flume::Sender;
use tokio::net::TcpListener;

use crate::network::{accept::accept_client, event::NetworkEvent};

pub async fn listen(addr: SocketAddr, events: Sender<NetworkEvent>) {
    if let Ok(listener) = TcpListener::bind(addr).await {
        loop {
            if let Ok((mut socket, addr)) = listener.accept().await {
                let events_clone = events.clone();
                tokio::spawn(async move {
                    accept_client(socket, addr, events_clone).await;
                });
            }
        }
    } else {
        tracing::error!(?addr, "unable to bind to address");
    }
}
