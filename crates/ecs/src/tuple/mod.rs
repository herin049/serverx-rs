use std::{any::TypeId, fmt::Debug};

use serverx_macros::ecs_tuple_impl;

use crate::{
    component::Component,
    storage::column::Column,
    tuple::{borrow::*, component::*, ptr::*, table::*, value::*},
};
pub mod borrow;
pub mod component;
pub mod ptr;
pub mod table;
pub mod value;

ecs_tuple_impl!(8);
