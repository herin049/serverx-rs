use crate::chunk::Chunk;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Into<(i32, i32, i32)> for BlockPosition {
    fn into(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

impl From<(i32, i32, i32)> for BlockPosition {
    fn from(value: (i32, i32, i32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

impl From<ChunkPosition> for BlockPosition {
    fn from(value: ChunkPosition) -> Self {
        Self {
            x: value.x * (Chunk::WIDTH as i32),
            y: 0,
            z: value.z * (Chunk::WIDTH as i32),
        }
    }
}

impl From<Position> for BlockPosition {
    fn from(value: Position) -> Self {
        Self {
            x: value.x.floor() as i32,
            y: value.y.floor() as i32,
            z: value.z.floor() as i32,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl From<BlockPosition> for ChunkPosition {
    fn from(value: BlockPosition) -> Self {
        Self {
            x: num_integer::div_floor(value.x, Chunk::WIDTH as i32),
            z: num_integer::div_floor(value.z, Chunk::WIDTH as i32),
        }
    }
}

impl From<Position> for ChunkPosition {
    fn from(value: Position) -> Self {
        Self {
            x: num_integer::div_floor(value.x as i32, Chunk::WIDTH as i32),
            z: num_integer::div_floor(value.z as i32, Chunk::WIDTH as i32),
        }
    }
}

impl Into<(i32, i32)> for ChunkPosition {
    fn into(self) -> (i32, i32) {
        (self.x, self.z)
    }
}

impl From<(i32, i32)> for ChunkPosition {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: value.0,
            z: value.1,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<ChunkPosition> for Position {
    fn from(value: ChunkPosition) -> Self {
        Self {
            x: (value.x * Chunk::WIDTH as i32) as f64,
            y: 0.0,
            z: (value.z * Chunk::WIDTH as i32) as f64,
        }
    }
}

impl From<BlockPosition> for Position {
    fn from(value: BlockPosition) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            z: value.z as f64,
        }
    }
}

impl Into<(f64, f64, f64)> for Position {
    fn into(self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

impl From<(f64, f64, f64)> for Position {
    fn from(value: (f64, f64, f64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}
