use std::str::FromStr;

use darling::{
    ast,
    export::syn::{
        parse, parse2, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields,
        Ident, LitStr, Type, Variant,
    },
    util, FromDeriveInput, FromField, FromVariant,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

#[derive(FromField, Default)]
#[darling(default, attributes(proto))]
struct FieldOpts {
    repr: Option<LitStr>,
    min_len: Option<usize>,
    max_len: Option<usize>,
    exact_len: Option<usize>,
}

impl FieldOpts {
    pub fn is_seq(&self) -> bool {
        self.min_len.is_some() || self.max_len.is_some() || self.exact_len.is_some()
    }
}

#[derive(FromDeriveInput, Default)]
struct EnumOpts {
    tag_repr: Option<LitStr>,
}

#[derive(FromVariant, Default)]
#[darling(default, attributes(proto))]
struct VariantOpts {
    tag: Option<usize>,
}

pub fn proto_encode_macro_impl(input: TokenStream) -> TokenStream {
    let input = parse2::<DeriveInput>(input).expect("error parsing derive input");
    let data = input.data.clone();
    match data {
        Data::Struct(s) => proto_encode_macro_struct(input, s),
        Data::Enum(e) => proto_encode_macro_enum(input, e),
        Data::Union(_) => panic!("union types are not supported"),
    }
}

pub fn proto_encode_macro_struct(input: DeriveInput, data_struct: DataStruct) -> TokenStream {
    let name = &input.ident;

    let encode_stmts = data_struct.fields.iter().enumerate().map(|(i, e)| {
        let name: Expr = if let Some(ident) = &e.ident {
            parse_quote! { &data.#ident }
        } else {
            let str = format!("&data.{}", i);
            parse2(TokenStream::from_str(str.as_str()).unwrap()).unwrap()
        };
        let writer: Expr = parse_quote! { writer };
        encode_field_impl(&name, &writer, e)
    });

    quote! {
        impl protocol::encode::ProtoEncode for #name {
            type Repr = Self;

            fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), protocol::encode::ProtoEncodeErr> {
                #(#encode_stmts)*
                Ok(())
            }
        }
    }
}

pub fn proto_encode_macro_enum(input: DeriveInput, data_enum: DataEnum) -> TokenStream {
    let name = &input.ident;
    let enum_opts = EnumOpts::from_derive_input(&input).expect("invalid enum attributes");
    let enum_repr = enum_opts
        .tag_repr
        .map(|s| TokenStream::from_str(s.value().as_str()).unwrap())
        .unwrap_or(quote! { VarInt });
    let encode_arms = data_enum.variants.iter().enumerate().map(|(i, e)| {
        let variant_opts = VariantOpts::from_variant(e).expect("invalid variant attributes");
        let tag = TokenStream::from_str(&format!("&{}", variant_opts.tag.unwrap_or(i))).unwrap();
        let tag_expr = parse_quote! { #tag };
        let writer: Expr = parse_quote! { writer };
        encode_variant_impl(name, e, &enum_repr, &tag_expr, &writer)
    });
    quote! {
        impl protocol::encode::ProtoEncode for #name {
            type Repr = Self;

            fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), protocol::encode::ProtoEncodeErr> {
                match data {
                    #(#encode_arms)*
                }
            }
        }
    }
}

pub fn proto_decode_macro_impl(input: TokenStream) -> TokenStream {
    let input = parse2::<DeriveInput>(input).expect("error parsing derive input");
    let data = input.data.clone();
    match data {
        Data::Struct(s) => proto_decode_macro_struct(input, s),
        Data::Enum(e) => proto_decode_macro_enum(input, e),
        Data::Union(_) => panic!("union types are not supported"),
    }
}

pub fn proto_decode_macro_struct(input: DeriveInput, data_struct: DataStruct) -> TokenStream {
    let name = &input.ident;

    let decode_stmts = data_struct.fields.iter().enumerate().map(|(i, e)| {
        let name: Expr = if let Some(ident) = &e.ident {
            parse_quote! { #ident }
        } else {
            let var_name = format_ident!("var{}", i);
            parse_quote! { #var_name }
        };
        let reader: Expr = parse_quote! { reader };
        let alloc_tracker: Expr = parse_quote! { alloc_tracker };
        decode_field_impl(&name, &reader, &alloc_tracker, e)
    });

    let decode_vars = data_struct.fields.iter().enumerate().map(|(i, e)| {
        let name: Expr = if let Some(ident) = &e.ident {
            parse_quote! { #ident }
        } else {
            let var_name = format_ident!("var{}", i);
            parse_quote! { #var_name }
        };
        name
    });

    let decode_result: TokenStream = if data_struct
        .fields
        .iter()
        .find(|f| f.ident.is_some())
        .is_some()
    {
        quote! {
            Ok(#name {
                #(#decode_vars),*
            })
        }
    } else if data_struct.fields.len() > 0 {
        quote! {
            Ok(#name (
                #(#decode_vars),*
            ))
        }
    } else {
        quote! {
            Ok(#name)
        }
    };

    quote! {
        impl protocol::decode::ProtoDecode for #name {
            type Repr = Self;

            fn decode<R: Read + Seek, A: protocol::decode::AllocTracker>(reader: &mut R, alloc_tracker: &mut A) -> Result<Self::Repr, protocol::decode::ProtoDecodeErr> {
                #(#decode_stmts)*
                #decode_result
            }
        }
    }
}

pub fn proto_decode_macro_enum(input: DeriveInput, data_enum: DataEnum) -> TokenStream {
    let name = &input.ident;
    let enum_opts = EnumOpts::from_derive_input(&input).expect("invalid enum attributes");
    let enum_repr = enum_opts
        .tag_repr
        .map(|s| TokenStream::from_str(s.value().as_str()).unwrap())
        .unwrap_or(quote! { VarInt });

    let decode_arms = data_enum.variants.iter().enumerate().map(|(i, e)| {
        let variant_opts = VariantOpts::from_variant(e).expect("invalid variant attributes");
        let tag = TokenStream::from_str(&format!("{}", variant_opts.tag.unwrap_or(i))).unwrap();
        let tag_expr: Expr = parse_quote! { #tag };
        let reader: Expr = parse_quote! { reader };
        let alloc_tracker: Expr = parse_quote! { alloc_tracker };
        decode_variant_impl(name, e, &enum_repr, &tag_expr, &reader, &alloc_tracker)
    });

    quote! {
        impl protocol::decode::ProtoDecode for #name {
            type Repr = Self;

            fn decode<R: Read + Seek, A: protocol::decode::AllocTracker>(
                reader: &mut R,
                alloc_tracker: &mut A,
            ) -> Result<Self::Repr, protocol::decode::ProtoDecodeErr> {
                match <#enum_repr as protocol::decode::ProtoDecode>::decode(reader, alloc_tracker)? {
                    #(#decode_arms)*
                    _ => Err(protocol::decode::ProtoDecodeErr::InvalidEnumTag)
                }
            }
        }
    }
}

pub fn proto_macro_struct(input: DeriveInput, data_struct: DataStruct) -> TokenStream {
    let name = &input.ident;

    let encode_stmts = data_struct.fields.iter().enumerate().map(|(i, e)| {
        let name: Expr = if let Some(ident) = &e.ident {
            parse_quote! { &data.#ident }
        } else {
            let str = format!("&data.{}", i);
            parse2(TokenStream::from_str(str.as_str()).unwrap()).unwrap()
        };
        let writer: Expr = parse_quote! { writer };
        encode_field_impl(&name, &writer, e)
    });

    let decode_stmts = data_struct.fields.iter().enumerate().map(|(i, e)| {
        let name: Expr = if let Some(ident) = &e.ident {
            parse_quote! { #ident }
        } else {
            let var_name = format_ident!("var{}", i);
            parse_quote! { #var_name }
        };
        let reader: Expr = parse_quote! { reader };
        let alloc_tracker: Expr = parse_quote! { alloc_tracker };
        decode_field_impl(&name, &reader, &alloc_tracker, e)
    });

    let decode_vars = data_struct.fields.iter().enumerate().map(|(i, e)| {
        let name: Expr = if let Some(ident) = &e.ident {
            parse_quote! { #ident }
        } else {
            let var_name = format_ident!("var{}", i);
            parse_quote! { #var_name }
        };
        name
    });

    let decode_result: TokenStream = if data_struct
        .fields
        .iter()
        .find(|f| f.ident.is_some())
        .is_some()
    {
        quote! {
            Ok(#name {
                #(#decode_vars),*
            })
        }
    } else if data_struct.fields.len() > 0 {
        quote! {
            Ok(#name (
                #(#decode_vars),*
            ))
        }
    } else {
        quote! {
            Ok(#name)
        }
    };

    let result = quote! {
        impl protocol::encode::ProtoEncode for #name {
            type Repr = Self;

            fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), protocol::encode::ProtoEncodeErr> {
                #(#encode_stmts)*
                Ok(())
            }
        }

        impl protocol::decode::ProtoDecode for #name {
            type Repr = Self;

            fn decode<R: Read + Seek, A: protocol::decode::AllocTracker>(reader: &mut R, alloc_tracker: &mut A) -> Result<Self::Repr, protocol::decode::ProtoDecodeErr> {
                #(#decode_stmts)*
                #decode_result
            }
        }

    };
    result
}

pub fn proto_macro_enum(input: DeriveInput, data_enum: DataEnum) -> TokenStream {
    let name = &input.ident;
    let enum_opts = EnumOpts::from_derive_input(&input).expect("invalid enum attributes");
    let enum_repr = enum_opts
        .tag_repr
        .map(|s| TokenStream::from_str(s.value().as_str()).unwrap())
        .unwrap_or(quote! { VarInt });
    let encode_arms = data_enum.variants.iter().enumerate().map(|(i, e)| {
        let variant_opts = VariantOpts::from_variant(e).expect("invalid variant attributes");
        let tag = TokenStream::from_str(&format!("&{}", variant_opts.tag.unwrap_or(i))).unwrap();
        let tag_expr = parse_quote! { #tag };
        let writer: Expr = parse_quote! { writer };
        encode_variant_impl(name, e, &enum_repr, &tag_expr, &writer)
    });

    let decode_arms = data_enum.variants.iter().enumerate().map(|(i, e)| {
        let variant_opts = VariantOpts::from_variant(e).expect("invalid variant attributes");
        let tag = TokenStream::from_str(&format!("{}", variant_opts.tag.unwrap_or(i))).unwrap();
        let tag_expr: Expr = parse_quote! { #tag };
        let reader: Expr = parse_quote! { reader };
        let alloc_tracker: Expr = parse_quote! { alloc_tracker };
        decode_variant_impl(name, e, &enum_repr, &tag_expr, &reader, &alloc_tracker)
    });

    let result = quote! {
        impl protocol::encode::ProtoEncode for #name {
            type Repr = Self;

            fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), protocol::encode::ProtoEncodeErr> {
                match data {
                    #(#encode_arms)*
                }
            }
        }

        impl protocol::decode::ProtoDecode for #name {
            type Repr = Self;

            fn decode<R: Read + Seek, A: protocol::decode::AllocTracker>(
                reader: &mut R,
                alloc_tracker: &mut A,
            ) -> Result<Self::Repr, protocol::decode::ProtoDecodeErr> {
                match <#enum_repr as protocol::decode::ProtoDecode>::decode(reader, alloc_tracker)? {
                    #(#decode_arms)*
                    _ => Err(protocol::decode::ProtoDecodeErr::InvalidEnumTag)
                }
            }
        }
    };
    result
}

pub fn get_field_names(fields: &Fields) -> Vec<Ident> {
    fields
        .iter()
        .enumerate()
        .map(|(i, e)| {
            if let Some(ident) = &e.ident {
                ident.clone()
            } else {
                format_ident!("var{}", i)
            }
        })
        .collect()
}

pub fn encode_variant_impl(
    enum_name: &Ident,
    variant: &Variant,
    enum_repr: &TokenStream,
    tag: &Expr,
    writer: &Expr,
) -> TokenStream {
    let variant_name = &variant.ident;
    let field_names = get_field_names(&variant.fields);
    let encode_stmts = variant.fields.iter().zip(&field_names).map(|(f, n)| {
        let name_expr: Expr = parse_quote! { #n };
        encode_field_impl(&name_expr, writer, f)
    });
    let result = if variant.fields.is_empty() {
        quote! {
            #enum_name::#variant_name => {
                <#enum_repr as protocol::encode::ProtoEncode>::encode(#tag, #writer)?;
                Ok(())
            }
        }
    } else if variant.fields.iter().find(|f| f.ident.is_some()).is_some() {
        quote! {
            #enum_name::#variant_name { #(#field_names),* } => {
                <#enum_repr as protocol::encode::ProtoEncode>::encode(#tag, #writer)?;
                #(#encode_stmts)*
                Ok(())
            }
        }
    } else {
        quote! {
            #enum_name::#variant_name(#(#field_names),* ) => {
                <#enum_repr as protocol::encode::ProtoEncode>::encode(#tag, #writer)?;
                #(#encode_stmts)*
                Ok(())
            }
        }
    };
    result
}

pub fn decode_variant_impl(
    enum_name: &Ident,
    variant: &Variant,
    enum_repr: &TokenStream,
    tag: &Expr,
    reader: &Expr,
    alloc_tracker: &Expr,
) -> TokenStream {
    let variant_name = &variant.ident;
    let field_names = get_field_names(&variant.fields);
    let decode_stmts = variant.fields.iter().zip(&field_names).map(|(f, n)| {
        let name_expr: Expr = parse_quote! { #n };
        decode_field_impl(&name_expr, reader, alloc_tracker, f)
    });
    let result = if variant.fields.is_empty() {
        quote! {
            #tag => {
                Ok(#enum_name::#variant_name)
            }
        }
    } else if variant.fields.iter().find(|f| f.ident.is_some()).is_some() {
        quote! {
            #tag => {
                #(#decode_stmts)*
                Ok(#enum_name::#variant_name {
                    #(#field_names),*
                })
            }
        }
    } else {
        quote! {
            #tag => {
                #(#decode_stmts)*
                Ok(#enum_name::#variant_name(
                    #(#field_names),*
                ))
            }
        }
    };
    result
}

pub fn encode_field_impl(name: &Expr, writer: &Expr, field: &Field) -> TokenStream {
    let field_opts = FieldOpts::from_field(field).expect("invalid field attributes");
    let field_ty = &field.ty;
    let field_repr = field_opts
        .repr
        .clone()
        .map(|r| TokenStream::from_str(r.value().as_str()).expect("invalid field repr"))
        .unwrap_or(quote! { #field_ty });

    let mut result = TokenStream::new();
    if field_opts.is_seq() {
        result.extend(quote! {
            let __len: usize = <#field_repr as protocol::encode::ProtoEncodeSeq>::encode_len(#name, #writer)?;
        });

        if let Some(min_len) = field_opts.min_len {
            result.extend(quote! {
                if __len < #min_len {
                    return Err(protocol::encode::ProtoEncodeErr::SeqTooShort(__len, #min_len));
                }
            });
        }

        if let Some(max_len) = field_opts.max_len {
            result.extend(quote! {
                if __len > #max_len {
                    return Err(protocol::encode::ProtoEncodeErr::SeqTooLong(__len, #max_len));
                }
            })
        }

        if let Some(exact_len) = field_opts.exact_len {
            result.extend(quote! {
                if __len != #exact_len {
                    return Err(protocol::encode::ProtoEncodeErr::SeqLenMismatch(__len, #exact_len));
                }
            })
        }

        result.extend(quote! {
            <#field_repr as protocol::encode::ProtoEncodeSeq>::encode_seq(#name, #writer, __len)?;
        });
    } else {
        result.extend(quote! {
            <#field_repr as protocol::encode::ProtoEncode>::encode(#name, #writer)?;
        });
    }
    result
}

pub fn decode_field_impl(
    name: &Expr,
    reader: &Expr,
    alloc_tracker: &Expr,
    field: &Field,
) -> TokenStream {
    let field_opts = FieldOpts::from_field(field).expect("invalid field attributes");
    let field_ty = &field.ty;
    let field_repr = field_opts
        .repr
        .clone()
        .map(|r| TokenStream::from_str(r.value().as_str()).expect("invalid field repr"))
        .unwrap_or(quote! { #field_ty });
    let mut result = TokenStream::new();
    if field_opts.is_seq() {
        result.extend(quote! {
            let __len: usize = <#field_repr as protocol::decode::ProtoDecodeSeq>::decode_len(#reader, #alloc_tracker)?;
        });

        if let Some(min_len) = field_opts.min_len {
            result.extend(quote! {
                if __len < #min_len {
                    return Err(protocol::decode::ProtoDecodeErr::SeqTooShort(__len, #min_len));
                }
            });
        }

        if let Some(max_len) = field_opts.max_len {
            result.extend(quote! {
                if __len > #max_len {
                    return Err(protocol::decode::ProtoDecodeErr::SeqTooLong(__len, #max_len));
                }
            })
        }

        if let Some(exact_len) = field_opts.exact_len {
            result.extend(quote! {
                if __len != #exact_len {
                    return Err(protocol::decode::ProtoDecodeErr::SeqLenMismatch(__len, #exact_len));
                }
            })
        }

        result.extend(quote! {
            let #name = <#field_repr as protocol::decode::ProtoDecodeSeq>::decode_seq(#reader, #alloc_tracker, __len)?;
        });
    } else {
        result.extend(quote! {
            let #name = <#field_repr as protocol::decode::ProtoDecode>::decode(#reader, #alloc_tracker)?;
        });
    }
    result
}
