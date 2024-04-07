use std::{any::TypeId, cmp, collections::BTreeSet, marker::PhantomData, ops::Range};

use crate::{
    archetype::{ArchetypeId, ArchetypeIdx, UnsafeArchetypeCell},
    entity::Entity,
    execution::run::{Runnable, RunnablePar},
    registry::{
        access::{Accessor, IterAccessor},
        Registry, UnsafeRegistryCell,
    },
    tuple::{
        borrow::BorrowTuple,
        component::{ComponentBorrowTuple, ComponentRefTuple},
        message::SenderTuple,
        value::ValueTuple,
    },
    util,
    util::assert_no_alias,
};

pub trait RegistryIter: Sized {
    type Local<'l>: ComponentBorrowTuple<'l>;
    type Global<'g>: ComponentBorrowTuple<'g>;
    type Send<'s>: SenderTuple<'s>;

    fn runnable(&mut self) -> RegistryIterRunnable<'_, Self> {
        RegistryIterRunnable { iter: self }
    }

    fn iter(
        &mut self,
        components: Self::Local<'_>,
        accessor: &mut impl Accessor,
        send: &mut Self::Send<'_>,
    );
}

pub struct RegistryIterRunnable<'a, T: RegistryIter> {
    iter: &'a mut T,
}

impl<'a, T: RegistryIter> Runnable for RegistryIterRunnable<'a, T> {
    fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());
    }

    fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());
    }

    fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());
    }

    fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());
    }

    fn extend_message_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Send<'_> as SenderTuple<'_>>::MessageType::type_ids().as_ref());
    }

    fn prepare(&self, registry: &mut Registry) {
        T::Send::register(&mut registry.messages_mut());
    }

    fn run(&mut self, registry: &mut Registry) {
        assert_no_alias(
            <T::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref(),
            "aliasing in write components",
        );
        self.prepare(registry);
        let registry_cell = UnsafeRegistryCell(registry);
        let mut senders = unsafe { T::Send::sender(&registry_cell.messages().unsafe_cell()) };
        let mut accessor = IterAccessor::<T::Local<'static>, T::Global<'static>>::new(
            UnsafeRegistryCell(registry),
        );
        for archetype in registry_cell.archetypes() {
            if util::subset(
                <T::Local<'_> as BorrowTuple<'_>>::ValueType::type_ids().as_ref(),
                archetype.type_ids().as_ref(),
            ) {
                unsafe {
                    accessor.iter_pos = (archetype.id(), 0);
                    for values in archetype.table().iter_mut::<T::Local<'_>>() {
                        accessor.iter_pos.1 += 1;
                        self.iter.iter(values, &mut accessor, &mut senders);
                    }
                }
            }
        }
    }
}

pub trait RegistryParIter: Sized + Sync {
    type Local<'l>: ComponentBorrowTuple<'l> + Send;
    type Global<'g>: ComponentRefTuple<'g> + ComponentBorrowTuple<'g>;
    type Send<'s>: SenderTuple<'s>;

    fn runnable(&mut self) -> RegistryParIterRunnable<'_, Self> {
        RegistryParIterRunnable { iter: self }
    }
    fn iter(
        &self,
        components: Self::Local<'_>,
        accessor: &mut impl Accessor,
        send: &mut Self::Send<'_>,
    );
}

pub struct RegistryParIterRunnable<'a, T: RegistryParIter> {
    iter: &'a mut T,
}

impl<'a, T: RegistryParIter> RunnablePar for RegistryParIterRunnable<'a, T> {
    fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());
    }

    fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());
    }

    fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());
    }

    fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());
    }

    fn extend_message_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Send<'_> as SenderTuple<'_>>::MessageType::type_ids().as_ref());
    }

    fn finalize(&self, registry: &mut Registry) {
        unsafe { T::Send::sync(registry.messages_mut()) }
    }

    fn prepare(&self, registry: &mut Registry) {
        T::Send::register(registry.messages_mut());
    }

    fn run(&mut self, registry: &mut Registry) {
        self.prepare(registry);
        let self_ref = &*self;
        assert_no_alias(
            <T::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref(),
            "aliasing in write components",
        );
        let registry_cell = UnsafeRegistryCell(registry);
        if !util::disjoint(
            <T::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref(),
            <T::Local<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref(),
        ) {
            panic!("system global read aliases with local write");
        }
        rayon::scope(|s| {
            for archetype in registry_cell.archetypes() {
                if util::subset(
                    <T::Local<'_> as BorrowTuple<'_>>::ValueType::type_ids().as_ref(),
                    archetype.type_ids(),
                ) {
                    unsafe {
                        for mut chunk in archetype
                            .table()
                            .partitions_mut::<'_, '_, '_, T::Local<'_>>(4096)
                        {
                            let registry_cell_copy = registry_cell.clone();
                            let archetype_id = archetype.id();
                            s.spawn(move |_| {
                                let mut send = unsafe {
                                    T::Send::sender_tl(&registry_cell_copy.messages().unsafe_cell())
                                };
                                let mut accessor =
                                    IterAccessor::<T::Local<'static>, T::Global<'static>>::new(
                                        registry_cell_copy,
                                    );
                                accessor.iter_pos = (archetype_id, chunk.start() as ArchetypeIdx);
                                for values in chunk.iter() {
                                    self_ref.iter.iter(values, &mut accessor, &mut send);
                                    accessor.iter_pos.1 += 1;
                                }
                            });
                        }
                    }
                }
            }
        });
        self.finalize(registry);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        component::Component,
        entity::Entity,
        execution::{
            iter::{RegistryIter, RegistryParIter},
            run::{Runnable, RunnablePar},
        },
        message::Message,
        registry::{
            access::{Accessor, IterAccessor},
            Registry,
        },
        storage::channel::Sender,
    };

    #[derive(Debug)]
    pub struct Position(i64, i64, i64);
    #[derive(Debug)]
    pub struct Velocity(i64, i64, i64);

    impl Component for Position {}
    impl Component for Velocity {}

    pub struct SimpleIter<'a>(&'a i32);

    impl<'b> RegistryIter for SimpleIter<'b> {
        type Global<'g> = ();
        type Local<'l> = (&'l mut Position, &'l Velocity, &'l Entity);
        type Send<'s> = (Sender<'s, String>,);

        fn iter(
            &mut self,
            (p, v, e): Self::Local<'_>,
            accessor: &mut impl Accessor,
            send: &mut Self::Send<'_>,
        ) {
            send.0.send(format!("{:?}", e));
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    pub struct SimpleParIter<'a>(&'a i32);

    impl<'a> RegistryParIter for SimpleParIter<'a> {
        type Global<'g> = ();
        type Local<'l> = (&'l mut Position, &'l Velocity, &'l Entity);
        type Send<'s> = ();

        fn iter(
            &self,
            (p, v, e): Self::Local<'_>,
            accessor: &mut impl Accessor,
            send: &mut Self::Send<'_>,
        ) {
            println!("{:?}", e);
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    impl Message for String {}

    fn run_dyn(sys: &mut dyn Runnable, registry: &mut Registry) {
        sys.run(registry);
    }

    fn run_dyn_par(sys: &mut dyn RunnablePar, registry: &mut Registry) {
        sys.run(registry);
    }

    #[test]
    fn test() {
        let mut reg = Registry::new();
        for _ in 0..20 {
            reg.push((Position(0, 0, 0), Velocity(1, -1, 0)));
        }
        println!("{:#?}", reg);
        let i = 123;
        let mut s = SimpleIter(&i);
        run_dyn(&mut s.runnable(), &mut reg);
        println!("{:?}", reg.messages().messages::<String>());
        // s.runnable().run(&mut reg);
        let mut sp = SimpleParIter(&i);
        run_dyn_par(&mut sp.runnable(), &mut reg);
        // sp.runnable().run(&mut reg);
        println!("{:#?}", reg);
    }

    #[test]
    fn test2() {
        let a = {
            #[derive(Debug)]
            struct Foo {
                bar: i32,
            }
            Foo { bar: 1 }
        };
        let b = {
            #[derive(Debug)]
            struct Foo {
                baz: i32,
            }
            Foo { baz: 2 }
        };

        println!("{:?}, {:?}", a, b);
    }

    #[test]
    fn test3() {
        #[derive(Debug)]
        struct Foo<'a> {
            bar: &'a i32,
        };

        #[derive(Debug)]
        struct Baz<'a> {
            qux: &'a i32,
        };

        let x = 1i32;
        let y = 2i32;

        let mut a = Foo { bar: &x };
        let mut b = Baz { qux: &y };

        let something = {
            #[derive(Debug)]
            struct Something<'a, T, U> {
                a: &'a mut T,
                b: &'a mut U,
            }

            Something {
                a: &mut a,
                b: &mut b,
            }
        };

        println!("{:?}", something);
    }
}
