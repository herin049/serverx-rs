#![feature(proc_macro_quote)]

mod identifier;
mod nbt;
mod packet;
mod proto;

use proc_macro::TokenStream;

use crate::{
    identifier::identifier_macro_impl,
    nbt::nbt_macro_impl,
    packet::packet_macro_impl,
    proto::{proto_decode_macro_impl, proto_encode_macro_impl},
};

#[proc_macro_derive(Packet, attributes(packet))]
pub fn packet_macro(input: TokenStream) -> TokenStream {
    packet_macro_impl(input.into()).into()
}

#[proc_macro_derive(ProtoEncode, attributes(proto))]
pub fn proto_encode_macro(input: TokenStream) -> TokenStream {
    proto_encode_macro_impl(input.into()).into()
}

#[proc_macro_derive(ProtoDecode, attributes(proto))]
pub fn proto_decode_macro(input: TokenStream) -> TokenStream {
    proto_decode_macro_impl(input.into()).into()
}

#[proc_macro]
pub fn nbt(input: TokenStream) -> TokenStream {
    nbt_macro_impl(input.into()).into()
}

#[proc_macro]
pub fn identifier(input: TokenStream) -> TokenStream {
    identifier_macro_impl(input.into()).into()
}
