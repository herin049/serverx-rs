use std::marker::PhantomData;

use crate::{
    entity::Entity,
    sort::insertion_sort_noalias,
    tuple::{
        ptr::PtrTuple, type_tuple::TypeTuple, ComponentBorrowTuple, ComponentRefTuple,
        ComponentTuple,
    },
    types,
    world::World,
};

pub struct SystemAccessor<'a, L: ComponentBorrowTuple<'a>, G: ComponentBorrowTuple<'a>> {
    phantom: PhantomData<(L, G)>,
    world: &'a World,
    current_entity: Entity,
}

impl<'a, L: ComponentBorrowTuple<'a>, G: ComponentBorrowTuple<'a>> SystemAccessor<'a, L, G> {
    pub fn new(world: &'a World) -> Self {
        Self {
            phantom: PhantomData,
            world,
            current_entity: Entity::default(),
        }
    }

    #[inline(always)]
    pub unsafe fn update_entity(&mut self, entity: Entity) {
        self.current_entity = entity;
    }

    pub fn has_components<T: ComponentTuple>(&self, entity: Entity) -> bool {
        if let Some(archetype) = self.world.archetypes.get(entity.archetype_id() as usize) {
            types::subset(T::type_ids().as_ref(), archetype.type_ids.as_ref())
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.world.lookup_entity(entity).is_some()
    }

    pub fn get<'b, T: ComponentRefTuple<'b> + ComponentBorrowTuple<'b>>(
        &self,
        entity: Entity,
    ) -> Option<T>
    where
        'a: 'b,
    {
        let mut type_ids = <T as ComponentRefTuple<'b>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if entity == self.current_entity
            && !types::disjoint(type_ids.as_ref(), L::ValueType::type_ids().as_ref())
        {
            panic!("invalid read");
        } else if !types::subset(type_ids.as_ref(), G::ValueType::type_ids().as_ref()) {
            panic!("invalid read");
        }
        if let Some((archetype, index)) = self.world.lookup_entity(entity) {
            if let Ok(ptr) = archetype.try_as_mut_ptr::<<T as ComponentRefTuple<'b>>::ValueType>() {
                unsafe {
                    return Some(<T as ComponentRefTuple<'b>>::deref(
                        ptr.offset(index as isize),
                    ));
                }
            }
        }
        None
    }

    pub fn get_mut<'b, T: ComponentBorrowTuple<'b>>(&mut self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        let mut type_ids = <T as ComponentBorrowTuple<'b>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if entity == self.current_entity
            && !types::disjoint(type_ids.as_ref(), L::ValueType::type_ids().as_ref())
        {
            panic!("invalid write");
        } else if !types::subset(
            T::ReadType::type_ids().as_ref(),
            G::ValueType::type_ids().as_ref(),
        ) || !types::subset(
            T::WriteType::type_ids().as_ref(),
            G::WriteType::type_ids().as_ref(),
        ) {
            panic!("invalid write");
        }
        if let Some((archetype, index)) = self.world.lookup_entity(entity) {
            if let Ok(ptr) =
                archetype.try_as_mut_ptr::<<T as ComponentBorrowTuple<'b>>::ValueType>()
            {
                unsafe {
                    return Some(T::deref(ptr.offset(index as isize)));
                }
            }
        }
        None
    }
}

pub trait System<'a> {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentRefTuple<'a> + ComponentBorrowTuple<'a>;
    fn run(
        &self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
    );
}

pub trait SystemMut<'a> {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentBorrowTuple<'a>;
    fn run(
        &mut self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
    );
}
