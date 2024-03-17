use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use serverx_core::ecs::{
    component::Component, fibonacci, storage::archetype::ArchetypeStorage, tuple::ComponentTuple,
    world::World, ComponentId,
};

#[derive(Debug)]
struct Position(i32, i32, i32);
#[derive(Debug)]
struct Velocity(i32, i32, i32);
#[derive(Debug)]
struct Name(&'static str);
#[derive(Debug)]
struct Dimensions(u32, u32);

unsafe impl Component for Position {
    const ID: ComponentId = 0;
}

unsafe impl Component for Velocity {
    const ID: ComponentId = 1;
}

unsafe impl Component for Name {
    const ID: ComponentId = 2;
}

unsafe impl Component for Dimensions {
    const ID: ComponentId = 3;
}

pub fn vec_push(
    vecp: &mut Vec<Position>,
    vecv: &mut Vec<Velocity>,
    vecn: &mut Vec<Name>,
    vecd: &mut Vec<Dimensions>,
    count: usize,
) {
    for _ in 0..count {
        vecp.push(Position(1, 2, 3));
        vecv.push(Velocity(4, 5, 6));
        vecn.push(Name("test entity"));
        vecd.push(Dimensions(7, 8));
    }
}

pub fn vec_push_benchmark(c: &mut Criterion) {
    c.bench_function("vec push 10000", |b| {
        b.iter(|| {
            vec_push(
                black_box(&mut Vec::new()),
                black_box(&mut Vec::new()),
                black_box(&mut Vec::new()),
                black_box(&mut Vec::new()),
                black_box(10000),
            )
        })
    });
}

pub fn legion_push(world: &mut legion::World, count: usize) {
    for _ in 0..count {
        world.push((
            Position(1, 2, 3),
            Velocity(4, 5, 6),
            Name("test entity"),
            Dimensions(7, 8),
        ));
    }
}

pub fn legion_push_benchmark(c: &mut Criterion) {
    let mut world = legion::World::default();
    c.bench_function("legion push 10000", |b| {
        b.iter(|| legion_push(&mut world, black_box(10000)))
    });
}

fn archetype_raw_push(archetype: &mut ArchetypeStorage, count: usize) {
    for _ in 0..count {
        unsafe {
            <(Position, Velocity, Name, Dimensions) as ComponentTuple>::push_components(
                (
                    Position(1, 2, 3),
                    Velocity(4, 5, 6),
                    Name("test entity"),
                    Dimensions(7, 8),
                ),
                archetype,
            );
        }
    }
}

pub fn archetype_raw_push_benchmark(c: &mut Criterion) {
    let mut archetype = ArchetypeStorage::new::<(Position, Velocity, Name, Dimensions)>(0);
    c.bench_function("archetype raw push 10000", |b| {
        b.iter(|| archetype_raw_push(&mut archetype, black_box(10000)))
    });
}

fn archetype_push(archetype: &mut ArchetypeStorage, count: usize) {
    for _ in 0..count {
        unsafe {
            archetype.push((
                Position(1, 2, 3),
                Velocity(4, 5, 6),
                Name("test entity"),
                Dimensions(7, 8),
            ));
        }
    }
}

pub fn archetype_push_benchmark(c: &mut Criterion) {
    c.bench_function("archetype push 10000", |b| {
        b.iter(|| {
            let mut archetype = ArchetypeStorage::new::<(Position, Velocity, Name, Dimensions)>(0);
            archetype_push(&mut archetype, black_box(10000));
        })
    });
}

fn world_push(world: &mut World, count: usize) {
    for _ in 0..count {
        world.push((
            Position(1, 2, 3),
            Velocity(4, 5, 6),
            Name("test entity"),
            Dimensions(7, 8),
        ));
    }
}

pub fn world_push_benchmark(c: &mut Criterion) {
    c.bench_function("world push 10000", |b| {
        b.iter(|| {
            let mut world = black_box(World::new());
            world_push(&mut world, black_box(10000));
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = world_push_benchmark, archetype_push_benchmark, legion_push_benchmark
    // targets = vec_push_benchmark, archetype_raw_push_benchmark, archetype_push_benchmark, world_push_benchmark, legion_push_benchmark
);
criterion_main!(benches);
