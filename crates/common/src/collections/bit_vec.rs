use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, Seek, Write},
};

use itertools::Itertools;
use smallvec::{SmallVec, smallvec};

#[derive(Clone)]
pub struct BitVec {
    data: SmallVec<[u64; 4]>,
    len: usize,
}

impl BitVec {
    pub fn new() -> Self {
        Self {
            data: SmallVec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > 0 {
            Self {
                data: SmallVec::with_capacity((capacity - 1) / 64 + 1),
                len: 0,
            }
        } else {
            Self::new()
        }
    }

    pub fn zeros(len: usize) -> Self {
        if len > 0 {
            Self {
                data: smallvec![0u64; (len - 1) / 64 + 1],
                len,
            }
        } else {
            Self::new()
        }
    }

    pub fn ones(len: usize) -> Self {
        if len > 0 {
            let data_len = (len - 1) / 64 + 1;
            let mut data = smallvec![!0u64; data_len];
            let rem = len % 64;
            if rem != 0 {
                *data.last_mut().unwrap() = (1u64 << rem) - 1;
            }
            Self { data, len }
        } else {
            Self::new()
        }
    }

    pub fn from_raw_parts(data: Vec<u64>) -> Self {
        let len = data.len() * 8;
        Self { data: SmallVec::from(data), len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[u64] {
        self.data.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u64] {
        self.data.as_mut_slice()
    }

    pub fn push(&mut self, value: bool) {
        let rem = self.len % 64;
        if rem == 0 {
            self.data.push(0u64);
        }
        *self.data.last_mut().unwrap() |= (value as u64) << rem;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<bool> {
        if self.len > 0 {
            let mask = 1u64 << ((self.len - 1) % 64);
            let value = *self.data.last().unwrap() & mask != 0u64;
            *self.data.last_mut().unwrap() &= !mask;
            self.len -= 1;
            if self.len % 64 == 0 {
                self.data.pop();
            }
            Some(value)
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index < self.len {
            let data_index = index / 64;
            let rem = index % 64;
            Some((self.data[data_index] & (1u64 << rem)) != 0)
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, value: bool) -> Option<bool> {
        if index < self.len {
            let data_index = index / 64;
            let rem = index % 64;
            let mask = 1u64 << rem;
            let old = self.data[data_index] & mask != 0;
            self.data[data_index] = (self.data[data_index] & !mask) | (value as u64) << rem;
            Some(old)
        } else {
            None
        }
    }
}

impl Debug for BitVec {
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
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::collections::bit_vec::BitVec;

    const RUNS: usize = 100;

    #[test]
    pub fn test_debug() {
        let mut bitvec = BitVec::zeros(100);
        bitvec.set(42, true);
        bitvec.set(44, true);
        bitvec.set(1, true);
        bitvec.set(3, true);
        bitvec.set(4, true);
        println!("{:?}", bitvec);
    }

    #[test]
    pub fn test_zeros() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let len: usize = rng.gen_range(0..10000);
            let bitvec = BitVec::zeros(len);
            for i in 0..len {
                assert_eq!(bitvec.get(i), Some(false));
            }
        }
    }

    #[test]
    pub fn test_ones() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let len: usize = rng.gen_range(0..10000);
            let bitvec = BitVec::ones(len);
            for i in 0..len {
                assert_eq!(bitvec.get(i), Some(true));
            }
        }
    }

    #[test]
    pub fn test_push() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let mut bitvec = BitVec::new();
            let mut values: Vec<bool> = Vec::new();
            let len = rng.gen_range(0..10000);
            for _ in 0..len {
                let b: bool = rng.gen();
                bitvec.push(b);
                values.push(b);
            }
            assert_eq!(bitvec.len(), values.len());
            for i in 0..len {
                assert_eq!(bitvec.get(i).unwrap(), values[i]);
            }
        }
    }

    #[test]
    pub fn test_pop() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let mut bitvec = BitVec::new();
            let mut values: Vec<bool> = Vec::new();
            let len = rng.gen_range(0..10000);
            for _ in 0..len {
                if rng.gen::<bool>() {
                    assert_eq!(bitvec.pop(), values.pop())
                } else {
                    let b: bool = rng.gen();
                    bitvec.push(b);
                    values.push(b);
                }
            }
            assert_eq!(bitvec.len(), values.len());
            for _ in 0..values.len() {
                assert_eq!(bitvec.pop(), values.pop());
            }
        }
    }

    #[test]
    pub fn test_get_set() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let len: usize = rng.gen_range(0..10000);
            let mut bitvec = BitVec::zeros(len);
            let mut vec = vec![false; len];
            let num: usize = rng.gen_range(5000..10000);
            for _ in 0..num {
                if rng.gen::<bool>() {
                    let i: usize = rng.gen_range(0..len);
                    let b: bool = rng.gen();
                    bitvec.set(i, b);
                    vec[i] = b;
                } else {
                    let i: usize = rng.gen_range(0..(2 * len));
                    assert_eq!(bitvec.get(i), vec.get(i).copied());
                }
            }
        }
    }
}
