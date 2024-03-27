use std::{cell::RefCell, ops::DerefMut, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{seq::SliceRandom, thread_rng};
use serverx_ecs::{
    entity::Entity,
    system::{SystemAccessor, SystemPar},
    world::World,
};
use thread_local::ThreadLocal;

#[derive(Copy, Clone, Debug)]
pub struct Position(i32, i32, i32);

#[derive(Copy, Clone, Debug)]
pub struct Velocity(i32, i32, i32);

#[derive(Debug)]
pub struct Health(f64);

#[derive(Debug)]
pub struct Bounds(f64, f64);

impl evenio::prelude::Component for Position {}
impl evenio::prelude::Component for Velocity {}
impl evenio::prelude::Component for Health {}
impl evenio::prelude::Component for Bounds {}

pub struct TestSystem1;

pub struct TestSystem2 {
    tls: ThreadLocal<RefCell<u64>>,
}

impl<'a> SystemPar<'a> for TestSystem2 {
    type Ctx = &'a RefCell<u64>;
    type Global = ();
    type Local = (&'a mut Position, &'a Velocity);

    fn par_ctx(&'a self) -> Self::Ctx {
        self.tls.get_or(|| RefCell::new(0))
    }

    fn run(
        &'a self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
        ctx: &mut Self::Ctx,
    ) {
        let (p, v) = components;
        p.0 += v.0;
        p.1 += v.1;
        p.2 += v.2;
        *(*ctx).borrow_mut().deref_mut() += 1;
    }
}

impl<'a> SystemPar<'a> for TestSystem1 {
    type Ctx = ();
    type Global = ();
    type Local = (&'a mut Position, &'a Velocity);

    fn par_ctx(&'a self) -> Self::Ctx {
        ()
    }

    fn run(
        &'a self,
        entity: Entity,
        components: Self::Local,
        _accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
        _ctx: &mut Self::Ctx,
    ) {
        let (p, v) = components;
        p.0 += v.0;
        p.1 += v.1;
        p.2 += v.2;
    }
}

pub fn par_system_bench2(c: &mut Criterion) {
    let mut world = World::new();
    for _ in 0..100000 {
        world.push((
            Position(0, 0, 0),
            Velocity(1, 1, 1),
            Health(10.0),
            Bounds(1.0, 1.0),
        ));
    }
    c.bench_function("par system 2 100000", |b| {
        b.iter(|| {
            let system = TestSystem2 {
                tls: ThreadLocal::new(),
            };
            World::run_par(black_box(&mut world), black_box(&system));
        })
    });
}
pub fn par_system_bench(c: &mut Criterion) {
    let mut world = World::new();
    for _ in 0..100000 {
        world.push((
            Position(0, 0, 0),
            Velocity(1, 1, 1),
            Health(10.0),
            Bounds(1.0, 1.0),
        ));
    }
    let system = TestSystem1;
    c.bench_function("par system 100000", |b| {
        b.iter(|| {
            World::run_par(black_box(&mut world), black_box(&system));
        })
    });
}

pub fn evenio_random_access(
    world: &mut evenio::world::World,
    entities: &[evenio::entity::EntityId],
) {
    for e in entities {
        let velocity = world.get::<Velocity>(*e).map(|x| *x);
        if let Some(v) = velocity {
            if let Some(p) = world.get_mut::<Position>(*e) {
                p.0 += v.0;
                p.1 += v.1;
                p.2 += v.2;
            }
        }
    }
}

pub fn evenio_random_access_bench(c: &mut Criterion) {
    let mut world = evenio::world::World::new();
    let mut entities = Vec::new();
    for _ in 0..100000 {
        let e = world.spawn();
        world.insert(e, Position(0, 0, 0));
        world.insert(e, Velocity(1, 1, 1));
        world.insert(e, Health(10.0));
        world.insert(e, Bounds(1.0, 1.0));
        entities.push(e);
    }
    entities.shuffle(&mut thread_rng());
    c.bench_function("evenio random access 100000", |b| {
        b.iter(|| {
            evenio_random_access(black_box(&mut world), black_box(entities.as_slice()));
        });
    });
}

pub fn random_access(world: &mut World, entities: &[Entity]) {
    for e in entities {
        if let Some((position, velocity)) = world.get_mut::<(&mut Position, &Velocity)>(*e) {
            position.0 += velocity.0;
            position.1 += velocity.1;
            position.2 += velocity.2;
        }
    }
}

pub fn random_access_bench(c: &mut Criterion) {
    let mut world = World::new();
    let mut entities = Vec::new();
    for _ in 0..100000 {
        let e = world.push((
            Position(0, 0, 0),
            Velocity(1, 1, 1),
            Health(10.0),
            Bounds(1.0, 1.0),
        ));
        entities.push(e);
    }
    entities.shuffle(&mut thread_rng());
    c.bench_function("world random access 100000", |b| {
        b.iter(|| {
            random_access(black_box(&mut world), black_box(entities.as_slice()));
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = par_system_bench, par_system_bench2
    // targets = random_access_bench, evenio_random_access_bench
    // targets = world_bench, evenio_bench, world_iter_bench
    // targets = type_sort_benchmark
);
criterion_main!(benches);
// use std::time::Duration;
//
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use serverx_benches::fibonacci;
// use serverx_block::{
//     blocks::Block,
//     states::{BlockState},
// };
//
// pub fn push_blocks(states: &[BlockState], blocks: &mut Vec<Block>) {
//     for s in states {
//         blocks.push((*s).into());
//     }
// }
//
// pub fn block_benchmark(c: &mut Criterion) {
//     let mut states = Vec::with_capacity(BlockState::COUNT);
//     for i in 0..BlockState::COUNT {
//         states.push(BlockState::try_from(i).unwrap());
//     }
//     c.bench_function("push blocks all", |b| {
//         b.iter(|| {
//             let mut blocks = Vec::with_capacity(BlockState::COUNT);
//             push_blocks(black_box(states.as_slice()), black_box(&mut blocks));
//         })
//     });
// }
//
// pub fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
// }
//
// criterion_group!(
//     name = benches;
//     config = Criterion::default().measurement_time(Duration::from_secs(10));
//     targets = block_benchmark
//     // targets = world_bench, evenio_bench, world_iter_bench
//     // targets = type_sort_benchmark
// );
// criterion_main!(benches);
