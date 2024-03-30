use criterion::{black_box, Bencher, Criterion};
use rand::{prelude::SliceRandom, thread_rng};
use serverx_ecs::{
    entity::Entity,
    registry::Registry,
    system::{System, SystemAccessor, SystemPar},
};

use crate::ecs::common::*;

struct SimpleSystem;

impl<'a> System<'a> for SimpleSystem {
    type Global = ();
    type Local = (&'a mut ComponentA, &'a ComponentB, &'a ComponentC);


    fn run(
        &self,
        _entity: Entity,
        components: Self::Local,
        _accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
    ) {
        let (a, b, c) = components;
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    }
}

impl<'a> SystemPar<'a> for SimpleSystem {
    type Ctx = ();
    type Global = ();
    type Local = (&'a mut ComponentA, &'a ComponentB, &'a ComponentC);

    fn par_ctx(&'a self) -> Self::Ctx {
        ()
    }

    fn run(
        &'a self,
        _entity: Entity,
        components: Self::Local,
        _accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
        _ctx: &mut Self::Ctx,
    ) {
        let (a, b, c) = components;
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
            self.reg.run_par(&SimpleSystem);
        } else {
            self.reg.run(&SimpleSystem);
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            Self::run(black_box(self));
        });
    }
}
