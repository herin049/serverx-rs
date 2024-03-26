use std::fmt::{Debug, Formatter};

pub struct PackedVec {
    data: Vec<u64>,
    elm_size: usize,
    len: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct InvalidPackedDataError;

impl PackedVec {
    pub fn new(elm_size: usize) -> Self {
        Self {
            data: Vec::new(),
            elm_size,
            len: 0,
        }
    }

    #[inline(always)]
    pub fn with_capacity(elm_size: usize, capacity: usize) -> Self {
        debug_assert!(elm_size >= 1 && elm_size <= 64);
        let elm_per_int = 64 / elm_size;
        let data_cap = (capacity + elm_per_int - 1) / elm_per_int;
        Self {
            data: Vec::with_capacity(data_cap),
            elm_size,
            len: 0,
        }
    }

    #[inline(always)]
    pub fn zeros(elm_size: usize, len: usize) -> Self {
        debug_assert!(elm_size >= 1 && elm_size <= 64);
        let elm_per_int = 64 / elm_size;
        let data_len = (len + elm_per_int - 1) / elm_per_int;
        let data = vec![0u64; data_len];
        Self {
            data,
            elm_size,
            len,
        }
    }

    #[inline(always)]
    pub fn try_from_raw_parts(data: Vec<u64>, elm_size: usize, len: usize) -> Result<Self, InvalidPackedDataError> {
        let elm_per_int = (u64::BITS as usize) / elm_size;
        let data_cap = data.len() * elm_per_int;
        if data_cap < len || data_cap >= len + elm_per_int {
            Err(InvalidPackedDataError)
        } else {
            Ok(Self::from_raw_parts(data, elm_size, len))
        }
    }

    #[inline(always)]
    pub fn from_raw_parts(data: Vec<u64>, elm_size: usize, len: usize) -> Self {
        Self {
            data,
            elm_size,
            len,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn elm_size(&self) -> usize {
        self.elm_size
    }

    #[inline(always)]
    pub fn data(&self) -> &Vec<u64> {
        &self.data
    }

    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut Vec<u64> {
        &mut self.data
    }

    #[inline(always)]
    pub fn elm_per_int(&self) -> usize {
        64 / self.elm_size
    }

    pub fn push<T: Into<u64>>(&mut self, value: T) {
        let elm_per_int = 64 / self.elm_size;
        let data_index = self.len / elm_per_int;
        if data_index == self.data.len() {
            self.data.push(0);
        }
        self.len += 1;
        self.set(self.len - 1, value);
    }

    pub fn pop(&mut self) -> Option<u64> {
        if self.len > 0 {
            let value = self.get(self.len - 1);
            self.len -= 1;
            let elm_per_int = 64 / self.elm_size;
            if self.len % elm_per_int == 0 {
                self.data.pop();
            }
            value
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<u64> {
        if index < self.len {
            let mask = !0u64 >> (64 - self.elm_size);
            let elm_per_int = 64 / self.elm_size;
            let data_index = index / elm_per_int;
            let offset = (index % elm_per_int) * self.elm_size;
            let value = (self.data[data_index] >> offset) & mask;
            Some(value)
        } else {
            None
        }
    }

    pub fn set<T: Into<u64>>(&mut self, index: usize, value: T) -> Option<u64> {
        if index < self.len {
            let mask = !0u64 >> (64 - self.elm_size);
            let elm_per_int = 64 / self.elm_size;
            let data_index = index / elm_per_int;
            let offset = (index % elm_per_int) * self.elm_size;
            let old = (self.data[data_index] & (mask << offset)) >> offset;
            self.data[data_index] =
                (self.data[data_index] & !(mask << offset)) | ((value.into() & mask) << offset);
            Some(old)
        } else {
            None
        }
    }
}

impl Debug for PackedVec {
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

    use crate::collections::packed_vec::PackedVec;

    const RUNS: usize = 30;

    #[test]
    pub fn test_display() {
        let mut packedvec = PackedVec::zeros(12, 4096);
        for i in [5, 10, 100, 42, 32] {
            packedvec.set(i, i as u64);
        }
        println!("{:?}", packedvec);
    }

    #[test]
    pub fn test_zeros() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let elm_size: usize = rng.gen_range(1..64);
            let len: usize = rng.gen_range(1000..2000);
            let packedvec = PackedVec::zeros(elm_size, len);
            for i in 0..len {
                assert_eq!(packedvec.get(i), Some(0));
            }
        }
    }

    #[test]
    pub fn test_push() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let elm_size: usize = rng.gen_range(1..64);
            let len: usize = rng.gen_range(5000..10000);
            let mut packedvec = PackedVec::new(elm_size);
            let mut values: Vec<u64> = Vec::new();
            for _ in 0..len {
                let v: u64 = rng.gen_range(0..(1 << elm_size));
                packedvec.push(v);
                values.push(v);
            }
            assert_eq!(packedvec.len(), values.len());
            for i in 0..len {
                assert_eq!(packedvec.get(i).unwrap(), values[i]);
            }
        }
    }

    #[test]
    pub fn test_pop() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let elm_size: usize = rng.gen_range(1..64);
            let len: usize = rng.gen_range(5000..10000);
            let mut packedvec = PackedVec::new(elm_size);
            let mut values: Vec<u64> = Vec::new();
            for _ in 0..len {
                if rng.gen::<bool>() {
                    assert_eq!(packedvec.pop(), values.pop());
                } else {
                    let v: u64 = rng.gen_range(0..(1 << elm_size));
                    packedvec.push(v);
                    values.push(v);
                }
            }
            assert_eq!(packedvec.len(), values.len());
            for i in 0..values.len() {
                assert_eq!(packedvec.get(i).unwrap(), values[i]);
            }
        }
    }

    #[test]
    pub fn test_get_set() {
        let mut rng = StdRng::seed_from_u64(2023);
        for _ in 0..RUNS {
            let elm_size: usize = rng.gen_range(1..64);
            let len: usize = rng.gen_range(5000..10000);
            let mut packedvec = PackedVec::zeros(elm_size, len);
            let mut values: Vec<u64> = vec![0u64; len];
            let num: usize = rng.gen_range(5000..10000);
            for _ in 0..num {
                if rng.gen::<bool>() {
                    let i: usize = rng.gen_range(0..len);
                    let v: u64 = rng.gen_range(0..(1 << elm_size));
                    packedvec.set(i, v);
                    values[i] = v;
                } else {
                    let i: usize = rng.gen_range(0..(2 * len));
                    assert_eq!(packedvec.get(i), values.get(i).copied());
                }
            }
        }
    }
}
