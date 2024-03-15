use serverx_core::{
    protocol::v765::clientbound::{
        FeatureFlags, RegistryData, ServerFinishConfiguration, UpdateTags,
    },
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

impl ClientState {
    #[instrument(skip_all)]
    pub fn update_client_mut(self, server: &mut Server, client_handle: ClientHandle) -> Self {
        self
    }

    #[instrument(skip_all)]
    pub fn update_client(self, server: &Server, client_handle: ClientHandle) -> Self {
        match self {
            Self::Init => {
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
                    let update_tags = UpdateTags { tags: vec![] };
                    tracing::trace!(?update_tags, "sending update tags packet");
                    let _ = client.outgoing.send(Box::new(update_tags));
                    let finish_config = ServerFinishConfiguration;
                    tracing::trace!("sending finish configuration packet");
                    let _ = client.outgoing.send(Box::new(finish_config));
                    Self::Connecting
                } else {
                    Self::Disconnecting
                }
            }
            Self::Connecting => Self::Connected,
            Self::Connected => self,
            Self::Disconnecting => Self::Disconnected,
            Self::Disconnected => self,
        }
    }
}