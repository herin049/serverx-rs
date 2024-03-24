use tracing::instrument;

use crate::{client::Client, server::Server};

#[instrument(skip_all)]
pub fn sync_client(client: &mut Client, server: &Server) {}
