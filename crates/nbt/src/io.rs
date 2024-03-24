use std::io::{Read, Seek, Write};

use crate::{
    decode::{BasicResourceTracker, NbtDecode, NbtDecodeErr},
    encode::{NbtEncode, NbtEncodeErr},
    tag::TagType,
    NamedTag, Tag,
};

pub fn read_tag<R: Read + Seek>(reader: &mut R) -> Result<Tag, NbtDecodeErr> {
    let mut tracker = BasicResourceTracker::new(1 << 30, 32);
    let tag_type_id = <i8 as NbtDecode>::decode(TagType::Byte, reader, &mut tracker)?;
    let tag_type =
        TagType::try_from(tag_type_id).map_err(|err| NbtDecodeErr::InvalidTagType(tag_type_id))?;
    let _ = <String as NbtDecode>::decode(TagType::String, reader, &mut tracker)?;
    Tag::decode(tag_type, reader, &mut tracker)
}

pub fn write_tag<W: Write + Seek>(writer: &mut W, tag: &Tag) -> Result<(), NbtEncodeErr> {
    <i8 as NbtEncode>::encode(&TagType::of(tag).into(), writer)?;
    <String as NbtEncode>::encode(&"".to_string(), writer)?;
    Tag::encode(tag, writer)
}
