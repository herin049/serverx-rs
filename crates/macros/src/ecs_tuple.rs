use std::str::FromStr;

use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, parse_quote, LitInt};

pub fn tuple_macro_impl(input: TokenStream) -> TokenStream {
    let input_count: LitInt = parse2(input).expect("unable to parse macro input");
    let count: usize = input_count.base10_parse().unwrap();
    let component_value_tuple_impls = (1..=count).into_iter().map(|i| typle_impl_n(i));
    quote! {
        #(#component_value_tuple_impls)*
    }
}

pub fn typle_impl_n(count: usize) -> TokenStream {
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
    let ty_idents_head = ty_idents.first().unwrap();
    let ty_idents_tail = &ty_idents[1..];
    let count_ts = TokenStream::from_str(count.to_string().as_ref()).unwrap();
    let ty_indexes: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(i.to_string().as_str()).unwrap())
        .collect_vec();
    let mut results: Vec<TokenStream> = Vec::new();
    results.push(quote! {
        impl<#(#ty_idents),*> PtrTuple for (#(*mut #ty_idents,)*) {
            type PtrArray = [*mut u8; #count];

            #[inline(always)]
            fn null_ptr_slice() -> Self::PtrArray {
                [std::ptr::null_mut(); #count]
            }

            #[inline(always)]
            fn null_ptr() -> Self {
                (#(std::ptr::null_mut::<#ty_idents>(),)*)
            }

            #[inline(always)]
            unsafe fn from_ptr_slice(ptrs: &[*mut u8]) -> Self {
                (#(*ptrs.get_unchecked(#ty_indexes) as *mut #ty_idents,)*)
            }

            #[inline(always)]
            unsafe fn offset(self, count: isize) -> Self {
                (#(self.#ty_indexes.offset(count),)*)
            }

            #[inline(always)]
            unsafe fn add(self, count: usize) -> Self {
                (#(self.#ty_indexes.add(count),)*)
            }
        }

        impl<#(#ty_idents: 'static),*> ValueTuple for (#(#ty_idents,)*) {
            type PtrType = (#(*mut #ty_idents,)*);
            type TypeIdArray = [TypeId; #count];

            #[inline(always)]
            fn type_ids() -> Self::TypeIdArray {
                [#(TypeId::of::<#ty_idents>()),*]
            }

            #[inline(always)]
            unsafe fn write(self, ptr: Self::PtrType) {
                #(std::ptr::write(ptr.#ty_indexes, self.#ty_indexes));*
            }

            #[inline(always)]
            unsafe fn read(ptr: Self::PtrType) -> Self {
                (#(std::ptr::read::<#ty_idents>(ptr.#ty_indexes as *const #ty_idents),)*)
            }
        }

        impl<#(#ty_idents: 'static + Sized + Debug),*> TableLayout for (#(#ty_idents,)*) {
            type ColumnArray = [Column; #count];

            #[inline(always)]
            fn columns() -> Self::ColumnArray {
                [#(Column::new::<#ty_idents>()),*]
            }
        }

        impl<#(#ty_idents),*> TupleAdd<#ty_idents_head> for (#(#ty_idents_tail,)*) {
            type Result = (#(#ty_idents,)*);
        }

        impl<'a#(,#ty_idents)*> TupleAddRef<&'a #ty_idents_head> for (#(#ty_idents_tail,)*) {
            type Result = (#(#ty_idents,)*);
        }

        impl<'a#(,#ty_idents)*> TupleAddRef<&'a mut #ty_idents_head> for (#(#ty_idents_tail,)*) {
            type Result = (#(#ty_idents_tail,)*);
        }

        impl<'a#(,#ty_idents)*> TupleAddMut<&'a mut #ty_idents_head> for (#(#ty_idents_tail,)*) {
            type Result = (#(#ty_idents,)*);
        }

        impl<'a#(,#ty_idents)*> TupleAddMut<&'a #ty_idents_head> for (#(#ty_idents_tail,)*) {
            type Result = (#(#ty_idents_tail,)*);
        }

        impl<'a#(,#ty_idents: RefType<'a>)*> RefTuple<'a> for (#(#ty_idents,)*) {
            type ValueType = (#(#ty_idents::ValueType,)*);

            unsafe fn deref(ptr: <Self::ValueType as ValueTuple>::PtrType) -> Self {
               (#(#ty_idents::deref(ptr.#ty_indexes),)*)
            }
        }

        impl<'a#(,#ty_idents: BorrowType<'a>)*> BorrowTuple<'a> for (#(#ty_idents,)*) where
            (#(#ty_idents_tail,)*): BorrowTuple<'a>,
            <(#(#ty_idents_tail,)*) as BorrowTuple<'a>>::ReadType: TupleAddRef<#ty_idents_head>,
            <(#(#ty_idents_tail,)*) as BorrowTuple<'a>>::WriteType: TupleAddMut<#ty_idents_head>,
            <<(#(#ty_idents_tail,)*) as BorrowTuple<'a>>::ReadType as TupleAddRef<#ty_idents_head>>::Result: ValueTuple,
            <<(#(#ty_idents_tail,)*) as BorrowTuple<'a>>::WriteType as TupleAddMut<#ty_idents_head>>::Result: ValueTuple,
        {
            type ValueType = (#(#ty_idents::ValueType,)*);
            type ReadType = <<(#(#ty_idents_tail,)*) as BorrowTuple<'a>>::ReadType as TupleAddRef<#ty_idents_head>>::Result;
            type WriteType = <<(#(#ty_idents_tail,)*) as BorrowTuple<'a>>::WriteType as TupleAddMut<#ty_idents_head>>::Result;

            #[inline(always)]
            unsafe fn deref(ptr: <Self::ValueType as ValueTuple>::PtrType) -> Self {
               (#(#ty_idents::deref(ptr.#ty_indexes),)*)
            }
        }

        impl<#(#ty_idents: Component),*> ComponentTuple for (#(#ty_idents,)*) {
        }
    });
    quote! {
        #(#results)*
    }
}
