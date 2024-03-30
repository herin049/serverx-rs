use std::str::FromStr;

use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, parse_quote, LitInt};

pub fn tuple_macro_impl(input: TokenStream) -> TokenStream {
    let input_count: LitInt = parse2(input).expect("unable to parse macro input");
    let count: usize = input_count.base10_parse().unwrap();
    let component_value_tuple_impls = (1..=count).into_iter().map(|i| typle_impl_n(i, count));
    quote! {
        #(#component_value_tuple_impls)*
    }
}

pub fn typle_impl_n(count: usize, max_count: usize) -> TokenStream {
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
    let count_ts = TokenStream::from_str(count.to_string().as_ref()).unwrap();
    let ty_indexes: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(i.to_string().as_str()).unwrap())
        .collect_vec();
    let mut results: Vec<TokenStream> = Vec::new();
    results.push(
    quote! {
        impl<#(#ty_idents: Component),*> ComponentTuple for (#(#ty_idents,)*) {
            type PtrType = (#(*mut #ty_idents,)*);
            type BlobArray = [BlobStorage; #count];

            #[inline(always)]
            unsafe fn write(self, ptr: Self::PtrType) {
                #(ptr.#ty_indexes.write(self.#ty_indexes);) *
            }

            #[inline(always)]
            fn blobs() -> Self::BlobArray {
                [#(BlobStorage::new::<#ty_idents>()),*]
            }
        }

        impl<#(#ty_idents: Component),*> TypeTuple for (#(#ty_idents,)*) {
            type TypeIdArray = [TypeId; #count];
            #[inline(always)]
            fn type_ids() -> Self::TypeIdArray {
                [#(TypeId::of::<#ty_idents>()),*]
            }
        }

        impl<#(#ty_idents: Component),*> PtrTuple for (#(*mut #ty_idents,)*) {
            type PtrArray = [*mut u8; #count];

            #[inline(always)]
            fn null_ptr_slice() -> Self::PtrArray {
                [std::ptr::null_mut(); #count]
            }

            #[inline(always)]
            fn from_ptr_slice(ptrs: &[*mut u8]) -> Self {
                (#(ptrs[#ty_indexes] as *mut #ty_idents,)*)
            }

            #[inline(always)]
            unsafe fn offset(self, count: isize) -> Self {
                (#(self.#ty_indexes.offset(count),)*)
            }
        }

        impl<'a, #(#ty_idents: ComponentRefType<'a>),*> ComponentRefTuple<'a> for (#(#ty_idents,)*) {
            type ValueType = (#(#ty_idents::ValueType,)*);

            #[inline(always)]
            unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self {
                (#(#ty_idents::deref(ptr.#ty_indexes),)*)
            }
        }

        impl<#(#ty_idents: Component,)* T: Component> ComponentTupleAdd<T> for (#(#ty_idents,)*)
            where (#(#ty_idents,)* T) : ComponentTuple {
            type ValueType = (#(#ty_idents,)* T);
        }

        impl<'a, #(#ty_idents: Component,)* T: Component> ComponentTupleAddRef<&'a T> for (#(#ty_idents,)*)
            where (#(#ty_idents,)* T) : ComponentTuple {
            type ValueType = (#(#ty_idents,)* T);
        }

        impl<'a, #(#ty_idents: Component,)* T: Component> ComponentTupleAddRef<&'a mut T> for (#(#ty_idents,)*)
            where (#(#ty_idents,)*) : ComponentTuple {
            type ValueType = (#(#ty_idents,)*);
        }

        impl<'a, #(#ty_idents: Component,)* T: Component> ComponentTupleAddMut<&'a T> for (#(#ty_idents,)*)
            where (#(#ty_idents,)*) : ComponentTuple {
            type ValueType = (#(#ty_idents,)*);
        }

        impl<'a, #(#ty_idents: Component,)* T: Component> ComponentTupleAddMut<&'a mut T> for (#(#ty_idents,)*)
            where (#(#ty_idents,)* T) : ComponentTuple {
            type ValueType = (#(#ty_idents,)* T);
        }
    });
    if count < max_count {
        results.push(quote! {
            impl<'a, #(#ty_idents: ComponentBorrowType<'a>),*, T: ComponentBorrowType<'a>> ComponentBorrowTuple<'a> for (#(#ty_idents),*, T) where
                (#(#ty_idents,)*) : ComponentBorrowTuple<'a>, <(#(#ty_idents,)*) as ComponentBorrowTuple<'a>>::ReadType: ComponentTupleAddRef<T>,
                <(#(#ty_idents,)*) as ComponentBorrowTuple<'a>>::WriteType: ComponentTupleAddMut<T>
            {
                type ValueType = (#(#ty_idents::ValueType,)* T::ValueType);
                type ReadType = <<(#(#ty_idents,)*) as ComponentBorrowTuple<'a>>::ReadType as ComponentTupleAddRef<T>>::ValueType;
                type WriteType = <<(#(#ty_idents,)*) as ComponentBorrowTuple<'a>>::WriteType as ComponentTupleAddMut<T>>::ValueType;

                #[inline(always)]
                unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self {
                    (#(#ty_idents::deref(ptr.#ty_indexes),)* T::deref(ptr.#count_ts))
                }
            }
        });
    }
    quote! {
        #(#results)*
    }
}
