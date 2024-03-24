use std::{
    fs::File,
    io::{Read, Write},
    iter::Iterator,
    option::Option,
    path::Path,
};

use serde_derive::{Deserialize, Serialize};
use toml::de::Error;
use tracing::instrument;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 25565,
        }
    }
}

#[instrument]
pub fn load(file_name: &str) -> Option<ServerConfig> {
    if Path::new(file_name).exists() {
        tracing::debug!("reading configuration file");
        let mut file = File::open(file_name).expect("unable to read from configuration file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        match toml::from_str::<ServerConfig>(contents.as_str()) {
            Ok(config) => Some(config),
            Err(err) => {
                tracing::error!("unable to parse configuration file");
                panic!("malformed configuration file");
            }
        }
    } else {
        tracing::debug!("configuration file not found");
        None
    }
}

#[instrument]
pub fn create_default(file_name: &str) {
    tracing::debug!("creating default configuration file");
    let mut file = File::create(file_name).expect("unable to create configuration file");
    let toml_str = toml::to_string(&ServerConfig::default()).unwrap();
    file.write_all(toml_str.as_bytes()).unwrap();
}
