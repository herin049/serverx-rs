use std::{any::TypeId, fmt::Debug};

use serverx_macros::ecs_tuple_impl;

use crate::{
    component::Component,
    message::{Message, Messages, UnsafeMessagesCell},
    storage::{channel::Sender, column::Column},
    tuple::{borrow::*, component::*, message::*, ptr::*, table::*, value::*},
};
pub mod borrow;
pub mod component;
pub mod message;
pub mod ptr;
pub mod table;
pub mod value;

ecs_tuple_impl!(8);
