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
            type OptionType = (#(Option::<#ty_idents>,)*);

            const COMPONENT_COUNT: usize = #count;

            const COMPONENT_SET: ComponentSet = {
                let mut component_set = [0u64; COMPONENT_SET_LEN];
                #(component_set[(#ty_idents::ID as usize) / 64] |= 1 << ((#ty_idents::ID as u64) % 64);)*
                ComponentSet(component_set)
            };

            unsafe fn insert_unchecked(
                self,
                components: &Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>,
                index: Index,
            ) {
                unsafe {
                    #(
                        (&mut *components.get(#ty_idents::ID).unwrap().get())
                            .downcast_mut_unchecked::<ComponentVec<#ty_idents>>()
                            .insert_unchecked(index, self.#ty_indexes);
                    )*
                }
            }

            unsafe fn put_unchecked(
                self,
                components: &Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>,
                component_set: &ComponentSet,
                index: Index,
            ) -> Self::OptionType {
                (#(
                    if component_set.contains(#ty_idents::ID) {
                        let old = (&mut *components.get(#ty_idents::ID).unwrap().get())
                            .downcast_mut_unchecked::<ComponentVec<#ty_idents>>()
                            .remove_unchecked(index);
                        (&mut *components.get(#ty_idents::ID).unwrap().get())
                            .downcast_mut_unchecked::<ComponentVec<#ty_idents>>()
                            .insert_unchecked(index, self.#ty_indexes);
                        Some(old)
                    } else {
                        (&mut *components.get(#ty_idents::ID).unwrap().get())
                            .downcast_mut_unchecked::<ComponentVec<#ty_idents>>()
                            .insert_unchecked(index, self.#ty_indexes);
                        None
                    }
                ,)*)
            }
        }

        impl<#(#ty_idents: ComponentBorrow),*> ComponentBorrowTuple for (#(#ty_idents,)*) {
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

            unsafe fn get_unchecked(storage: &SyncUnsafeCell<Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>>, index: Index) -> Self {
                (#(#ty_idents::get_unchecked((&*storage.get()).get(#ty_idents::ValueType::ID).unwrap(), index),)*)
            }
        }

        impl<#(#ty_idents: ComponentRef),*> ComponentRefTuple for (#(#ty_idents,)*) where (#(#ty_idents,)*): ComponentBorrowTuple {}
    }
}
