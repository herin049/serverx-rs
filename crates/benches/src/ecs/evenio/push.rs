use criterion::{black_box, Bencher};
use evenio::world::World;

use crate::ecs::evenio::common::{ComponentA, ComponentB, ComponentC, ComponentD};

pub struct Benchmark {
    pub count: usize,
}

impl Benchmark {
    pub fn new(count: usize) -> Self {
        Self { count }
    }

    pub fn run(&mut self, world: &mut World) {
        for _ in 0..self.count {
            let e = world.spawn();
            world.insert(e, ComponentA(0, 0, 0));
            world.insert(e, ComponentB(1, 1, 1));
            world.insert(e, ComponentC(2, 2, 2));
            world.insert(e, ComponentD(3, 3, 3));
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            let mut reg = World::new();
            Self::run(black_box(self), black_box(&mut reg));
        });
    }
}
