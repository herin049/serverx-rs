
use std::ops::{Add, Sub};
use std::simd::f32x4;

use criterion::{black_box, Bencher};
use glam::Vec3;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use serverx_hyperion_bvh::aabb::Aabb;
use serverx_hyperion_bvh::{Bvh, HasAabb, TrivialHeuristic};

#[derive(Copy, Clone, Debug)]
pub struct Primative {
    name: &'static str,
    bb: Aabb
}

impl HasAabb for Primative {
    fn aabb(&self) -> Aabb {
        self.bb
    }
}

pub struct Benchmark {
    count: usize,
    elements: Vec<Aabb>,
}

impl Benchmark {
    pub fn new(count: usize) -> Self {
        let mut elements = Vec::new();
        let mut rng = thread_rng();
        let bound = Uniform::new(0.0, 5.0);
        let pos = Uniform::new(-100.0, 100.0);
        for _ in 0..count {
            let pos = Vec3::new(pos.sample(&mut rng), pos.sample(&mut rng), pos.sample(&mut rng));
            let width = Vec3::new(bound.sample(&mut rng), bound.sample(&mut rng), bound.sample(&mut rng));
            let aabb = Aabb::new(pos.sub(width), pos.add(width));
            elements.push(aabb);
        }

        Self {
            count,
            elements,
        }
    }

    pub fn run(&mut self, mut primatives: Vec<Primative>) {
        // self.bvh.build();
        Bvh::<Primative>::build::<TrivialHeuristic>(primatives);
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            let mut primatives = Vec::new();
            for e in &self.elements {
                primatives.push(Primative { name: "primative", bb: *e });
            }
            Self::run(black_box(self), black_box(primatives));
        });
    }
}
