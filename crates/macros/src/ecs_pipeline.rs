use std::str::FromStr;

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, LitInt};

use crate::ecs_tuple::typle_impl_n;

pub fn pipeline_impl(input: TokenStream) -> TokenStream {
    let input_count: LitInt = parse2(input).expect("unable to parse macro input");
    let count: usize = input_count.base10_parse().unwrap();
    let pipeline_impls = (1..=count).into_iter().map(|i| pipeline_impl_n(i));
    quote! {
        #(#pipeline_impls)*
    }
}

fn pipeline_impl_n(count: usize) -> TokenStream {
    let ty_idents: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(format!("T{}", i).as_str()).unwrap())
        .collect_vec();
    let var0: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(format!("v{}", i).as_str()).unwrap())
        .collect_vec();
    let var1: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(format!("w{}", i).as_str()).unwrap())
        .collect_vec();
    let ty_indexes: Vec<TokenStream> = (0..count)
        .into_iter()
        .map(|i| TokenStream::from_str(i.to_string().as_str()).unwrap())
        .collect_vec();
    quote! {
        impl<'a, 'b, #(#ty_idents: SystemIter<'b>),*> SystemPipeline<'a>
            for Pipeline<'a, (#(&'a mut #ty_idents,)*)> {}

        impl<'a, 'b, 'r, #(#ty_idents: SystemIter<'b>),*> Runnable<'r>
            for SystemPipelineRunnable<'a, Pipeline<'a, (#(&'a mut #ty_idents,)*)>>
        where 'r: 'b
        {
            fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local as BorrowTuple<'b>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());)*
            }

            fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global as BorrowTuple<'b>>::WriteType::type_ids().as_ref());)*
            }

            fn run(&mut self, registry: &'r mut Registry) {
                #(util::assert_no_alias(<#ty_idents::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref(), "aliasing in write components");)*
                let registry_cell = UnsafeRegistryCell(registry);
                for archetype in registry_cell.archetypes() {
                    #(let mut #var0 = if util::subset(<<#ty_idents as SystemIter<'b>>::Local as BorrowTuple<'b>>::ValueType::type_ids().as_ref(), archetype.type_ids()) {
                        unsafe {
                            archetype.table().partitions_mut::<'_, '_, 'b, #ty_idents::Local>(self.pipeline.1)
                        }
                    } else {
                        TablePartitionsMut::empty()
                    };)*

                    loop {
                        let mut some = false;
                        #(if let Some(mut chunk) = #var0.next() {
                            let mut accessor = Accessor::<'_, 'b, #ty_idents::Local, #ty_idents::Global>::new(registry_cell.clone());
                            accessor.iter_pos = (archetype.id(), chunk.start() as ArchetypeIdx);
                            for values in chunk.iter() {
                                self.pipeline.0.#ty_indexes.iter(values, &mut accessor);
                                accessor.iter_pos.1 += 1;
                            }
                            some = true;
                        })*
                        if !some {
                            break;
                        }
                    }
                }
            }
        }

        impl<'a, 'b, #(#ty_idents: SystemParIter<'b>),*> SystemParPipeline<'a> for
            ParPipeline<'a, (#(&'a mut #ty_idents,)*)> {}

        impl<'a, 'b, 'r, #(#ty_idents: SystemParIter<'b>),*> RunnablePar<'r> for
            SystemParPipelineRunnable<'a, ParPipeline<'a, (#(&'a mut #ty_idents,)*)>> where 'r: 'b {
            fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local as BorrowTuple<'b>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());)*
            }

            fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global as BorrowTuple<'b>>::WriteType::type_ids().as_ref());)*
            }

            fn run(&mut self, registry: &'r mut Registry) {
                let self_ref = &*self;
                let registry_cell = UnsafeRegistryCell(registry);
                let mut global_read: BTreeSet<TypeId> = BTreeSet::new();
                let mut local_write: BTreeSet<TypeId> = BTreeSet::new();
                #(util::assert_no_alias(<#ty_idents::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref(), "aliasing in write components");)*
                #(global_read.extend(<#ty_idents::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());)*
                #(local_write.extend(<#ty_idents::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());)*
                if !global_read.is_disjoint(&local_write) {
                    panic!("system global read aliases with local write");
                }
                rayon::scope(|s| {
                    for archetype in registry_cell.archetypes() {
                        #(let mut #var0 = if util::subset(<#ty_idents::Local as BorrowTuple<'b>>::ValueType::type_ids().as_ref(), archetype.type_ids()) {
                            unsafe {
                                archetype.table().partitions_mut::<'_, '_, 'b, #ty_idents::Local>(self.pipeline.1)
                            }
                        } else {
                            TablePartitionsMut::empty()
                        };)*

                        loop {
                            #(let mut #var1 = #var0.next();)*
                            if #(#var1.is_none() &&)* true {
                                break;
                            }
                            let registry_cell_copy = registry_cell.clone();
                            let archetype_id = archetype.id();
                            s.spawn(move |_| {
                                #(if let Some(mut chunk) = #var1 {
                                    let mut accessor = Accessor::<'_, 'b, #ty_idents::Local, #ty_idents::Global>::new(registry_cell_copy.clone());
                                    accessor.iter_pos = (archetype_id, chunk.start() as ArchetypeIdx);
                                    for values in chunk.iter() {
                                       self_ref.pipeline.0.#ty_indexes.iter(values, &mut accessor);
                                    }
                                })*
                            });
                        }
                    }
                });
            }
        }
    }
}
