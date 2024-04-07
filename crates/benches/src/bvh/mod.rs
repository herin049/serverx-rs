pub mod hyperion;

use std::ops::{Add, Sub};
use std::simd::f32x4;

use criterion::{black_box, Bencher};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use serverx_bvh::{Aabb, Bvh, Vec3};

pub struct Benchmark {
    count: usize,
    bvh: Bvh<&'static str>,
    elements: Vec<Aabb>,
}

impl Benchmark {
    pub fn new(count: usize) -> Self {
        let mut bvh = Bvh::new(16);
        let mut elements = Vec::new();
        let mut rng = thread_rng();
        let bound = Uniform::new(0.0, 5.0);
        let pos = Uniform::new(-100.0, 100.0);
        for _ in 0..count {
            let pos = Vec3::new(pos.sample(&mut rng), pos.sample(&mut rng), pos.sample(&mut rng));
            let width = Vec3::new(bound.sample(&mut rng), bound.sample(&mut rng), bound.sample(&mut rng));
            let aabb = Aabb {
                from: pos.sub(width),
                to: pos.add(width),
            };
            elements.push(aabb);
        }

        Self {
            count,
            bvh,
            elements,
        }
    }

    pub fn run(&mut self) {
        // self.bvh.build();
        self.bvh.build_par();
    }

    pub fn iter(&mut self, b: &mut Bencher) {
        b.iter(|| {
            self.bvh.clear();
            for e in &self.elements {
                self.bvh.insert("primative", *e);
            }
            Self::run(black_box(self));
        });
    }
}
