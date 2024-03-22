use std::any::TypeId;

mod component;
pub mod entity;
pub mod sort;
pub mod storage;
pub mod system;
pub mod tuple;
pub mod types;
pub mod world;

pub type Generation = u64;
pub type ArchetypeId = u32;
pub type ArchetypeIndex = u32;
