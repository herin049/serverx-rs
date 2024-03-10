use std::io::Cursor;

use serverx_core::{
    protocol::{encode::ProtoEncode, v765::clientbound::ConfigClientBoundPluginMessage},
    utils,
    utils::identifier::Identifier,
};
use serverx_macros::identifier;

pub fn make_server_brand_message(brand: &Identifier) -> ConfigClientBoundPluginMessage {
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let data = if <Identifier as ProtoEncode>::encode(&brand, &mut cursor).is_err() {
        tracing::error!("unable to encode server brand message payload");
        Vec::new()
    } else {
        cursor.into_inner()
    };
    ConfigClientBoundPluginMessage {
        channel: identifier!("brand"),
        data,
    }
}
