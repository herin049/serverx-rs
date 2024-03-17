use crate::ecs::{
    component::ComponentSet,
    entity::Entity,
    storage::archetype::ArchetypeChunk,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    world::World,
};

pub struct SystemPermissions {
    pub read_local: ComponentSet,
    pub write_local: ComponentSet,
    pub read_global: ComponentSet,
    pub write_global: ComponentSet,
}

pub struct SystemAccessor<'a> {
    pub world: &'a World,
    pub permissions: &'a SystemPermissions,
    current: Entity,
}

impl<'a> SystemAccessor<'a> {
    pub fn new(world: &'a World, permissions: &'a SystemPermissions) -> Self {
        Self {
            world,
            permissions,
            current: Entity::default(),
        }
    }

    pub fn get_components<'b, T: ComponentRefTuple<'b>>(&'b self, entity: Entity) -> Option<T> {
        if entity == self.current
            && !T::ValueType::COMPONENT_SET.disjoint(&self.permissions.write_local)
        {
            panic!("read types alias with local write components");
        } else if !T::READ_COMPONENT_SET.subset2(
            &self.permissions.read_global,
            &self.permissions.write_global,
        ) {
            panic!("invalid read");
        } else {
            if let Some(archetype) = self.world.archetypes.get(entity.archetype_id() as usize) {
                if T::ValueType::COMPONENT_SET.subset(&archetype.component_set) {
                    unsafe {
                        return archetype.get_components_mut(entity);
                    }
                }
            }
            None
        }
    }

    pub fn get_components_mut<'b, T: ComponentBorrowTuple<'b>>(
        &'b mut self,
        entity: Entity,
    ) -> Option<T> {
        if entity == self.current
            && !T::ValueType::COMPONENT_SET
                .disjoint2(&self.permissions.write_local, &self.permissions.read_local)
        {
            panic!("write types alias with local read/write components");
        } else if !T::READ_COMPONENT_SET.subset2(
            &self.permissions.read_global,
            &self.permissions.write_global,
        ) && !T::WRITE_COMPONENT_SET.subset(&self.permissions.write_global)
        {
            panic!("invalid read");
        } else {
            if let Some(archetype) = self.world.archetypes.get(entity.archetype_id() as usize) {
                if T::ValueType::COMPONENT_SET.subset(&archetype.component_set) {
                    unsafe {
                        return archetype.get_components_mut(entity);
                    }
                }
            }
            None
        }
    }

    pub unsafe fn set_current(&mut self, entity: Entity) {
        self.current = entity;
    }
}

pub trait System<'a> {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentRefTuple<'a>;
    fn run(&self, entity: Entity, components: Self::Local, system_accessor: &mut SystemAccessor);
}

pub trait SystemMut<'a> {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentBorrowTuple<'a>;
    fn run(
        &mut self,
        entity: Entity,
        components: Self::Local,
        system_accessor: &mut SystemAccessor,
    );
}

