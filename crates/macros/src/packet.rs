use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, LitInt, Token,
};

struct PacketOpts {
    id: LitInt,
    direction: Ident,
    state: Ident,
}

impl Parse for PacketOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id: LitInt = input.parse()?;
        let sep0: Token![,] = input.parse()?;
        let direction: Ident = input.parse()?;
        let sep1: Token![,] = input.parse()?;
        let state: Ident = input.parse()?;
        Ok(Self {
            id,
            direction,
            state,
        })
    }
}

pub fn packet_macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse2::<DeriveInput>(input).expect("error parsing derive input");
    let name = &input.ident;
    let packet_attrs = input
        .attrs
        .iter()
        .find(|s| s.path.is_ident("packet"))
        .expect("missing \"packet\" attribute");
    let packet_opts: PacketOpts = packet_attrs
        .parse_args()
        .expect("invalid \"packet\" attribute");
    let id: LitInt = packet_opts.id;
    let direction: Ident = packet_opts.direction;
    let state: Ident = packet_opts.state;
    let result = quote! {
        impl #name {
            pub const ID: i32 = #id;
            pub const DIRECTION: protocol::packet::PacketDirection = protocol::packet::PacketDirection::#direction;
            pub const STATE: protocol::packet::ConnectionState = protocol::packet::ConnectionState::#state;
        }

        impl protocol::packet::Packet for #name {
            fn id(&self) -> i32 {
                Self::ID
            }

            fn direction(&self) -> protocol::packet::PacketDirection {
                Self::DIRECTION
            }

            fn state(&self) -> protocol::packet::ConnectionState {
                Self::STATE
            }

            fn as_any(&self) -> &(dyn Any + Send + Sync) {
                self
            }

            fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
                self
            }
        }
    };
    result
}
