use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use serverx_benches::{ecs, misc};

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
    let id_str = format!("registry execution {}", count);
    let mut bench = ecs::system::Benchmark::new(count, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_system_par(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry execution par {}", count);
    let mut bench = ecs::system::Benchmark::new(count, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_pipeline(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry pipeline execution {}", count);
    let mut bench = ecs::pipeline::Benchmark::new(count, true, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_no_pipeline(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry no pipeline execution {}", count);
    let mut bench = ecs::pipeline::Benchmark::new(count, false, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_par_pipeline(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry par pipeline execution {}", count);
    let mut bench = ecs::pipeline::Benchmark::new(count, true, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn registry_par_no_pipeline(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("registry par no pipeline execution {}", count);
    let mut bench = ecs::pipeline::Benchmark::new(count, false, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_push(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio push {}", count);
    let mut bench = ecs::evenio::push::Benchmark::new(count);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_random(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio random {}", count);
    let mut bench = ecs::evenio::random::Benchmark::new(count, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_random_seq(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio random seq {}", count);
    let mut bench = ecs::evenio::random::Benchmark::new(count, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_system(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio execution {}", count);
    let mut bench = ecs::evenio::system::Benchmark::new(count, false);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn evenio_system_par(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("evenio execution par {}", count);
    let mut bench = ecs::evenio::system::Benchmark::new(count, true);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn btree_insert(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("btree insert {}", count);
    let mut bench = misc::btree::Benchmark::new(count);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

pub fn sorted(c: &mut Criterion) {
    let count: usize = 100000;
    let id_str = format!("sorted insert {}", count);
    let mut bench = misc::sorted::Benchmark::new(count);
    c.bench_function(id_str.as_str(), |b| bench.iter(b));
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    // targets = btree_insert, sorted
    // targets = registry_pipeline, registry_no_pipeline, registry_par_pipeline, registry_par_no_pipeline
    // targets = registry_system, registry_system_par
    // targets = registry_system, registry_system_par, registry_system_runnable, registry_system_par_runnable
    // targets = registry_system, registry_system_par, evenio_system, evenio_system_par
    // targets = registry_random, evenio_random, registry_random_seq, evenio_random_seq
    targets = registry_push, evenio_push
    // targets = registry_push, registry_random, registry_random_seq
    // targets = registry_system, evenio_system, registry_system_par, evenio_system_par
    // targets = registry_system, registry_system_par, evenio_system, evenio_system_par
    // targets = registry_random_seq, registry_random
    // targets = registry_push
);
criterion_main!(benches);
