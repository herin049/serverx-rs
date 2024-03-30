use std::marker::PhantomData;

use crate::{
    entity::Entity,
    registry::Registry,
    sort::insertion_sort_noalias,
    tuple::{
        ptr::PtrTuple, type_tuple::TypeTuple, ComponentBorrowTuple, ComponentRefTuple,
        ComponentTuple,
    },
    types,
};
use crate::event::Event;
use crate::tuple::EventTuple;

pub struct SystemAccessor<'a, 'b, L: ComponentBorrowTuple<'b>, G: ComponentBorrowTuple<'b>> {
    phantom: PhantomData<&'b (L, G)>,
    registry: &'a Registry,
    current_entity: Entity,
    par: bool
}

impl<'a, 'b, L: ComponentBorrowTuple<'b>, G: ComponentBorrowTuple<'b>> SystemAccessor<'a, 'b, L, G> {
    pub fn new<'r>(world: &'r Registry, par: bool) -> Self where 'r: 'a {
        Self {
            phantom: PhantomData,
            registry: world,
            current_entity: Entity::default(),
            par
        }
    }

    #[inline(always)]
    pub unsafe fn update_entity(&mut self, entity: Entity) {
        self.current_entity = entity;
    }

    pub fn has<T: ComponentTuple>(&self, entity: Entity) -> bool {
        if let Some(archetype) = self.registry.archetypes.get(entity.archetype_id() as usize) {
            types::subset(T::type_ids().as_ref(), archetype.type_ids.as_ref())
        } else {
            false
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.registry.find(entity).is_some()
    }

    pub fn send<T: Event>(&self, e: T) {
        if self.par {
            unsafe {
                self.registry.events.send_sync(e);
            }
        } else {
            unsafe {
                self.registry.events.send(e);
            }
        }
    }

    pub fn get<'c, T: ComponentRefTuple<'c> + ComponentBorrowTuple<'c>>(
        &self,
        entity: Entity,
    ) -> Option<T>
    where
        'a: 'c,
    {
        let mut type_ids = <T as ComponentRefTuple<'c>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if entity == self.current_entity
            && !types::disjoint(type_ids.as_ref(), L::ValueType::type_ids().as_ref())
        {
            panic!("invalid read");
        } else if !types::subset(type_ids.as_ref(), G::ValueType::type_ids().as_ref()) {
            panic!("invalid read");
        }
        if let Some((archetype, index)) = self.registry.find(entity) {
            if let Ok(ptr) = archetype.try_as_mut_ptr::<<T as ComponentRefTuple<'c>>::ValueType>() {
                unsafe {
                    return Some(<T as ComponentRefTuple<'c>>::deref(
                        ptr.offset(index as isize),
                    ));
                }
            }
        }
        None
    }

    pub fn get_mut<'c, T: ComponentBorrowTuple<'c>>(&mut self, entity: Entity) -> Option<T>
    where
        'a: 'c,
    {
        let mut type_ids = <T as ComponentBorrowTuple<'c>>::ValueType::type_ids();
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
        if let Some((archetype, index)) = self.registry.find(entity) {
            if let Ok(ptr) =
                archetype.try_as_mut_ptr::<<T as ComponentBorrowTuple<'c>>::ValueType>()
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
    type Send: EventTuple;
    fn run(
        &self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'_, 'a, Self::Local, Self::Global>,
    );
}

pub trait SystemMut<'a> {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentBorrowTuple<'a>;
    type Send: EventTuple;
    fn run(
        &mut self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'_, 'a, Self::Local, Self::Global>,
    );
}

pub trait SystemPar<'a> {
    type Local: ComponentBorrowTuple<'a>;
    type Global: ComponentBorrowTuple<'a>;
    type Send: EventTuple;
    type Ctx;
    fn par_ctx(&self) -> Self::Ctx;
    fn run(
        &self,
        entity: Entity,
        components: Self::Local,
        accessor: &mut SystemAccessor<'_, 'a, Self::Local, Self::Global>,
        ctx: &mut Self::Ctx,
    );
}
