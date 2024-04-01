use criterion::{black_box, Bencher, Criterion};
use rand::{prelude::SliceRandom, thread_rng};
use serverx_ecs::{entity::Entity, registry::Registry};

use crate::ecs::common::*;

pub struct Benchmark {
    pub reg: Registry,
    pub entities: Vec<Entity>,
}

impl Benchmark {
    pub fn new(count: usize, shuffle: bool) -> Self {
        let mut reg = Registry::new();
        let mut entities = Vec::new();
        for _ in 0..count {
            let e = reg.push((
                ComponentA(0, 0, 0),
                ComponentB(1, 1, 1),
                ComponentC(2, 2, 2),
                ComponentD(3, 3, 3),
            ));
            entities.push(e);
        }
        if shuffle {
            entities.shuffle(&mut thread_rng());
        }
        Self { reg, entities }
    }

    pub fn run(&mut self) {
        for e in &self.entities {
            if let Some((a, b, c)) = self
                .reg
                .get_mut::<(&mut ComponentA, &ComponentB, &ComponentC)>(*e)
            {
                a.0 = b.0 + c.0;
                a.1 = b.1 + c.1;
                a.2 = b.2 + c.2;
            }
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            Self::run(black_box(self));
        });
    }
}
