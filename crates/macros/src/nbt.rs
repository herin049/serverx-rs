use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token, Expr, ExprArray, LitStr, Token,
};

pub struct NbtCompoundExpr {
    pub name: LitStr,
    pub colon: Token![:],
    pub value: NbtExpr,
}

impl Parse for NbtCompoundExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub enum NbtExpr {
    Value(Expr),
    Compound(Vec<(String, NbtExpr)>),
    List(Vec<NbtExpr>),
}

impl Parse for NbtExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace) {
            let content;
            let _ = braced!(content in input);
            Ok(Self::Compound(
                Punctuated::<NbtCompoundExpr, Token![,]>::parse_terminated(&content)?
                    .into_iter()
                    .map(|s| (s.name.value(), s.value))
                    .collect_vec(),
            ))
        } else if input.peek(token::Bracket) {
            let content;
            let _ = bracketed!(content in input);
            Ok(Self::List(
                Punctuated::<NbtExpr, Token![,]>::parse_terminated(&content)?
                    .into_iter()
                    .collect_vec(),
            ))
        } else {
            Ok(Self::Value(input.parse()?))
        }
    }
}

pub fn nbt_macro_impl(input: TokenStream) -> TokenStream {
    let nbt_expr: NbtExpr = parse2::<NbtExpr>(input).expect("unable to parse NBT expression");
    expand_nbt_expr(nbt_expr)
}

pub fn expand_nbt_expr(nbt_expr: NbtExpr) -> TokenStream {
    match nbt_expr {
        NbtExpr::Value(expr) => quote! {
            serverx_nbt::Tag::from(#expr)
        },
        NbtExpr::Compound(elms) => {
            let names = elms.iter().map(|s| s.0.clone()).collect_vec();
            let elm_expr = elms
                .into_iter()
                .map(|(n, e)| expand_nbt_expr(e))
                .collect_vec();
            quote! {
                serverx_nbt::Tag::Compound(vec![#(serverx_nbt::NamedTag::from((#names, #elm_expr))),*])
            }
        }
        NbtExpr::List(elms) => {
            let elm_expr = elms.into_iter().map(expand_nbt_expr).collect_vec();
            quote! {
                serverx_nbt::Tag::List(vec![#(#elm_expr),*])
            }
        }
    }
}
