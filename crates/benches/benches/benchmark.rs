use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use serverx_benches::ecs;

pub fn registry_push(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry push {}", count);
    let mut bench = ecs::push::Benchmark { count };
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_random(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry random {}", count);
    let mut bench = ecs::random::Benchmark::new(count, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_random_seq(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry random seq {}", count);
    let mut bench = ecs::random::Benchmark::new(count, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_system(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry system {}", count);
    let mut bench = ecs::system::Benchmark::new(count, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_system_par(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry system par {}", count);
    let mut bench = ecs::system::Benchmark::new(count, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_system(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio system {}", count);
    let mut bench = ecs::evenio::system::Benchmark::new(count, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_system_par(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio system par {}", count);
    let mut bench = ecs::evenio::system::Benchmark::new(count, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = registry_system, registry_system_par, evenio_system, evenio_system_par
    // targets = registry_random_seq, registry_random
    // targets = registry_push
);
criterion_main!(benches);
