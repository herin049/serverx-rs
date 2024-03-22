use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use serverx_core::ecs::{
    entity::Entity,
    storage::archetype::ArchetypeStorage,
    system::{System, SystemAccessor},
    world::World,
};

#[derive(Debug)]
pub struct Position(i32, i32, i32);
#[derive(Debug)]
pub struct Velocity(i32, i32, i32);
#[derive(Debug)]
pub struct Name(&'static str);
#[derive(Debug)]
pub struct Health(f64);

impl evenio::component::Component for Position {}
impl evenio::component::Component for Velocity {}
impl evenio::component::Component for Name {}
impl evenio::component::Component for Health {}
struct Tick;
impl evenio::event::Event for Tick {}

pub fn evenio_push(world: &mut evenio::world::World, count: usize) {
    for _ in 0..count {
        let e = world.spawn();
        world.insert(e, Position(1, 2, 3));
        world.insert(e, Velocity(4, 5, 6));
        world.insert(e, Name("foobar"));
        world.insert(e, Health(123.456));
    }
}

pub fn evenio_bench(criterion: &mut Criterion) {
    criterion.bench_function("evenio push 10000", |b| {
        b.iter(|| {
            let mut world = evenio::world::World::new();
            evenio_push(black_box(&mut world), black_box(10000));
        })
    });
}
fn update_positions(
    _: evenio::prelude::Receiver<Tick>,
    mut entities: evenio::prelude::Fetcher<(&mut Position, &Velocity)>,
) {
    // Loop over all entities with both the `Position` and `Velocity` components, and update their
    // positions.
    entities.into_par_iter().for_each(|x| {
        let (p, v) = x;
        p.0 += v.0;
        p.1 += v.1;
        p.2 += v.2;
    });
}
pub fn evenio_iter_bench(criterion: &mut Criterion) {
    let mut world = evenio::world::World::new();
    evenio_push(&mut world, 100000);
    criterion.bench_function("evenio world iter 100000", |b| {
        b.iter(|| {
            world.add_handler(update_positions);
            world.send(Tick);
        })
    });
}

pub fn world_push(world: &mut World, count: usize) {
    for _ in 0..count {
        unsafe {
            world.push((
                Position(1, 2, 3),
                Velocity(4, 5, 6),
                Name("foobar"),
                Health(123.456),
            ));
        }
    }
}

pub struct PhysicsSystem;

impl<'a> System<'a> for PhysicsSystem {
    type Global = ();
    type Local = (&'a Velocity, &'a mut Position);

    fn run(
        &self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
    ) {
        let (v, p) = components;
        p.0 += v.0;
        p.1 += v.1;
        p.2 += v.2;
    }
}

pub fn world_iter(world: &mut World) {
    let physics_system = PhysicsSystem;
    world.run_par(&physics_system);
}

pub fn world_iter_bench(criterion: &mut Criterion) {
    let mut world = World::new();
    world_push(&mut world, 100000);
    criterion.bench_function("world iter 100000", |b| {
        b.iter(|| {
            world_iter(black_box(&mut world));
        })
    });
}

pub fn world_bench(criterion: &mut Criterion) {
    criterion.bench_function("world push 10000", |b| {
        b.iter(|| {
            let mut world = World::new();
            world_push(black_box(&mut world), black_box(10000));
        })
    });
}

pub fn archetype_push(archetype: &mut ArchetypeStorage, count: usize) {
    for _ in 0..count {
        unsafe {
            archetype.push((
                Position(1, 2, 3),
                Velocity(4, 5, 6),
                Name("foobar"),
                Health(123.456),
            ));
        }
    }
}

pub fn archetype_bench(criterion: &mut Criterion) {
    criterion.bench_function("archetype push 10000", |b| {
        b.iter(|| {
            let mut archetype = ArchetypeStorage::new::<(Position, Velocity, Name, Health)>(0);
            archetype_push(black_box(&mut archetype), black_box(10000));
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = world_iter_bench, evenio_iter_bench
    // targets = world_bench, evenio_bench, world_iter_bench
    // targets = type_sort_benchmark
);
criterion_main!(benches);
