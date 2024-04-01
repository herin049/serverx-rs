use criterion::{black_box, Bencher};
use evenio::{entity::EntityId, world::World};
use rand::{seq::SliceRandom, thread_rng};

use crate::ecs::evenio::common::*;

pub struct Benchmark {
    pub world: World,
    pub entities: Vec<EntityId>,
}

impl Benchmark {
    pub fn new(count: usize, shuffle: bool) -> Self {
        let mut world = World::new();
        let mut entities = Vec::new();
        for _ in 0..count {
            let e = world.spawn();
            world.insert(e, ComponentA(0, 0, 0));
            world.insert(e, ComponentB(1, 1, 1));
            world.insert(e, ComponentC(2, 2, 2));
            world.insert(e, ComponentD(3, 3, 3));
            entities.push(e);
        }
        if shuffle {
            entities.shuffle(&mut thread_rng());
        }
        Self { world, entities }
    }

    pub fn run(&mut self) {
        for e in &self.entities {
            let (b, c) = (
                *self.world.get::<ComponentB>(*e).unwrap(),
                *self.world.get::<ComponentC>(*e).unwrap(),
            );
            let a = self.world.get_mut::<ComponentA>(*e).unwrap();
            a.0 = b.0 + c.0;
            a.1 = b.1 + c.1;
            a.2 = b.2 + c.2;
        }
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            Self::run(black_box(self));
        });
    }
}
