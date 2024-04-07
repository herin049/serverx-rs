use criterion::{black_box, Bencher, Criterion};
use serverx_ecs::{
    entity::Entity,
    execution::{
        iter::{RegistryIter, RegistryParIter},
        run::{Runnable, RunnablePar},
    },
    registry::{
        access::{Accessor, IterAccessor},
        Registry,
    },
};

use crate::ecs::common::*;

struct SimpleSystem;

impl RegistryIter for SimpleSystem {
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

struct SimpleSystemPar;

impl RegistryParIter for SimpleSystemPar {
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
