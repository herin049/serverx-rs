use std::{cmp, mem};

use crate::collections::packed_vec::PackedVec;

pub struct PalletOpts {
    pub repr_bits: u8,
    pub indirect_range: (u8, u8),
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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn bits(&self) -> usize {
        self.bits
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
                }
            }
            PalletStorage::ArrayMap { .. } => {}
            PalletStorage::HashMap { .. } => {}
            PalletStorage::Direct { .. } => {}
        }
    }
}
