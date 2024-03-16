use crate::ecs::{
    component::ComponentSet,
    entity::Entity,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    world::World,
};

pub struct WorldPermissions {
    pub read: ComponentSet,
    pub write: ComponentSet,
}

pub struct SystemAccessor<'a> {
    pub world: &'a World,
    pub permissions: &'a WorldPermissions,
    pub current_entity: (Entity, ComponentSet),
}

impl<'a> SystemAccessor<'a> {
    pub fn has_components<T: ComponentTuple>(&self, entity: Entity) -> bool {
        self.world.has_components::<T>(entity)
    }

    pub fn get_components<'b, T: ComponentRefTuple<'b>>(&'b self, entity: Entity) -> Option<T> {
        if self.current_entity.0 == entity
            && !T::ValueType::COMPONENT_SET.disjoint(&self.current_entity.1)
        {
            panic!("attempt to get components from already referenced entity in system accessor");
        } else if !T::READ_COMPONENT_SET.subset2(&self.permissions.read, &self.permissions.write) {
            panic!("invalid read");
        } else {
            if let Some(archetype) = self.world.archetypes.get(entity.archetype() as usize) {
                if T::ValueType::COMPONENT_SET.subset(&archetype.components) {
                    unsafe {
                        return Some(archetype.storage.get_components_unchecked(entity.index()));
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
        if self.current_entity.0 == entity
            && !T::ValueType::COMPONENT_SET.disjoint(&self.current_entity.1)
        {
            panic!("attempt to get components from already referenced entity in system accessor");
        } else if !T::READ_COMPONENT_SET.subset2(&self.permissions.read, &self.permissions.write)
            || !T::WRITE_COMPONENT_SET.subset(&self.permissions.write)
        {
            panic!("invalid read");
        } else {
            if let Some(archetype) = self.world.archetypes.get(entity.archetype() as usize) {
                if T::ValueType::COMPONENT_SET.subset(&archetype.components) {
                    unsafe {
                        return Some(
                            archetype
                                .storage
                                .get_components_mut_unchecked(entity.index()),
                        );
                    }
                }
            }
            None
        }
    }

}
