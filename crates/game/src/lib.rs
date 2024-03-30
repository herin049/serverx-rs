use serverx_ecs::registry::Registry;

use crate::chunk::store::ChunkStore;

pub mod chunk;

pub struct Game {
    ecs: serverx_ecs::registry::Registry,
    chunk_store: ChunkStore,
}

impl Game {
    pub fn new() -> Self {
        Self {
            ecs: Registry::new(),
            chunk_store: ChunkStore::new(),
        }
    }

    pub fn tick(&mut self) {}
}

#[cfg(test)]
mod tests {}
