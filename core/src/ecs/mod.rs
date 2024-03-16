mod access;
pub mod component;
mod entity;
pub mod storage;
mod system;
mod tuple;
pub mod world;

pub type Index = u32;
pub type Generation = u64;
pub type ComponentId = u32;
pub type ArchetypeId = u32;
