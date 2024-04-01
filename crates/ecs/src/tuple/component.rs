use crate::tuple::{
    borrow::{BorrowTuple, RefTuple},
    table::TableLayout,
    value::ValueTuple,
};

pub trait ComponentTuple: ValueTuple + TableLayout {}
pub trait ComponentRefTuple<'a>: RefTuple<'a> {}
pub trait ComponentBorrowTuple<'a>: BorrowTuple<'a> {}

impl<'a, T: RefTuple<'a>> ComponentRefTuple<'a> for T where
    <T as RefTuple<'a>>::ValueType: ValueTuple
{
}

impl<'a, T: BorrowTuple<'a>> ComponentBorrowTuple<'a> for T where
    <T as BorrowTuple<'a>>::ValueType: ValueTuple
{
}
