use std::fmt::Debug;

use crate::component::Component;

pub trait Event: 'static + Sized + Debug + Send {}

impl<T: 'static + Sized + Debug + Send> Event for T {}
