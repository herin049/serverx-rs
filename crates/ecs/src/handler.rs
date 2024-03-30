use std::any::TypeId;
use std::marker::PhantomData;
use crate::entity::Entity;
use crate::event::Event;
use crate::registry::Registry;
use crate::sort::insertion_sort_noalias;
use crate::tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple, EventTuple};
use crate::tuple::ptr::PtrTuple;
use crate::tuple::type_tuple::TypeTuple;
use crate::types;


pub struct HandlerAccessor<'a, 'b, G: ComponentBorrowTuple<'b>, E: EventTuple> {
    phantom: PhantomData<&'b (G, E)>,
    registry: &'a Registry,
    par: bool
}

impl<'a, 'b, G: ComponentBorrowTuple<'b>, E: EventTuple> HandlerAccessor<'a, 'b, G, E> {
    pub fn new<'r>(registry: &'r Registry, par: bool) -> Self where 'r: 'a {
        Self {
            phantom: PhantomData,
            registry,
            par
        }
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
        if !E::type_ids().as_ref().iter().find(|x| TypeId::of::<T>().eq(*x)).is_some() {
            panic!("invalid send");
        }
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
    {
        let mut type_ids = <T as ComponentRefTuple<'c>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if !types::subset(type_ids.as_ref(), G::ValueType::type_ids().as_ref()) {
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
    {
        let mut type_ids = <T as ComponentBorrowTuple<'c>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if !types::subset(
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


pub trait Handler<'a> {
    type Global: ComponentBorrowTuple<'a>;
    type Send: EventTuple;
    type Event: Event;

    fn run(&self, event: &Self::Event, accessor: &mut HandlerAccessor<'_, 'a, Self::Global, Self::Send>);
}

pub trait HandlerMut<'a> {
    type Global: ComponentBorrowTuple<'a>;
    type Send: EventTuple;
    type Event: Event;

    fn run(&mut self, event: &Self::Event, accessor: &mut HandlerAccessor<'_, 'a, Self::Global, Self::Send>);
}

pub trait HandlerPar<'a> {
    type Global: ComponentBorrowTuple<'a>;
    type Send: EventTuple;
    type Event: Event;
    type Ctx;
    fn par_ctx(&'a self) -> Self::Ctx;
    fn run(&self, event: &Self::Event, accessor: &mut HandlerAccessor<'_, 'a, Self::Global, Self::Send>, ctx: &mut Self::Ctx);
}
