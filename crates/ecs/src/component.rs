use core::fmt::Debug;
use crate::entity::Entity;

pub trait Component: 'static + Sized + Debug {}

impl Component for Entity {}
