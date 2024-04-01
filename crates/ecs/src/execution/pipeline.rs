use std::{any::TypeId, collections::BTreeSet, iter, marker::PhantomData};

use serverx_macros::ecs_pipeline_impl;

use crate::{
    archetype::{ArchetypePartitionsMut, UnsafeArchetypeCell},
    execution::{
        iter::{SystemIter, SystemParIter},
        run::{Runnable, RunnablePar},
    },
    registry::{access::Accessor, Registry, UnsafeRegistryCell},
    tuple::{borrow::BorrowTuple, value::ValueTuple},
    util,
};

pub trait SystemPipeline<'a>: Sized + 'a {
    fn runnable(self) -> SystemPipelineRunnable<'a, Self> {
        SystemPipelineRunnable {
            phantom: PhantomData,
            pipeline: self,
        }
    }
}

pub struct Pipeline<'a, T: 'a>(T, usize, PhantomData<&'a T>);

impl<'a, T: 'a> Pipeline<'a, T> {
    pub fn new(iters: T, batch_size: usize) -> Self {
        Self(iters, batch_size, PhantomData)
    }
}

pub struct SystemPipelineRunnable<'a, T: SystemPipeline<'a>> {
    phantom: PhantomData<&'a T>,
    pipeline: T,
}

pub trait SystemParPipeline<'a>: Sized + 'a {
    fn runnable(self) -> SystemParPipelineRunnable<'a, Self> {
        SystemParPipelineRunnable {
            phantom: PhantomData,
            pipeline: self,
        }
    }
}

pub struct ParPipeline<'a, T: 'a>(T, usize, PhantomData<&'a T>);

impl<'a, T: 'a> ParPipeline<'a, T> {
    pub fn new(iters: T, batch_size: usize) -> Self {
        Self(iters, batch_size, PhantomData)
    }
}

pub struct SystemParPipelineRunnable<'a, T: SystemParPipeline<'a>> {
    phantom: PhantomData<&'a T>,
    pipeline: T,
}

// impl<'a, 'b, T0: SystemParIter<'b>, T1: SystemParIter<'b>> SystemParPipeline<'a> for
// ParPipeline<'a, (&'a T0, &'a T1)> {}
//
// impl<'a, 'b, 'r, T0: SystemParIter<'b>, T1: SystemParIter<'b>> RunnablePar<'r> for
// SystemParPipelineRunnable<'a, ParPipeline<'a, (&'a T0, &'a T1)>> where 'r: 'b {
//     fn extend_local_read(&self, type_ids: &mut BTreeSet<TypeId>) {
//         type_ids.extend(<T0::Local as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
//         type_ids.extend(<T1::Local as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
//     }
//     fn extend_local_write(&self, type_ids: &mut BTreeSet<TypeId>) {
//         type_ids.extend(<T0::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
//         type_ids.extend(<T1::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
//
//     }
//     fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
//         type_ids.extend(<T0::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
//         type_ids.extend(<T1::Global as BorrowTuple<'b>>::ReadType::type_ids().as_ref());
//     }
//
//     fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
//         type_ids.extend(<T0::Global as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
//         type_ids.extend(<T1::Global as BorrowTuple<'b>>::WriteType::type_ids().as_ref());
//     }
//
//     fn run(&self, registry: &'r mut Registry) {
//         let registry_cell = UnsafeRegistryCell(registry);
//         util::assert_no_alias(<T0::Local as BorrowTuple<'b>>::WriteType::type_ids().as_ref(),
// "aliasing in write components");         util::assert_no_alias(<T1::Local as
// BorrowTuple<'b>>::WriteType::type_ids().as_ref(), "aliasing in write components");         let
// mut global_read: BTreeSet<TypeId> = BTreeSet::new();         let mut local_write:
// BTreeSet<TypeId> = BTreeSet::new();         global_read.extend(<T0::Global as
// BorrowTuple<'b>>::ReadType::type_ids().as_ref());         global_read.extend(<T1::Global as
// BorrowTuple<'b>>::ReadType::type_ids().as_ref());         local_write.extend(<T0::Local as
// BorrowTuple<'b>>::WriteType::type_ids().as_ref());         local_write.extend(<T1::Local as
// BorrowTuple<'b>>::WriteType::type_ids().as_ref());         if
// !global_read.is_disjoint(&local_write) {             panic!("system global read aliases with
// local write");         }
//         rayon::scope(|s| {
//             for archetype in registry_cell.archetypes() {
//                 let mut t0 = if util::subset(<T0::Local as
// BorrowTuple<'b>>::ValueType::type_ids().as_ref(), archetype.type_ids()) {
// unsafe {                         UnsafeArchetypeCell(archetype).partitions_mut::<'_, 'b,
// T0::Local>(self.pipeline.1)                     }
//                 } else {
//                     ArchetypePartitionsMut::empty()
//                 };
//
//                 let mut t1 = if util::subset(<T1::Local as
// BorrowTuple<'b>>::ValueType::type_ids().as_ref(), archetype.type_ids()) {
// unsafe {                         UnsafeArchetypeCell(archetype).partitions_mut::<'_, 'b,
// T1::Local>(self.pipeline.1)                     }
//                 } else {
//                     ArchetypePartitionsMut::empty()
//                 };
//
//                 loop {
//                     let mut chunk0 = t0.next();
//                     let mut chunk1 = t1.next();
//                     if chunk0.is_none() && chunk1.is_none() {
//                         break;
//                     }
//
//                     let registry_cell_copy = registry_cell.clone();
//                     s.spawn(move |_| {
//                         if let Some(mut chunk) = chunk0 {
//                             let mut accessor = Accessor::<'_, 'b, T0::Local,
// T0::Global>::new(registry_cell_copy.clone());                             for (entity, values) in
// chunk.iter() {                                 accessor.entity = entity;
//                                 self.pipeline.0.0.iter(entity, values, &mut accessor);
//                             }
//                         }
//
//                         if let Some(mut chunk) = chunk1 {
//                             let mut accessor = Accessor::<'_, 'b, T1::Local,
// T1::Global>::new(registry_cell_copy.clone());                             for (entity, values) in
// chunk.iter() {                                 accessor.entity = entity;
//                                 self.pipeline.0.1.iter(entity, values, &mut accessor);
//                             }
//                         }
//                     });
//                 }
//             }
//         });
//         // let mut global_read: Vec<TypeId> =
//     }
// }

ecs_pipeline_impl!(10);

#[cfg(test)]
mod tests {
    use crate::{
        component::Component,
        entity::Entity,
        execution::{
            iter::{SystemIter, SystemParIter},
            pipeline::{ParPipeline, Pipeline, SystemParPipeline, SystemPipeline},
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
    pub struct SimpleParIter<'a>(&'a i32);

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

    #[test]
    fn test() {
        let mut reg = Registry::new();
        for _ in 0..20 {
            reg.push((Position(0, 0, 0), Velocity(1, -1, 0)));
        }
        println!("{:#?}", reg);
        let i = 123;
        let mut s1 = SimpleParIter(&i);
        let mut s2 = SimpleParIter(&i);
        let p = ParPipeline::new((&mut s1, &mut s2), 5);
        p.runnable().run(&mut reg);
        println!("{:?}", reg);
    }
}
