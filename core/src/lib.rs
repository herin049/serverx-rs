use std::{
    any::Any,
    io::{Read, Seek, Write},
};

use serde_derive::Serialize;
use serverx_macros::{Packet, ProtoDecode, ProtoEncode};

use crate::protocol::{
    decode::{AllocTracker, ProtoDecode, ProtoDecodeErr, ProtoDecodeSeq},
    encode::{ProtoEncode, ProtoEncodeErr, ProtoEncodeSeq},
    packet::{ConnectionState, PacketDirection},
    types::{VarInt, VarLong},
};

pub mod collections;
pub mod ecs;
pub mod nbt;
pub mod protocol;
pub mod utils;

#[derive(Debug, ProtoEncode, ProtoDecode, Serialize)]
pub struct TestType {
    #[proto(repr = "VarInt")]
    foo: i32,
    #[proto(max_len = 1024)]
    bar: Vec<u16>,
    #[proto(exact_len = 4, repr = "Vec<VarInt>")]
    baz: Vec<i32>,
}

#[derive(Debug, ProtoEncode, ProtoDecode)]
pub struct FooBar(i32, #[proto(repr = "VarLong")] i64);

#[derive(Debug, ProtoEncode, ProtoDecode)]
#[proto(enum_repr = "i32")]
pub enum TestEnum {
    A,
    B {
        i: i32,
        #[proto(repr = "VarInt")]
        b: i32,
    },
    #[proto(tag = 0xF)]
    D {
        s: String,
    },
    F(i32, i32),
}

#[derive(Packet, Debug)]
#[packet(0x1, ClientBound, Status)]
pub struct TestPacket;

#[cfg(test)]
mod tests {
    use std::{
        env,
        fmt::UpperHex,
        fs::File,
        io::{Cursor, Seek, SeekFrom},
    };

    use serverx_macros::{identifier, nbt};

    use super::*;
    use crate::{
        nbt::{
            decode::{BasicResourceTracker, NbtDecode},
            encode::NbtEncode,
            tag::TagType,
        },
        protocol::{
            decode::{BasicAllocTracker, ProtoDecode},
            encode::ProtoEncode,
            packet::Packet,
            types::VarInt,
        },
    };

    #[test]
    fn identifiers() {
        let a = identifier!("foo:foobar");
        println!("{} {} {}", a, a.namespace(), a.path());
    }

    #[test]
    fn testregistry() {
        println!("{:?}", env::current_dir());
        let mut registry_file = File::open("../run/resources/registries.nbt").unwrap();
        let registry_nbt = nbt::io::read_tag(&mut registry_file).unwrap();
        println!("{}", registry_nbt);
    }

    #[test]
    fn testnbt() {
        let foo = nbt!({
            "a": 123,
            "b": "abc",
            "c": {
                "d": 1,
                "e": "asdf",
                "f": [1, 2, 3]
            }
        });
        let mut vec = vec![0u8; 1024];
        let mut resource_tracker = BasicResourceTracker::new(1 << 30, 32);
        let mut buf = Cursor::new(vec);
        <nbt::Tag as NbtEncode>::encode(&foo, &mut buf).unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        let u = <nbt::Tag as NbtDecode>::decode(TagType::Compound, &mut buf, &mut resource_tracker)
            .unwrap();
        println!("{}", foo);
        println!("{}", u);
    }

    #[test]
    fn test3() {
        let mut vec = vec![0u8; 1024];
        let mut alloc_tracker = BasicAllocTracker::new(1 << 30);
        let mut buf = Cursor::new(vec);
        let e = TestEnum::F(123, 456);
        TestEnum::encode(&e, &mut buf).unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        let u = TestEnum::decode(&mut buf, &mut alloc_tracker).unwrap();
        println!("{:?}", u);
    }

    #[test]
    fn test2() {
        let mut vec = vec![0u8; 1024];
        let mut alloc_tracker = BasicAllocTracker::new(1 << 30);
        let mut buf = Cursor::new(vec);
        let t = FooBar(42, 67);
        FooBar::encode(&t, &mut buf).unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        let u = FooBar::decode(&mut buf, &mut alloc_tracker).unwrap();
        println!("{:?}", u);
    }

    #[test]
    fn test() {
        let a = 3;
        let a = 2;

        let mut vec = vec![0u8; 1024];
        let mut alloc_tracker = BasicAllocTracker::new(1 << 30);
        let mut buf = Cursor::new(vec);
        let t = TestType {
            foo: 123,
            bar: vec![1, 2, 3],
            baz: vec![4, 5, 6, 7],
        };
        TestType::encode(&t, &mut buf).unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        let u = TestType::decode(&mut buf, &mut alloc_tracker).unwrap();
        println!("{:?}", u);
        // let t = TestType {};
        // TestType::serialize(&t, &mut buf).unwrap();
    }

    #[test]
    fn it_works() {
        let a: TestPacket = TestPacket {};
        println!("{}", a.id());
        let mut vec = vec![0u8; 1024];
        let mut alloc_tracker = BasicAllocTracker::new(1 << 30);
        let mut buf = Cursor::new(vec);
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        Vec::<VarInt>::encode(&v, &mut buf).unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        let u = Vec::<VarInt>::decode(&mut buf, &mut alloc_tracker).unwrap();
        println!("{:?}", u);
    }
}
