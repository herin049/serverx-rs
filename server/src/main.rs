use std::path::Path;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::server::Server;

pub mod client;
mod config;
mod network;
mod resources;
mod server;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    if let Some(config) = config::load("run/config.toml") {
        if let Ok(resources) = resources::load(&Path::new("run/resources")) {
            let mut server = Server::new(config, resources);
            server.start();
        } else {
            tracing::error!("unable to load server resources");
        }
    } else {
        tracing::info!("Creating default configuration file");
        config::create_default("run/config.toml");
    }
}
