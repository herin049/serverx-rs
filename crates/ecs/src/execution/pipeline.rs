use std::{any::TypeId, collections::BTreeSet, iter, marker::PhantomData};

use serverx_macros::ecs_pipeline_impl;

use crate::{
    archetype::{ArchetypeIdx, UnsafeArchetypeCell},
    execution::{
        iter::{RegistryIter, RegistryParIter},
        run::{Runnable, RunnablePar},
    },
    registry::{access::IterAccessor, Registry, UnsafeRegistryCell},
    storage::table::TablePartitionsMut,
    tuple::{borrow::BorrowTuple, message::SenderTuple, value::ValueTuple},
    util,
};

pub trait RegistryPipeline: Sized {
    fn runnable<'a>(self) -> RegistryPipelineRunnable<'a, Self>
    where
        Self: 'a,
    {
        RegistryPipelineRunnable {
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

pub struct RegistryPipelineRunnable<'a, T: RegistryPipeline> {
    phantom: PhantomData<&'a T>,
    pipeline: T,
}

pub trait RegistryParPipeline: Sized {
    fn runnable<'a>(self) -> RegistryParPipelineRunnable<'a, Self>
    where
        Self: 'a,
    {
        RegistryParPipelineRunnable {
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

pub struct RegistryParPipelineRunnable<'a, T: RegistryParPipeline> {
    phantom: PhantomData<&'a T>,
    pipeline: T,
}

ecs_pipeline_impl!(10);

#[cfg(test)]
mod tests {
    use crate::{
        component::Component,
        entity::Entity,
        execution::{
            iter::{RegistryIter, RegistryParIter},
            pipeline::{ParPipeline, Pipeline, RegistryParPipeline, RegistryPipeline},
            run::{Runnable, RunnablePar},
        },
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
    pub struct SimpleParIter<'a>(&'a i32);

    impl<'b> RegistryIter for SimpleIter<'b> {
        type Global<'g> = ();
        type Local<'l> = (&'l mut Position, &'l Entity, &'l Velocity);
        type Send<'s> = ();

        fn iter(
            &mut self,
            (p, e, v): Self::Local<'_>,
            accessor: &mut impl Accessor,
            send: &mut Self::Send<'_>,
        ) {
            println!("{:?}", e);
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    impl<'a> RegistryParIter for SimpleParIter<'a> {
        type Global<'g> = ();
        type Local<'l> = (&'l Entity, &'l mut Position, &'l Velocity);
        type Send<'s> = (Sender<'s, String>,);

        fn iter(
            &self,
            (e, p, v): Self::Local<'_>,
            accessor: &mut impl Accessor,
            send: &mut Self::Send<'_>,
        ) {
            send.0.send(format!("system {}: {:?}", self.0, e));
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
        let i = 1;
        let j = 2;
        let mut s1 = SimpleParIter(&i);
        let mut s2 = SimpleParIter(&j);
        let p = ParPipeline::new((&mut s1, &mut s2), 5);
        p.runnable().run(&mut reg);
        // println!("{:?}", reg);
        println!("{:#?}", reg.messages().messages::<String>());
    }
}
