use std::{
    cmp,
    fmt::{Debug, Formatter},
    mem,
};

use crate::collections::packed_vec::PackedVec;

#[derive(Copy, Clone, Debug)]
pub struct PalletOpts {
    pub repr_bits: u8,
    pub indirect_range: (u8, u8),
}

impl PalletOpts {
    pub fn new(repr_bits: u8, indirect_range: (u8, u8)) -> Self {
        Self {
            repr_bits,
            indirect_range,
        }
    }
}

const HASH_MAP_THRESHOLD: usize = 4;

pub enum PalletStorage {
    Single {
        value: u64,
    },
    ArrayMap {
        mapping: Vec<u64>,
        values: PackedVec,
    },
    HashMap {
        mapping: hashbrown::HashMap<u64, u64>,
        rev_mapping: Vec<u64>,
        values: PackedVec,
    },
    Direct {
        values: PackedVec,
    },
}

impl Default for PalletStorage {
    fn default() -> Self {
        Self::Single { value: 0 }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PalletMode {
    Single,
    Indirect,
    Direct,
}

pub struct PalletContainer {
    opts: PalletOpts,
    storage: PalletStorage,
    len: usize,
    bits: usize,
}

#[derive(Debug, Clone)]
pub enum TrySetErr {
    WouldResize,
}

impl PalletContainer {
    pub fn single(opts: PalletOpts, len: usize, value: u64) -> Self {
        Self {
            opts,
            storage: PalletStorage::Single { value },
            len,
            bits: 0,
        }
    }

    pub fn array(
        opts: PalletOpts,
        len: usize,
        bits: usize,
        mapping: Vec<u64>,
        values: PackedVec,
    ) -> Self {
        Self {
            opts,
            storage: PalletStorage::ArrayMap { mapping, values },
            len,
            bits,
        }
    }

    pub fn hash(
        opts: PalletOpts,
        len: usize,
        bits: usize,
        mapping: hashbrown::HashMap<u64, u64>,
        rev_mapping: Vec<u64>,
        values: PackedVec,
    ) -> Self {
        Self {
            opts,
            storage: PalletStorage::HashMap {
                mapping,
                rev_mapping,
                values,
            },
            len,
            bits,
        }
    }

    pub fn direct(opts: PalletOpts, len: usize, bits: usize, values: PackedVec) -> Self {
        Self {
            opts,
            storage: PalletStorage::Direct { values },
            len,
            bits,
        }
    }

    pub fn indirect(
        opts: PalletOpts,
        len: usize,
        bits: usize,
        mapping: Vec<u64>,
        values: PackedVec,
    ) -> Self {
        if bits >= HASH_MAP_THRESHOLD {
            let mut hash_mapping = hashbrown::HashMap::with_capacity(1 << bits);
            for (i, v) in mapping.iter().enumerate() {
                hash_mapping.insert(*v, i as u64);
            }
            Self::hash(opts, len, bits, hash_mapping, mapping, values)
        } else {
            Self::array(opts, len, bits, mapping, values)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn bits(&self) -> usize {
        self.bits
    }

    pub fn storage(&self) -> &PalletStorage {
        &self.storage
    }

    pub fn mode(&self) -> PalletMode {
        match &self.storage {
            PalletStorage::Single { .. } => PalletMode::Single,
            PalletStorage::Direct { .. } => PalletMode::Direct,
            _ => PalletMode::Indirect,
        }
    }

    pub fn pallet_mapping(&self) -> Option<&[u64]> {
        match &self.storage {
            PalletStorage::ArrayMap { mapping, .. } => Some(mapping.as_slice()),
            PalletStorage::HashMap { rev_mapping, .. } => Some(rev_mapping.as_slice()),
            _ => None,
        }
    }

    pub fn pallet_entries(&self) -> &[u64] {
        match &self.storage {
            PalletStorage::Single { .. } => &[],
            PalletStorage::ArrayMap { values, .. } => values.data().as_slice(),
            PalletStorage::HashMap { values, .. } => values.data().as_slice(),
            PalletStorage::Direct { values } => values.data().as_slice(),
        }
    }

    pub fn get(&self, index: usize) -> Option<u64> {
        if index < self.len {
            match &self.storage {
                PalletStorage::Single { value } => Some(*value),
                PalletStorage::ArrayMap { mapping, values } => {
                    Some(mapping[values.get(index).unwrap() as usize])
                }
                PalletStorage::HashMap {
                    rev_mapping,
                    values,
                    ..
                } => Some(rev_mapping[values.get(index).unwrap() as usize]),
                PalletStorage::Direct { values } => Some(values.get(index).unwrap()),
            }
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, new_value: u64) -> Option<u64> {
        if index < self.len {
            if let Ok(prev) = self.try_set(index, new_value) {
                Some(prev)
            } else {
                self.resize(self.bits + 1);
                Some(self.try_set(index, new_value).unwrap())
            }
        } else {
            None
        }
    }

    pub fn fill(&mut self, value: u64) {
        self.storage = PalletStorage::Single { value };
        self.bits = 0;
    }

    pub fn try_set(&mut self, index: usize, new_value: u64) -> Result<u64, TrySetErr> {
        match &mut self.storage {
            PalletStorage::Single { value } => {
                if new_value == *value {
                    Ok(*value)
                } else {
                    Err(TrySetErr::WouldResize)
                }
            }
            PalletStorage::ArrayMap { mapping, values } => {
                if let Some(p) = mapping.iter().position(|x| *x == new_value) {
                    let prev = values.set(index, p as u64).unwrap();
                    Ok(mapping[prev as usize])
                } else if mapping.len() < (1 << self.bits) {
                    let prev = values.set(index, mapping.len() as u64).unwrap();
                    mapping.push(new_value);
                    Ok(mapping[prev as usize])
                } else {
                    Err(TrySetErr::WouldResize)
                }
            }
            PalletStorage::HashMap {
                mapping,
                rev_mapping,
                values,
            } => {
                if let Some(p) = mapping.get(&new_value) {
                    let prev = values.set(index, *p).unwrap();
                    Ok(rev_mapping[prev as usize])
                } else if rev_mapping.len() < (1 << self.bits) {
                    let m = rev_mapping.len() as u64;
                    mapping.insert(new_value, m);
                    rev_mapping.push(new_value);
                    let prev = values.set(index, m).unwrap();
                    Ok(rev_mapping[prev as usize])
                } else {
                    Err(TrySetErr::WouldResize)
                }
            }
            PalletStorage::Direct { values } => Ok(values.set(index, new_value).unwrap()),
        }
    }

    pub fn resize(&mut self, bits: usize) {
        let bits = if bits > usize::from(self.opts.indirect_range.1) {
            usize::from(self.opts.repr_bits)
        } else {
            cmp::max(bits, usize::from(self.opts.indirect_range.0))
        };
        let mut prev_storage = mem::take(&mut self.storage);
        match prev_storage {
            PalletStorage::Single { value } => {
                if bits == usize::from(self.opts.repr_bits) {
                    let mut values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        values.set(i, value);
                    }
                    self.storage = PalletStorage::Direct { values };
                    self.bits = bits;
                } else if bits > HASH_MAP_THRESHOLD {
                    let mut mapping: hashbrown::HashMap<u64, u64> =
                        hashbrown::HashMap::with_capacity(1 << bits);
                    let mut rev_mapping: Vec<u64> = Vec::with_capacity(1 << bits);
                    mapping.insert(value, 0);
                    rev_mapping.push(value);
                    self.storage = PalletStorage::HashMap {
                        mapping,
                        rev_mapping,
                        values: PackedVec::zeros(bits, self.len),
                    };
                    self.bits = bits;
                } else {
                    let mut mapping: Vec<u64> = Vec::with_capacity(1 << bits);
                    mapping.push(value);
                    let mut values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        values.set(i, value);
                    }
                    self.storage = PalletStorage::ArrayMap { mapping, values };
                    self.bits = bits;
                }
            }
            PalletStorage::ArrayMap {
                mut mapping,
                values,
            } => {
                if bits == usize::from(self.opts.repr_bits) {
                    let mut new_values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        new_values.set(i, mapping[values.get(i).unwrap() as usize]);
                    }
                    self.storage = PalletStorage::Direct { values: new_values };
                    self.bits = bits;
                } else if bits > HASH_MAP_THRESHOLD {
                    let mut hash_mapping: hashbrown::HashMap<u64, u64> =
                        hashbrown::HashMap::with_capacity(1 << bits);
                    mapping.reserve((1 << bits) - (1 << self.bits));
                    for (i, v) in mapping.iter().enumerate() {
                        hash_mapping.insert(*v, i as u64);
                    }
                    let mut new_values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        new_values.set(i, values.get(i).unwrap());
                    }
                    self.storage = PalletStorage::HashMap {
                        mapping: hash_mapping,
                        rev_mapping: mapping,
                        values: new_values,
                    };
                    self.bits = bits;
                } else {
                    mapping.reserve((1 << bits) - (1 << self.bits));
                    let mut new_values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        new_values.set(i, values.get(i).unwrap());
                    }
                    self.storage = PalletStorage::ArrayMap {
                        mapping,
                        values: new_values,
                    };
                    self.bits = bits;
                }
            }
            PalletStorage::HashMap {
                mut mapping,
                mut rev_mapping,
                values,
            } => {
                if bits == usize::from(self.opts.repr_bits) {
                    let mut values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        values.set(i, rev_mapping[values.get(i).unwrap() as usize]);
                    }
                    self.storage = PalletStorage::Direct { values };
                    self.bits = bits;
                } else {
                    mapping.reserve((1 << bits) - (1 << self.bits));
                    rev_mapping.reserve((1 << bits) - (1 << self.bits));
                    let mut new_values = PackedVec::zeros(bits, self.len);
                    for i in 0..self.len {
                        new_values.set(i, values.get(i).unwrap());
                    }
                    self.storage = PalletStorage::HashMap {
                        mapping,
                        rev_mapping,
                        values: new_values,
                    };
                    self.bits = bits;
                }
            }
            PalletStorage::Direct { .. } => {}
        }
    }
}

impl Debug for PalletContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        for i in 0..self.len {
            debug_list.entry(&self.get(i).unwrap());
        }
        debug_list.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::pallet::{PalletContainer, PalletOpts};

    #[test]
    pub fn test_debug() {
        let mut container = PalletContainer::single(PalletOpts::new(10, (2, 4)), 4096, 0);
        println!("{} {:?}", container.bits(), container);
        container.set(5, 123);
        println!("{} {:?}", container.bits(), container);
        container.set(0, 42);
        container.set(3, 52);
        println!("{} {:?}", container.bits(), container);
        container.set(8, 44);
        println!("{} {:?}", container.bits(), container);
        container.set(6, 67);
        println!("{} {:?}", container.bits(), container);
        for i in 0..10 {
            container.set(10 + i, (i + 1) as u64);
        }
        println!("{} {:?}", container.bits(), container);
        container.set(9, 33);
        println!("{} {:?}", container.bits(), container);
    }
}
