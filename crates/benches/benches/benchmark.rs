use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serverx_benches::fibonacci;
use serverx_block::blocks::Block;
use serverx_block::states::{BlockState, STATE_COUNT};

pub fn push_blocks(states: &[BlockState], blocks: &mut Vec<Block>) {
    for s in states {
        blocks.push(s.into());
    }
}

pub fn block_benchmark(c: &mut Criterion) {
    let mut states = Vec::with_capacity(STATE_COUNT as usize);
    for i in 0..STATE_COUNT {
        states.push(BlockState::try_from(i).unwrap());
    }
    c.bench_function("push blocks all", |b| b.iter(|| {
        let mut blocks = Vec::with_capacity(STATE_COUNT as usize);
        push_blocks(black_box(states.as_slice()), black_box(&mut blocks));
    }));
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}


criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = block_benchmark
    // targets = world_bench, evenio_bench, world_iter_bench
    // targets = type_sort_benchmark
);
criterion_main!(benches);