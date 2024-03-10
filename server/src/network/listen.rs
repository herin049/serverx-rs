use std::net::SocketAddr;

use flume::Sender;
use tokio::net::TcpListener;

use crate::network::{accept::accept_client, event::NetEvent};

pub fn listen(addr: SocketAddr, event_queue: Sender<NetEvent>) {
    tokio::spawn(async move {
        if let Ok(listener) = TcpListener::bind(addr).await {
            loop {
                if let Ok((mut socket, addr)) = listener.accept().await {
                    let queue_clone = event_queue.clone();
                    tokio::spawn(async move {
                        accept_client(socket, addr, queue_clone);
                    });
                }
            }
        } else {
            tracing::error!(?addr, "unable to bind to address");
        }
    });
}
