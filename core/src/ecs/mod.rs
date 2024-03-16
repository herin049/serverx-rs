mod access;
pub mod archetype;
pub mod component;
mod entity;
pub mod storage;
mod system;
mod tuple;
pub mod world;

pub type Index = u32;
pub type Generation = u32;
pub type ComponentId = u16;
pub type ArchetypeId = u16;
