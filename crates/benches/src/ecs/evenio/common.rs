use evenio::{component::Component, event::Event};

#[derive(Copy, Clone, Debug, Component)]
pub struct ComponentA(pub i64, pub i64, pub i64);

#[derive(Copy, Clone, Debug, Component)]
pub struct ComponentB(pub i64, pub i64, pub i64);

#[derive(Copy, Clone, Debug, Component)]
pub struct ComponentC(pub i64, pub i64, pub i64);

#[derive(Copy, Clone, Debug, Component)]
pub struct ComponentD(pub i64, pub i64, pub i64);

#[derive(Copy, Clone, Debug, Event)]
pub struct Tick;
