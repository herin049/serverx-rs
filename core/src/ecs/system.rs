use crate::ecs::{entity::Entity, tuple::ComponentBorrowTuple, world::WorldAccessor};

pub trait System<'a>: Clone + Send + Sync {
    type Local: ComponentBorrowTuple;
    type Global: ComponentBorrowTuple;

    fn run(&self, entity: Entity, components: Self::Local, world_accessor: &WorldAccessor);
    fn run_mut(
        &mut self,
        entity: Entity,
        components: Self::Local,
        world_accessor: &mut WorldAccessor,
    );
}
