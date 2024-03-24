use serverx_macros::identifier;
use tracing::instrument;
use serverx_protocol::v765::clientbound::{FeatureFlags, RegistryData, ServerFinishConfiguration, UpdateTags};
use serverx_common::identifier;

use crate::{
    client::{status::ClientStatus, Client},
    network::{brand::make_server_brand_message},
    server::Server,
};

#[instrument(skip_all)]
pub fn update_client(client: &mut Client, server: &mut Server) {
    match client.status {
        ClientStatus::Init => {
            let server_brand = make_server_brand_message(&identifier!("vanilla"));
            tracing::trace!(?server_brand, "sending server brand message");
            let _ = client.outgoing.send(Box::new(server_brand));
            let feature_flags = FeatureFlags {
                flags: vec![identifier!("vanilla")],
            };
            tracing::trace!(?feature_flags, "sending feature flags packet");
            let _ = client.outgoing.send(Box::new(feature_flags));
            let registry_data = RegistryData {
                registries: server.resources.registry_data.clone(),
            };
            tracing::trace!("sending registry data packet");
            let _ = client.outgoing.send(Box::new(registry_data));
            let update_tags = UpdateTags { tags: vec![] };
            tracing::trace!(?update_tags, "sending update tags packet");
            let _ = client.outgoing.send(Box::new(update_tags));
            let finish_config = ServerFinishConfiguration;
            tracing::trace!("sending finish configuration packet");
            let _ = client.outgoing.send(Box::new(finish_config));
            client.status = ClientStatus::Connecting;
        }
        ClientStatus::Connecting => {
            client.status = ClientStatus::Connected;
        }
        ClientStatus::Connected => {
            if client.incoming.is_disconnected() || client.outgoing.is_disconnected() {
                client.status = ClientStatus::Disconnecting;
            }
        }
        ClientStatus::Disconnecting => {
            tracing::debug!(profile = ?client.profile, "client disconnected");
            client.status = ClientStatus::Disconnected;
        }
        ClientStatus::Disconnected => {}
    }
}
