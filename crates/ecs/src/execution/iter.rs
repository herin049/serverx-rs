use std::{any::TypeId, cmp, collections::BTreeSet, marker::PhantomData, ops::Range};

use crate::{
    archetype::UnsafeArchetypeCell,
    entity::Entity,
    execution::run::{Runnable, RunnablePar},
    registry::{access::Accessor, Registry, UnsafeRegistryCell},
    tuple::{
        borrow::BorrowTuple,
        component::{ComponentBorrowTuple, ComponentRefTuple},
        value::ValueTuple,
    },
    util,
    util::assert_no_alias,
};

pub trait SystemIter<'a>: Sized + 'a {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentBorrowTuple<'a>;

    fn runnable(&mut self) -> SystemIterRunnable<'_, 'a, Self> {
        SystemIterRunnable {
            phantom: PhantomData,
            iter: self,
        }
    }

    fn iter(
        &mut self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    );
}

pub struct SystemIterRunnable<'a, 'b, T: SystemIter<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<&'b T>,
    iter: &'a mut T,
}

impl<'a, 'b, 'r, T: SystemIter<'b>> Runnable<'r> for SystemIterRunnable<'a, 'b, T>
where
    'r: 'b,
    'a: 'b,
{
    fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
    }

    fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
    }

    fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
    }

    fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
    }

    fn run(&mut self, registry: &'r mut Registry) {
        assert_no_alias(
            <T::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref(),
            "aliasing in write components",
        );
        let registry_cell = UnsafeRegistryCell(registry);
        let mut accessor =
            Accessor::<'_, 'b, T::Local, T::Global>::new(UnsafeRegistryCell(registry));
        for archetype in registry_cell.archetypes() {
            if util::subset(
                <T::Local as BorrowTuple<'b>>::ValueType::type_ids().as_ref(),
                archetype.type_ids().as_ref(),
            ) {
                unsafe {
                    for (entity, values) in UnsafeArchetypeCell(archetype).iter_mut::<T::Local>() {
                        accessor.entity = entity;
                        self.iter.iter(entity, values, &mut accessor);
                    }
                }
            }
        }
    }
}

pub trait SystemParIter<'a>: Sized + Sync + 'a {
    type Local: ComponentBorrowTuple<'a> + Send;
    type Global: ComponentRefTuple<'a> + ComponentBorrowTuple<'a>;

    fn runnable(&mut self) -> SystemParIterRunnable<'_, 'a, Self> {
        SystemParIterRunnable {
            phantom: PhantomData,
            iter: self,
        }
    }
    fn iter(
        &self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    );
}

pub struct SystemParIterRunnable<'a, 'b, T: SystemParIter<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<&'b T>,
    iter: &'a mut T,
}

impl<'a, 'b, 'r, T: SystemParIter<'b>> RunnablePar<'r> for SystemParIterRunnable<'a, 'b, T>
where
    'r: 'b,
    'a: 'b,
{
    fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
    }

    fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
    }

    fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
    }

    fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
    }

    fn run(&mut self, registry: &'r mut Registry) {
        let self_ref = &*self;
        assert_no_alias(
            <T::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref(),
            "aliasing in write components",
        );
        let registry_cell = UnsafeRegistryCell(registry);
        if !util::disjoint(
            <T::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref(),
            <T::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref(),
        ) {
            panic!("system global read aliases with local write");
        }
        rayon::scope(|s| {
            for archetype in registry_cell.archetypes() {
                if util::subset(
                    <T::Local as BorrowTuple<'b>>::ValueType::type_ids().as_ref(),
                    archetype.type_ids(),
                ) {
                    unsafe {
                        for mut chunk in
                            UnsafeArchetypeCell(archetype).partitions_mut::<'_, 'b, T::Local>(4096)
                        {
                            let registry_cell_copy = registry_cell.clone();
                            s.spawn(move |_| {
                                let mut accessor = Accessor::<'_, 'b, T::Local, T::Global>::new(
                                    registry_cell_copy,
                                );
                                for (entity, values) in chunk.iter() {
                                    accessor.entity = entity;
                                    self_ref.iter.iter(entity, values, &mut accessor);
                                }
                            });
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        component::Component,
        entity::Entity,
        execution::{
            iter::{SystemIter, SystemParIter},
            run::{Runnable, RunnablePar},
        },
        registry::{access::Accessor, Registry},
    };

    #[derive(Debug)]
    pub struct Position(i64, i64, i64);
    #[derive(Debug)]
    pub struct Velocity(i64, i64, i64);

    impl Component for Position {}
    impl Component for Velocity {}

    pub struct SimpleIter<'a>(&'a i32);

    impl<'a, 'b> SystemIter<'a> for SimpleIter<'b>
    where
        'b: 'a,
    {
        type Global = ();
        type Local = (&'a mut Position, &'a Velocity);

        fn iter(
            &mut self,
            entity: Entity,
            (p, v): Self::Local,
            accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
        ) {
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    pub struct SimpleParIter<'a>(&'a i32);

    impl<'a, 'b> SystemParIter<'a> for SimpleParIter<'b>
    where
        'b: 'a,
    {
        type Global = ();
        type Local = (&'a mut Position, &'a Velocity);

        fn iter(
            &self,
            entity: Entity,
            (p, v): Self::Local,
            accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
        ) {
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    fn run_dyn<'a, 'b>(sys: &'a mut dyn Runnable<'b>, registry: &'b mut Registry) {
        sys.run(registry);
    }

    fn run_dyn_par<'a, 'b>(sys: &'a mut dyn RunnablePar<'b>, registry: &'b mut Registry) {
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
