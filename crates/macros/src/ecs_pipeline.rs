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
        impl<'a, #(#ty_idents: RegistryIter),*> RegistryPipeline
            for Pipeline<'a, (#(&'a mut #ty_idents,)*)> {}

        impl<'a, #(#ty_idents: RegistryIter),*> Runnable
            for RegistryPipelineRunnable<'a, Pipeline<'a, (#(&'a mut #ty_idents,)*)>>
        {
            fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());)*
            }

            fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());)*
            }

            fn extend_message_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Send::<'_> as SenderTuple<'_>>::MessageType::type_ids().as_ref());)*
            }

            fn extend_message_read(&self, type_ids: &mut BTreeSet<TypeId>) {
            }

            fn prepare(&self, registry: &mut Registry) {
                #(#ty_idents::Send::<'_>::register(&mut registry.messages_mut());)*
            }

            fn run(&mut self, registry: &mut Registry) {
                #(util::assert_no_alias(<#ty_idents::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref(), "aliasing in write components");)*
                self.prepare(registry);
                let registry_cell = UnsafeRegistryCell(registry);
                for archetype in registry_cell.archetypes() {
                    #(let mut #var0 = if util::subset(<<#ty_idents as RegistryIter>::Local<'_> as BorrowTuple<'_>>::ValueType::type_ids().as_ref(), archetype.type_ids()) {
                        unsafe {
                            archetype.table().partitions_mut::<'_, '_, '_, #ty_idents::Local<'_>>(self.pipeline.1)
                        }
                    } else {
                        TablePartitionsMut::empty()
                    };)*
                    #(let mut #var1 = unsafe { #ty_idents::Send::<'_>::sender(&registry_cell.messages().unsafe_cell()) };)*
                    loop {
                        let mut some = false;
                        #(if let Some(mut chunk) = #var0.next() {
                            let mut accessor = IterAccessor::<#ty_idents::Local<'static>, #ty_idents::Global<'static>>::new(registry_cell.clone());
                            accessor.iter_pos = (archetype.id(), chunk.start() as ArchetypeIdx);
                            for values in chunk.iter() {
                                self.pipeline.0.#ty_indexes.iter(values, &mut accessor, &mut #var1);
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

        impl<'a, #(#ty_idents: RegistryParIter),*> RegistryParPipeline for
            ParPipeline<'a, (#(&'a mut #ty_idents,)*)> {}

        impl<'a, #(#ty_idents: RegistryParIter),*> RunnablePar for
            RegistryParPipelineRunnable<'a, ParPipeline<'a, (#(&'a mut #ty_idents,)*)>> {
            fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());)*
            }

            fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());)*
            }

            fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Global<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());)*
            }

            fn extend_message_write(&self, type_ids: &mut BTreeSet<TypeId>) {
                #(type_ids.extend(<#ty_idents::Send::<'_> as SenderTuple<'_>>::MessageType::type_ids().as_ref());)*
            }

            fn extend_message_read(&self, type_ids: &mut BTreeSet<TypeId>) {
            }

            fn finalize(&self, registry: &mut Registry) {
                unsafe {
                    #(#ty_idents::Send::sync(registry.messages_mut());)*
                }
            }

            fn prepare(&self, registry: &mut Registry) {
                #(#ty_idents::Send::<'_>::register(&mut registry.messages_mut());)*
            }

            fn run(&mut self, registry: &mut Registry) {
                self.prepare(registry);
                let self_ref = &*self;
                let registry_cell = UnsafeRegistryCell(registry);
                let mut global_read: BTreeSet<TypeId> = BTreeSet::new();
                let mut local_write: BTreeSet<TypeId> = BTreeSet::new();
                #(util::assert_no_alias(<#ty_idents::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref(), "aliasing in write components");)*
                #(global_read.extend(<#ty_idents::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());)*
                #(local_write.extend(<#ty_idents::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());)*
                if !global_read.is_disjoint(&local_write) {
                    panic!("system global read aliases with local write");
                }
                rayon::scope(|s| {
                    for archetype in registry_cell.archetypes() {
                        #(let mut #var0 = if util::subset(<#ty_idents::Local<'_> as BorrowTuple<'_>>::ValueType::type_ids().as_ref(), archetype.type_ids()) {
                            unsafe {
                                archetype.table().partitions_mut::<'_, '_, '_, #ty_idents::Local<'_>>(self.pipeline.1)
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
                                    let mut accessor = IterAccessor::<#ty_idents::Local<'static>, #ty_idents::Global<'static>>::new(registry_cell_copy.clone());
                                    let mut send = unsafe { #ty_idents::Send::sender_tl(&registry_cell_copy.messages().unsafe_cell()) };
                                    accessor.iter_pos = (archetype_id, chunk.start() as ArchetypeIdx);
                                    for values in chunk.iter() {
                                       self_ref.pipeline.0.#ty_indexes.iter(values, &mut accessor, &mut send);
                                    }
                                })*
                            });
                        }
                    }
                });
                self.finalize(registry);
            }
        }
    }
}
