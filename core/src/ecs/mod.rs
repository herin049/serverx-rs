mod access;
pub mod component;
pub mod entity;
pub mod storage;
pub mod system;
pub mod tuple;
pub mod world;

pub type Index = u32;
pub type Generation = u64;
pub type ComponentId = u32;
pub type ArchetypeId = u32;

#[inline]
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
