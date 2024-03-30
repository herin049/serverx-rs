use criterion::{black_box, Bencher, Criterion};
use serverx_ecs::registry::Registry;

use crate::ecs::common::*;

pub struct Benchmark {
    pub count: usize,
}

impl Benchmark {
    pub fn new(count: usize) -> Self {
        Self { count }
    }

    pub fn run(&mut self, reg: &mut Registry) {
        for _ in 0..self.count {
            reg.push((
                ComponentA(0, 0, 0),
                ComponentB(1, 1, 1),
                ComponentC(2, 2, 2),
                ComponentD(3, 3, 3),
            ));
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            let mut reg = Registry::new();
            Self::run(black_box(self), black_box(&mut reg));
        });
    }
}
