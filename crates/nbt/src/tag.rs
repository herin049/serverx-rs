use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, Seek, Write},
};

#[derive(Clone, PartialEq)]
pub struct NamedTag {
    pub name: String,
    pub payload: Tag,
}

impl Debug for NamedTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\": {}", self.name, self.payload)
    }
}

impl Display for NamedTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

#[derive(Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<NamedTag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::End => write!(f, "(End)"),
            Tag::Byte(value) => write!(f, "{} (Byte)", value),
            Tag::Short(value) => write!(f, "{} (Short)", value),
            Tag::Int(value) => write!(f, "{} (Int)", value),
            Tag::Long(value) => write!(f, "{} (Long)", value),
            Tag::Float(value) => write!(f, "{} (Float)", value),
            Tag::Double(value) => write!(f, "{} (Double)", value),
            Tag::ByteArray(value) => write!(f, "{:?} (Byte Array)", value),
            Tag::String(value) => write!(f, "\"{}\" (String)", value),
            Tag::List(value) => write!(f, "{:?} (List)", value),
            Tag::Compound(value) => write!(f, "{:#?} (Compound)", value),
            Tag::IntArray(value) => write!(f, "{:?} (Int Array)", value),
            Tag::LongArray(value) => write!(f, "{:?} (Long Array)", value),
        }
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

impl Display for TagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

impl TagType {
    pub fn of(tag: &Tag) -> Self {
        match tag {
            Tag::End => Self::End,
            Tag::Byte(_) => Self::Byte,
            Tag::Short(_) => Self::Short,
            Tag::Int(_) => Self::Int,
            Tag::Long(_) => Self::Long,
            Tag::Float(_) => Self::Float,
            Tag::Double(_) => Self::Double,
            Tag::ByteArray(_) => Self::ByteArray,
            Tag::String(_) => Self::String,
            Tag::List(_) => Self::List,
            Tag::Compound(_) => Self::Compound,
            Tag::IntArray(_) => Self::IntArray,
            Tag::LongArray(_) => Self::LongArray,
        }
    }
}

impl Into<i8> for TagType {
    fn into(self) -> i8 {
        match self {
            TagType::End => 0,
            TagType::Byte => 1,
            TagType::Short => 2,
            TagType::Int => 3,
            TagType::Long => 4,
            TagType::Float => 5,
            TagType::Double => 6,
            TagType::ByteArray => 7,
            TagType::String => 8,
            TagType::List => 9,
            TagType::Compound => 10,
            TagType::IntArray => 11,
            TagType::LongArray => 12,
        }
    }
}

pub struct TryFromTagTypeErr;

impl TryFrom<i8> for TagType {
    type Error = TryFromTagTypeErr;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TagType::End),
            1 => Ok(TagType::Byte),
            2 => Ok(TagType::Short),
            3 => Ok(TagType::Int),
            4 => Ok(TagType::Long),
            5 => Ok(TagType::Float),
            6 => Ok(TagType::Double),
            7 => Ok(TagType::ByteArray),
            8 => Ok(TagType::String),
            9 => Ok(TagType::List),
            10 => Ok(TagType::Compound),
            11 => Ok(TagType::IntArray),
            12 => Ok(TagType::LongArray),
            _ => Err(TryFromTagTypeErr),
        }
    }
}

macro_rules! from_impl {
    ($($from_ty:ty:$name:ident),*) => {
       $(impl From<$from_ty> for Tag {
            fn from(value: $from_ty) -> Self {
                Self::$name(value)
            }
       })*
    }
}

from_impl! {
    i8: Byte, i16: Short, i32: Int, i64: Long, f32: Float, f64: Double,
    Vec<u8>: ByteArray, String: String, Vec<Tag>: List, Vec<NamedTag>: Compound,
    Vec<i32>: IntArray, Vec<i64>: LongArray
}

impl From<()> for Tag {
    fn from(value: ()) -> Self {
        Self::End
    }
}

impl<'a> From<&'a str> for Tag {
    fn from(value: &'a str) -> Self {
        Self::String(value.to_string())
    }
}

impl<S: Into<String>> From<(S, Tag)> for NamedTag {
    fn from(value: (S, Tag)) -> Self {
        NamedTag {
            name: value.0.into(),
            payload: value.1,
        }
    }
}

pub const MAX_TAG_SIZE: usize = 1 << 22;
pub const MAX_RECURSION_DEPTH: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct TagRoot;

#[cfg(test)]
mod tests {
    use serverx_macros::nbt;

    mod serverx_nbt {
        pub use crate::*;
    }

    #[test]
    pub fn test_nbt_macro() {
        let v = vec![1, 2, 3];
        let a = nbt!({
           "a": 123i32,
            "b": "asdfasdf",
            "c": [1, 3, 4, 5],
            "d": {
                "e": "asdf",
                "f": v,
                "g": 23.0
            }
        });
        println!("{}", a);
    }
}
