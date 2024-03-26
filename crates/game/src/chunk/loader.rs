use rayon::ThreadPool;

use crate::chunk::store::ChunkPosition;

pub enum ChunkLoadStatus {
    Loading,
    Unloading,
}

pub struct ChunkLoader {
    pool: ThreadPool,
    status: hashbrown::HashMap<ChunkPosition, ChunkLoadStatus>,
}

impl ChunkLoader {}
