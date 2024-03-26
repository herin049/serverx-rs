use serverx_world::chunk::Chunk;

#[derive(Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

pub struct ChunkStore {}

pub struct ChunkStoreEntry {
    pub view_count: u32,
    pub chunk: Chunk,
}
