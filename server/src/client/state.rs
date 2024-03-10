use serverx_core::{
    protocol::v765::clientbound::{FeatureFlags, RegistryData},
    utils,
};
use serverx_macros::identifier;
use tracing::instrument;

use crate::{client::ClientHandle, network::brand::make_server_brand_message, server::Server};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ClientState {
    Init,
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
}

#[instrument(skip_all)]
pub fn update_init(
    server: &mut Server,
    client_handle: ClientHandle,
    state: ClientState,
) -> Option<ClientState> {
    if let Some(client) = server.clients.get(client_handle) {
        let server_brand = make_server_brand_message(&identifier!("vanilla"));
        tracing::trace!(?server_brand, "sending server brand message");
        let feature_flags = FeatureFlags {
            flags: vec![identifier!("vanilla")],
        };
        tracing::trace!(?feature_flags, "sending feature flags packet");
        let _ = client.outgoing.send(Box::new(feature_flags));
        tracing::trace!("sending registry data packet");
        let _ = client.outgoing.send(Box::new(RegistryData {
            registries: server.resources.registry_data.clone(),
        }));
        Some(ClientState::Connecting)
    } else {
        Some(ClientState::Disconnecting)
    }
}

pub fn update_client(
    server: &mut Server,
    client_handle: ClientHandle,
    state: ClientState,
) -> Option<ClientState> {
    match state {
        ClientState::Init => update_init(server, client_handle, state),
        ClientState::Connecting => Some(ClientState::Connected),
        ClientState::Connected => None,
        ClientState::Disconnecting => Some(ClientState::Disconnected),
        ClientState::Disconnected => None,
    }
}
