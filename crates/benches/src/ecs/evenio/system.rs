use criterion::{black_box, Bencher, Criterion};
use evenio::{
    event::Receiver,
    prelude::Fetcher,
    rayon::iter::{IntoParallelIterator, ParallelIterator},
};

use crate::ecs::evenio::common::*;

pub struct Benchmark {
    pub world: evenio::world::World,
}

fn simple_handler(
    _: Receiver<Tick>,
    entities: Fetcher<(&mut ComponentA, &ComponentB, &ComponentC)>,
) {
    for (a, b, c) in entities {
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    }
}

fn simple_par_handler(
    _: Receiver<Tick>,
    entities: Fetcher<(&mut ComponentA, &ComponentB, &ComponentC)>,
) {
    entities.into_par_iter().for_each(|(a, b, c)| {
        a.0 = b.0 + c.0;
        a.1 = b.1 + c.1;
        a.2 = b.2 + c.2;
    });
}

impl Benchmark {
    pub fn new(count: usize, par: bool) -> Self {
        let mut world = evenio::world::World::new();
        for _ in 0..count {
            let e = world.spawn();
            world.insert(e, ComponentA(0, 0, 0));
            world.insert(e, ComponentB(1, 1, 1));
            world.insert(e, ComponentC(2, 2, 2));
            world.insert(e, ComponentD(3, 3, 3));
        }
        if par {
            world.add_handler(simple_par_handler);
        } else {
            world.add_handler(simple_handler);
        }
        Self { world }
    }

    pub fn run(&mut self) {
        self.world.send(Tick);
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            Self::run(black_box(self));
        });
    }
}
