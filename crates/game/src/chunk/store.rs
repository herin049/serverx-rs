use std::{
    fmt::{Debug, Formatter},
    mem,
    mem::MaybeUninit,
};

use serverx_world::{chunk::Chunk, position::ChunkPosition};

pub struct ChunkStore {
    pub chunks: hashbrown::HashMap<
        (i32, i32),
        ChunkStoreGroup<{ Self::GROUP_WIDTH }, { Self::GROUP_SIZE }>,
    >,
}

impl ChunkStore {
    const GROUP_SIZE: usize = Self::GROUP_WIDTH * Self::GROUP_WIDTH;
    const GROUP_WIDTH: usize = 4;

    pub fn new() -> Self {
        Self {
            chunks: hashbrown::HashMap::new(),
        }
    }

    pub fn get(&self, position: ChunkPosition) -> Option<&ChunkStoreEntry> {
        let (gx, gxr) = num_integer::div_mod_floor(position.x, Self::GROUP_WIDTH as i32);
        let (gz, gzr) = num_integer::div_mod_floor(position.z, Self::GROUP_WIDTH as i32);
        if let Some(group) = self.chunks.get(&(gx, gz)) {
            group.get((gxr as usize, gzr as usize))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, position: ChunkPosition) -> Option<&mut ChunkStoreEntry> {
        let (gx, gxr) = num_integer::div_mod_floor(position.x, Self::GROUP_WIDTH as i32);
        let (gz, gzr) = num_integer::div_mod_floor(position.z, Self::GROUP_WIDTH as i32);
        if let Some(group) = self.chunks.get_mut(&(gx, gz)) {
            group.get_mut((gxr as usize, gzr as usize))
        } else {
            None
        }
    }

    pub fn insert(
        &mut self,
        position: ChunkPosition,
        chunk: ChunkStoreEntry,
    ) -> Option<ChunkStoreEntry> {
        let (gx, gxr) = num_integer::div_mod_floor(position.x, Self::GROUP_WIDTH as i32);
        let (gz, gzr) = num_integer::div_mod_floor(position.z, Self::GROUP_WIDTH as i32);
        if let Some(group) = self.chunks.get_mut(&(gx, gz)) {
            group.insert((gxr as usize, gzr as usize), chunk)
        } else {
            let mut group = ChunkStoreGroup::<{ Self::GROUP_WIDTH }, { Self::GROUP_SIZE }>::new();
            group.insert((gxr as usize, gzr as usize), chunk);
            self.chunks.insert((gx, gz), ChunkStoreGroup::new());
            None
        }
    }

    pub fn remove(&mut self, position: ChunkPosition) -> Option<ChunkStoreEntry> {
        let (gx, gxr) = num_integer::div_mod_floor(position.x, Self::GROUP_WIDTH as i32);
        let (gz, gzr) = num_integer::div_mod_floor(position.z, Self::GROUP_WIDTH as i32);
        let mut empty = false;
        let mut removed = None;
        if let Some(group) = self.chunks.get_mut(&(gx, gz)) {
            removed = group.remove((gxr as usize, gzr as usize));
            empty = group.occupied_count == 0;
        }
        self.chunks.remove(&(gx, gz));
        removed
    }
}

pub struct ChunkStoreGroup<const W: usize, const G: usize> {
    entries: [Option<ChunkStoreEntry>; G],
    occupied_count: usize,
}

impl<const W: usize, const G: usize> ChunkStoreGroup<W, G> {
    pub fn new() -> Self {
        const INIT: Option<ChunkStoreEntry> = None;
        Self {
            entries: [INIT; G],
            occupied_count: 0,
        }
    }

    pub fn get(&self, offset: (usize, usize)) -> Option<&ChunkStoreEntry> {
        let index = offset.0 + W * offset.1;
        if let Some(entry) = self.entries.get(index) {
            entry.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, offset: (usize, usize)) -> Option<&mut ChunkStoreEntry> {
        let index = offset.0 + W * offset.1;
        if let Some(entry) = self.entries.get_mut(index) {
            entry.as_mut()
        } else {
            None
        }
    }

    pub fn insert(
        &mut self,
        offset: (usize, usize),
        chunk: ChunkStoreEntry,
    ) -> Option<ChunkStoreEntry> {
        let prev = self.remove(offset);
        let index = offset.0 + W * offset.1;
        if index < self.entries.len() {
            unsafe {
                *self.entries.get_unchecked_mut(index) = Some(chunk);
            }
            self.occupied_count += 1;
        }
        prev
    }

    pub fn remove(&mut self, offset: (usize, usize)) -> Option<ChunkStoreEntry> {
        let index = offset.0 + W * offset.1;
        if let Some(entry) = self.entries.get_mut(index) {
            let mut prev = None;
            mem::swap(entry, &mut prev);
            self.occupied_count -= 1;
            prev
        } else {
            None
        }
    }

    pub fn iter(&self) -> ChunkStoreGroupIter<W, G> {
        ChunkStoreGroupIter {
            group: self,
            curr: 0,
        }
    }

    pub fn iter_mut(&mut self) -> ChunkStoreGroupIterMut<W, G> {
        ChunkStoreGroupIterMut {
            group: self,
            curr: 0,
        }
    }
}

impl<const W: usize, const G: usize> Debug for ChunkStoreGroup<W, G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        for e in self.iter() {
            debug_list.entry(e);
        }
        debug_list.finish()
    }
}

#[derive(Debug)]
pub struct ChunkStoreGroupIter<'a, const W: usize, const G: usize> {
    group: &'a ChunkStoreGroup<W, G>,
    curr: usize,
}

impl<'a, const W: usize, const G: usize> Iterator for ChunkStoreGroupIter<'a, W, G> {
    type Item = &'a ChunkStoreEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr < self.group.entries.len() {
            let entry = unsafe { self.group.entries.get_unchecked(self.curr) };
            self.curr += 1;
            if let Some(inner) = entry {
                return Some(inner);
            }
        }
        None
    }
}

pub struct ChunkStoreGroupIterMut<'a, const W: usize, const G: usize> {
    group: &'a mut ChunkStoreGroup<W, G>,
    curr: usize,
}

impl<'a, const W: usize, const G: usize> Iterator for ChunkStoreGroupIterMut<'a, W, G> {
    type Item = &'a mut ChunkStoreEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr < self.group.entries.len() {
            let entry = unsafe {
                self.group.entries.get_unchecked_mut(self.curr) as *mut Option<ChunkStoreEntry>
            };
            self.curr += 1;
            unsafe {
                if let Some(inner) = &mut *entry {
                    return Some(inner);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ChunkStoreEntry {
    pub view_count: u32,
    pub last_ticked: u64,
    pub chunk: Chunk,
}

impl ChunkStoreEntry {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            view_count: 0,
            last_ticked: 0,
            chunk,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serverx_world::chunk::Chunk;

    use crate::chunk::store::{ChunkStoreEntry, ChunkStoreGroup};

    #[test]
    fn test_chunk_store_group_empty() {
        let mut group = ChunkStoreGroup::<4, 16>::new();
        assert_eq!(group.occupied_count, 0);
        for i in 0..group.entries.len() {
            assert!(group.get((i / 4, i % 4)).is_none());
            assert!(group.get_mut((i / 4, i % 4)).is_none());
        }
    }

    #[test]
    fn test_chunk_store_group() {
        let mut group = ChunkStoreGroup::<4, 16>::new();
        for i in 0..4 {
            for j in 0..4 {
                assert!(group
                    .insert((i, j), ChunkStoreEntry::new(Chunk::new(384)))
                    .is_none());
            }
        }
        for i in 0..4 {
            for j in 0..4 {
                assert!(group.get((i, j)).is_some());
            }
        }

        for i in 0..4 {
            for j in 0..4 {
                assert!(group.get_mut((i, j)).is_some());
            }
        }

        for i in 0..4 {
            assert!(group.remove((i, i)).is_some());
        }

        for i in 0..4 {
            assert!(group.remove((i, i)).is_none());
        }

        for i in 0..4 {
            for j in 0..4 {
                assert_eq!(group.get((i, j)).is_some(), i != j);
                assert_eq!(group.get_mut((i, j)).is_some(), i != j);
            }
        }

        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert!(group
                        .insert((i, j), ChunkStoreEntry::new(Chunk::new(384)))
                        .is_some());
                }
            }
        }
    }

    #[test]
    fn test_hashmap() {}
}
