use std::str::FromStr;

use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, parse_quote, LitInt};

pub fn component_tuple_macro_impl(input: TokenStream) -> TokenStream {
    let input_count: LitInt = parse2(input).expect("unable to parse macro input");
    let component_value_tuple_impls = (1..=input_count.base10_parse().unwrap())
        .into_iter()
        .map(|i| component_ntuple_impl(i));
    quote! {
        #(#component_value_tuple_impls)*
    }
}

pub fn component_ntuple_impl(count: usize) -> TokenStream {
    let ty_idents: Vec<TokenStream> = (b'A'..=b'Z')
        .into_iter()
        .take(count)
        .map(|c| {
            let ident = format_ident!("{}", c as char);
            quote! {
                #ident
            }
        })
        .collect_vec();
    let ty_indexes: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(i.to_string().as_str()).unwrap())
        .collect_vec();
    quote! {
        impl<#(#ty_idents: Component),*> ComponentTuple for (#(#ty_idents,)*) {
            const COMPONENT_SET: ComponentSet = {
                let mut component_set = [0u64; COMPONENT_SET_LEN];
                #(component_set[(#ty_idents::ID as usize) / 64] |= 1 << ((#ty_idents::ID as u64) % 64);)*
                ComponentSet(component_set)
            };

            const COMPONENT_COUNT: usize = #count;

            unsafe fn init_storage_unchecked(storage: &mut ArchetypeStorage) {
                #(
                    storage
                        .components
                        .get_unchecked_mut(#ty_idents::ID as usize)
                        .write(Box::new(ComponentVec::<#ty_idents>::new()));
                )*
            }

            unsafe fn insert_unchecked(self, storage: &mut ArchetypeStorage, index: Index) {
                unsafe {
                    #(
                        storage
                            .components
                            .get_unchecked_mut(#ty_idents::ID as usize)
                            .assume_init_mut()
                            .downcast_mut_unchecked::<ComponentVec<#ty_idents>>()
                            .insert_unchecked(index, self.#ty_indexes);
                    )*
                }
            }
        }

        impl<'a, #(#ty_idents: ComponentBorrow<'a>),*> ComponentBorrowTuple<'a> for (#(#ty_idents,)*) {
            const READ_COMPONENT_SET: ComponentSet = {
                let mut component_set = [0u64; COMPONENT_SET_LEN];
                #(if #ty_idents::REF {
                    component_set[(#ty_idents::ValueType::ID as usize) / 64] |= 1 << ((#ty_idents::ValueType::ID as u64) % 64);
                })*
                ComponentSet(component_set)
            };
            const WRITE_COMPONENT_SET: ComponentSet = {
                let mut component_set = [0u64; COMPONENT_SET_LEN];
                #(if #ty_idents::MUT {
                    component_set[(#ty_idents::ValueType::ID as usize) / 64] |= 1 << ((#ty_idents::ValueType::ID as u64) % 64);
                })*
                ComponentSet(component_set)
            };

            type ValueType = (#(#ty_idents::ValueType,)*);

            unsafe fn get_unchecked(storage: &'a ArchetypeStorage, index: Index) -> Self {
                (#(#ty_idents::get_unchecked(storage, index),)*)
            }
        }

        impl<'a, #(#ty_idents: ComponentRef<'a>),*> ComponentRefTuple<'a> for (#(#ty_idents,)*) where (#(#ty_idents,)*): ComponentBorrowTuple<'a> {

        }
    }
}
