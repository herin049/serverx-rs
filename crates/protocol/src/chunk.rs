use std::{
    io,
    io::{Read, Seek, Write},
    mem,
};

use serverx_block::states::BlockState;
use serverx_common::collections::{
    packed_vec::PackedVec,
    pallet::{PalletContainer, PalletMode, PalletOpts, PalletStorage},
};
use serverx_world::chunk::{
    section::{BiomePallet, BlockPallet},
    Chunk,
};

use crate::{
    decode::{
        AllocTracker, ProtoDecode, ProtoDecodeErr, ProtoDecodeErr::VarIntTooLong, ProtoDecodeSeq,
    },
    encode::{ProtoEncode, ProtoEncodeErr},
    types::{VarInt, MAX_VEC_LEN},
};

pub fn encode_chunk(chunk: &Chunk) -> Result<Vec<u8>, ProtoEncodeErr> {
    let mut bound: usize = 0;
    for section in chunk.sections() {
        bound += mem::size_of::<u16>();
        bound += pallet_size_bound(&section.blocks.pallet);
        bound += pallet_size_bound(&section.biomes.pallet);
    }
    let mut cursor: io::Cursor<Vec<u8>> = io::Cursor::new(Vec::with_capacity(bound));
    for section in chunk.sections() {
        u16::encode(&section.occupied(), &mut cursor)?;
        encode_pallet(&section.blocks.pallet, &mut cursor)?;
        encode_pallet(&section.biomes.pallet, &mut cursor)?;
    }
    Ok(cursor.into_inner())
}

fn pallet_size_bound(pallet: &PalletContainer) -> usize {
    let mut bound: usize = 0;
    if pallet.mode() == PalletMode::Single {
        bound += VarInt::MAX_BYTES;
    } else if let Some(mapping) = pallet.pallet_mapping() {
        bound += VarInt::MAX_BYTES;
        bound += VarInt::MAX_BYTES * mapping.len();
    }
    bound += VarInt::MAX_BYTES;
    bound += mem::size_of::<u64>() * pallet.pallet_entries().len();
    bound
}

pub fn encode_pallet<W: Write + Seek>(
    pallet: &PalletContainer,
    writer: &mut W,
) -> Result<(), ProtoEncodeErr> {
    u8::encode(&(pallet.bits() as u8), writer)?;
    if let PalletStorage::Single { value } = pallet.storage() {
        VarInt::encode(&(*value as i32), writer)?;
    } else if let Some(mapping) = pallet.pallet_mapping() {
        let mapping_len = mapping.len();
        if mapping_len > MAX_VEC_LEN {
            return Err(ProtoEncodeErr::SeqTooLong(mapping_len, MAX_VEC_LEN));
        }
        VarInt::encode(&(mapping_len as i32), writer)?;
        for v in mapping {
            VarInt::encode(&(*v as i32), writer)?;
        }
    }

    let pallet_entries = pallet.pallet_entries();
    <&[u64] as ProtoEncode>::encode(&pallet_entries, writer)
}

pub fn decode_pallet<R: Read + Seek, A: AllocTracker>(
    pallet_opts: PalletOpts,
    pallet_size: usize,
    reader: &mut R,
    alloc_tracker: &mut A,
) -> Result<PalletContainer, ProtoDecodeErr> {
    let bits = u8::decode(reader, alloc_tracker)? as usize;
    if bits == 0 {
        let value = VarInt::decode(reader, alloc_tracker)? as u64;
        Ok(PalletContainer::single(pallet_opts, pallet_size, value))
    } else if bits >= (pallet_opts.indirect_range.0 as usize)
        && bits <= (pallet_opts.indirect_range.1 as usize)
    {
        let len = <Vec<VarInt> as ProtoDecodeSeq>::decode_len(reader, alloc_tracker)?;
        alloc_tracker.alloc(len * mem::size_of::<u64>())?;
        let mut mapping: Vec<u64> = Vec::with_capacity(len);
        for _ in 0..len {
            mapping.push(VarInt::decode(reader, alloc_tracker)? as u64);
        }
        let values = <Vec<u64> as ProtoDecode>::decode(reader, alloc_tracker)?;
        let packed = PackedVec::try_from_raw_parts(values, bits, pallet_size)
            .map_err(|_| ProtoDecodeErr::ChunkDecodeErr)?;
        Ok(PalletContainer::indirect(
            pallet_opts,
            pallet_size,
            bits,
            mapping,
            packed,
        ))
    } else if bits == (pallet_opts.repr_bits as usize) {
        let values = <Vec<u64> as ProtoDecode>::decode(reader, alloc_tracker)?;
        let packed = PackedVec::try_from_raw_parts(values, bits, pallet_size)
            .map_err(|_| ProtoDecodeErr::ChunkDecodeErr)?;
        Ok(PalletContainer::direct(
            pallet_opts,
            pallet_size,
            pallet_opts.repr_bits as usize,
            packed,
        ))
    } else {
        Err(ProtoDecodeErr::ChunkDecodeErr)
    }
}

impl ProtoEncode for BlockPallet {
    type Repr = BlockPallet;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        encode_pallet(&data.pallet, writer)
    }
}

impl ProtoDecode for BlockPallet {
    type Repr = BlockPallet;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let pallet = decode_pallet(
            BlockPallet::PALLET_OPTS,
            BlockPallet::SIZE,
            reader,
            alloc_tracker,
        )?;
        Ok(BlockPallet { pallet })
    }
}

impl ProtoEncode for BiomePallet {
    type Repr = BiomePallet;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        encode_pallet(&data.pallet, writer)
    }
}

impl ProtoDecode for BiomePallet {
    type Repr = BiomePallet;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let pallet = decode_pallet(
            BiomePallet::PALLET_OPTS,
            BiomePallet::SIZE,
            reader,
            alloc_tracker,
        )?;
        Ok(BiomePallet { pallet })
    }
}
