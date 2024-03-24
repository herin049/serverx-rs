use std::fmt::Debug;

pub trait Component: 'static + Sized + Debug {}

impl<T: 'static + Sized + Debug> Component for T {}
