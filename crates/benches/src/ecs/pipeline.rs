use std::cmp;

use criterion::{black_box, Bencher, Criterion};
use serverx_ecs::{
    entity::Entity,
    execution::{
        iter::{SystemIter, SystemParIter},
        pipeline::{ParPipeline, Pipeline, SystemParPipeline, SystemPipeline},
        run::{Runnable, RunnablePar},
    },
    registry::{access::Accessor, Registry},
};

use crate::ecs::common::*;

struct SimpleIterA;

impl<'a> SystemIter<'a> for SimpleIterA {
    type Global = ();
    type Local = (&'a mut ComponentA, &'a ComponentB, &'a ComponentC);

    fn iter(
        &mut self,
        (a, b, c): Self::Local,
        _accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    ) {
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    }
}

struct SimpleParIterA;

impl<'a> SystemParIter<'a> for SimpleParIterA {
    type Global = ();
    type Local = (&'a mut ComponentA, &'a ComponentB, &'a ComponentC);

    fn iter(
        &self,
        (a, b, c): Self::Local,
        _accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    ) {
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    }
}

struct SimpleIterB;

impl<'a> SystemIter<'a> for SimpleIterB {
    type Global = ();
    type Local = (&'a ComponentA, &'a mut ComponentB, &'a ComponentC);

    fn iter(
        &mut self,
        (a, b, c): Self::Local,
        _accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    ) {
        b.0 = a.0 - c.0;
        b.1 = a.1 - c.1;
        b.2 = a.2 - c.2;
    }
}

struct SimpleParIterB;

impl<'a> SystemParIter<'a> for SimpleParIterB {
    type Global = ();
    type Local = (&'a ComponentA, &'a mut ComponentB, &'a ComponentC);

    fn iter(
        &self,
        (a, b, c): Self::Local,
        _accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    ) {
        b.0 = a.0 - c.0;
        b.1 = a.1 - c.1;
        b.2 = a.2 - c.2;
    }
}

struct SimpleIterC;

impl<'a> SystemIter<'a> for SimpleIterC {
    type Global = ();
    type Local = (&'a ComponentA, &'a ComponentB, &'a mut ComponentC);

    fn iter(
        &mut self,
        (a, b, c): Self::Local,
        _accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
    ) {
        c.0 = cmp::min(a.0, b.0);
        c.1 = cmp::min(a.1, b.1);
        c.2 = cmp::min(a.2, b.2);
    }
}

struct SimpleParIterC;

impl<'a> SystemParIter<'a> for SimpleParIterC {
    type Global = ();
    type Local = (&'a ComponentA, &'a ComponentB, &'a mut ComponentC);

    fn iter(
        &self,
        (a, b, c): Self::Local,
        _accessor: &mut Accessor<'_, 'a, Self::Local, Self::Global>,
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
