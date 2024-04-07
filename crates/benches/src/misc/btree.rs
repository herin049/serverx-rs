use std::collections::BTreeSet;

use criterion::{black_box, Bencher};
use rand::{seq::SliceRandom, thread_rng};
use serverx_ecs::registry::Registry;

use crate::ecs::common::{ComponentA, ComponentB, ComponentC, ComponentD};

pub struct Benchmark {
    pub count: usize,
    pub values: Vec<i64>,
}

impl Benchmark {
    pub fn new(count: usize) -> Self {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            values.push(i as i64);
        }
        values.shuffle(&mut thread_rng());
        Self { count, values }
    }

    pub fn run(&mut self, set: &mut BTreeSet<i64>) {
        for i in 0..self.count {
            set.insert(i as i64);
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            let mut set = BTreeSet::<i64>::new();
            Self::run(black_box(self), black_box(&mut set));
        });
    }
}
