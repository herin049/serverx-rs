use std::fmt::{Debug, Formatter};

use slab::Slab;

use crate::ecs::{
    component::ComponentSet,
    entity::Entity,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    ArchetypeId, Generation, Index,
};
