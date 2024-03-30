pub mod component;
pub mod entity;
pub mod event;
pub mod registry;
pub mod sort;
pub mod storage;
pub mod system;
pub mod tuple;
pub mod types;
mod handler;

pub type Generation = u64;
pub type ArchetypeId = u32;
pub type ArchetypeIndex = u32;
