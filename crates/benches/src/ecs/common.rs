use serverx_ecs::component::Component;

#[derive(Copy, Clone, Debug)]
pub struct ComponentA(pub i64, pub i64, pub i64);

impl Component for ComponentA {}

#[derive(Copy, Clone, Debug)]
pub struct ComponentB(pub i64, pub i64, pub i64);

impl Component for ComponentB {}

#[derive(Copy, Clone, Debug)]
pub struct ComponentC(pub i64, pub i64, pub i64);

impl Component for ComponentC {}

#[derive(Copy, Clone, Debug)]
pub struct ComponentD(pub i64, pub i64, pub i64);

impl Component for ComponentD {}
