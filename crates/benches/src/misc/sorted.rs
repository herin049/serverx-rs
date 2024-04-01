use std::collections::BTreeSet;
use criterion::{Bencher, black_box};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serverx_ecs::registry::Registry;
use crate::ecs::common::{ComponentA, ComponentB, ComponentC, ComponentD};

pub struct Benchmark {
    pub count: usize,
    pub values: Vec<i64>
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

    pub fn run(&mut self, vec: &mut Vec<i64>) {
        for i in 0..self.count {
            vec.push(i as i64);
        }
        vec.sort_unstable();
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            let mut vec = Vec::<i64>::new();
            Self::run(black_box(self), black_box(&mut vec));
        });
    }
}
