use core::fmt::Debug;

use crate::{storage::column::Column, tuple::value::ValueTuple};

pub trait TableLayout: ValueTuple {
    type ColumnArray: AsMut<[Column]>
        + AsRef<[Column]>
        + Into<Box<[Column]>>
        + IntoIterator<Item = Column>;
    fn columns() -> Self::ColumnArray;
}
