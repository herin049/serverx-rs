use criterion::{black_box, Bencher, Criterion};
use serverx_ecs::{
    entity::Entity,
    execution::{
        iter::{SystemIter, SystemParIter},
        run::{Runnable, RunnablePar},
    },
    registry::{access::Accessor, Registry},
};

use crate::ecs::common::*;

struct SimpleSystem;

impl<'a> SystemIter<'a> for SimpleSystem {
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

struct SimpleSystemPar;

impl<'a> SystemParIter<'a> for SimpleSystemPar {
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

pub struct Benchmark {
    pub reg: Registry,
    pub par: bool,
}

impl Benchmark {
    pub fn new(count: usize, par: bool) -> Self {
        let mut reg = Registry::new();
        for _ in 0..count {
            let e = reg.push((
                ComponentA(0, 0, 0),
                ComponentB(1, 1, 1),
                ComponentC(2, 2, 2),
                ComponentD(3, 3, 3),
            ));
        }
        Self { reg, par }
    }

    pub fn run(&mut self) {
        if self.par {
            let mut sys = SimpleSystemPar;
            sys.runnable().run(&mut self.reg);
        } else {
            let mut sys = SimpleSystem;
            sys.runnable().run(&mut self.reg);
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            Self::run(black_box(self));
        });
    }
}
