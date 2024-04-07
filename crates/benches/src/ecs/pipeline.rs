use std::cmp;

use criterion::{black_box, Bencher, Criterion};
use serverx_ecs::{
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
};

use crate::ecs::common::*;

struct SimpleIterA;

impl RegistryIter for SimpleIterA {
    type Global<'g> = ();
    type Local<'l> = (&'l mut ComponentA, &'l ComponentB, &'l ComponentC);
    type Send<'s> = ();

    fn iter(
        &mut self,
        (a, b, c): Self::Local<'_>,
        _accessor: &mut impl Accessor,
        _send: &mut Self::Send<'_>,
    ) {
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    }
}

struct SimpleParIterA;

impl RegistryParIter for SimpleParIterA {
    type Global<'g> = ();
    type Local<'l> = (&'l mut ComponentA, &'l ComponentB, &'l ComponentC);
    type Send<'s> = ();

    fn iter(
        &self,
        (a, b, c): Self::Local<'_>,
        _accessor: &mut impl Accessor,
        _send: &mut Self::Send<'_>,
    ) {
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    }
}

struct SimpleIterB;

impl RegistryIter for SimpleIterB {
    type Global<'g> = ();
    type Local<'l> = (&'l ComponentA, &'l mut ComponentB, &'l ComponentC);
    type Send<'s> = ();

    fn iter(
        &mut self,
        (a, b, c): Self::Local<'_>,
        _accessor: &mut impl Accessor,
        _send: &mut Self::Send<'_>,
    ) {
        b.0 = a.0 - c.0;
        b.1 = a.1 - c.1;
        b.2 = a.2 - c.2;
    }
}

struct SimpleParIterB;

impl RegistryParIter for SimpleParIterB {
    type Global<'g> = ();
    type Local<'l> = (&'l ComponentA, &'l mut ComponentB, &'l ComponentC);
    type Send<'s> = ();

    fn iter(
        &self,
        (a, b, c): Self::Local<'_>,
        _accessor: &mut impl Accessor,
        _send: &mut Self::Send<'_>,
    ) {
        b.0 = a.0 - c.0;
        b.1 = a.1 - c.1;
        b.2 = a.2 - c.2;
    }
}

struct SimpleIterC;

impl RegistryIter for SimpleIterC {
    type Global<'g> = ();
    type Local<'l> = (&'l ComponentA, &'l ComponentB, &'l mut ComponentC);
    type Send<'s> = ();

    fn iter(
        &mut self,
        (a, b, c): Self::Local<'_>,
        _accessor: &mut impl Accessor,
        _send: &mut Self::Send<'_>,
    ) {
        c.0 = cmp::min(a.0, b.0);
        c.1 = cmp::min(a.1, b.1);
        c.2 = cmp::min(a.2, b.2);
    }
}

struct SimpleParIterC;

impl RegistryParIter for SimpleParIterC {
    type Global<'g> = ();
    type Local<'l> = (&'l ComponentA, &'l ComponentB, &'l mut ComponentC);
    type Send<'s> = ();

    fn iter(
        &self,
        (a, b, c): Self::Local<'_>,
        _accessor: &mut impl Accessor,
        _send: &mut Self::Send<'_>,
    ) {
        c.0 = cmp::min(a.0, b.0);
        c.1 = cmp::min(a.1, b.1);
        c.2 = cmp::min(a.2, b.2);
    }
}

pub struct Benchmark {
    pub reg: Registry,
    pub pipe: bool,
    pub par: bool,
}

impl Benchmark {
    pub fn new(count: usize, pipe: bool, par: bool) -> Self {
        let mut reg = Registry::new();
        for _ in 0..count {
            let e = reg.push((
                ComponentA(0, 0, 0),
                ComponentB(1, 1, 1),
                ComponentC(2, 2, 2),
                ComponentD(3, 3, 3),
            ));
        }
        Self { reg, pipe, par }
    }

    pub fn run(&mut self) {
        if self.par {
            let mut a = SimpleParIterA;
            let mut b = SimpleParIterB;
            let mut c = SimpleParIterC;
            if self.pipe {
                let mut pipe = ParPipeline::new((&mut a, &mut b, &mut c), 1024);
                pipe.runnable().run(&mut self.reg);
            } else {
                a.runnable().run(&mut self.reg);
                b.runnable().run(&mut self.reg);
                c.runnable().run(&mut self.reg);
            }
        } else {
            let mut a = SimpleIterA;
            let mut b = SimpleIterB;
            let mut c = SimpleIterC;
            if self.pipe {
                let mut pipe = Pipeline::new((&mut a, &mut b, &mut c), 1024);
                pipe.runnable().run(&mut self.reg);
            } else {
                a.runnable().run(&mut self.reg);
                b.runnable().run(&mut self.reg);
                c.runnable().run(&mut self.reg);
            }
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            Self::run(black_box(self));
        });
    }
}
