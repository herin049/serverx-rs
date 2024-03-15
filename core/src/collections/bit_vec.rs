use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, Seek, Write},
};

use itertools::Itertools;

use crate::protocol::{
    decode::{AllocTracker, ProtoDecodeErr, ProtoDecodeSeq},
    encode::{ProtoEncode, ProtoEncodeErr, ProtoEncodeSeq},
    types::{VarInt, MAX_VEC_LEN},
};

#[derive(Clone)]
pub struct BitVec {
    data: Vec<u64>,
    len: usize,
}

impl BitVec {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > 0 {
            Self {
                data: Vec::with_capacity((capacity - 1) / 64 + 1),
                len: 0,
            }
        } else {
            Self::new()
        }
    }

    pub fn zeros(len: usize) -> Self {
        if len > 0 {
            Self {
                data: vec![0u64; (len - 1) / 64 + 1],
                len,
            }
        } else {
            Self::new()
        }
    }

    pub fn ones(len: usize) -> Self {
        if len > 0 {
            let data_len = (len - 1) / 64 + 1;
            let mut data = vec![!0u64; data_len];
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
        Self { data, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn data(&self) -> &Vec<u64> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u64> {
        &mut self.data
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

impl ProtoEncodeSeq for BitVec {
    type Repr = BitVec;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        <Vec<u64> as ProtoEncodeSeq>::encode_len(data.data(), writer)
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        for e in data.data() {
            writer
                .write(e.to_le_bytes().as_slice())
                .map_err(|err| ProtoEncodeErr::IoErr(err))?;
        }
        Ok(())
    }
}

impl ProtoDecodeSeq for BitVec {
    type Repr = BitVec;

    fn decode_len<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<usize, ProtoDecodeErr> {
        <Vec<u64> as ProtoDecodeSeq>::decode_len(reader, alloc_tracker)
    }

    fn decode_seq<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
        len: usize,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        alloc_tracker.alloc(len * std::mem::size_of::<u64>())?;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            let mut bytes = [0u8; std::mem::size_of::<u64>()];
            reader
                .read_exact(bytes.as_mut_slice())
                .map_err(|err| ProtoDecodeErr::IoErr(err))?;
            result.push(u64::from_le_bytes(bytes));
        }
        Ok(Self::from_raw_parts(result))
    }
}

impl Display for BitVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            (0..self.len).map(|i| self.get(i).unwrap()).format(", ")
        )
    }
}

impl Debug for BitVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::collections::bit_vec::BitVec;

    const RUNS: usize = 100;

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
