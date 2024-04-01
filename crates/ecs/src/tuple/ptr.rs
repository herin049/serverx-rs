pub trait PtrTuple: Copy + Clone {
    type PtrArray: AsMut<[*mut u8]> + AsRef<[*mut u8]>;
    fn null_ptr_slice() -> Self::PtrArray;
    fn null_ptr() -> Self;
    unsafe fn from_ptr_slice(ptrs: &[*mut u8]) -> Self;
    unsafe fn offset(self, count: isize) -> Self;
    unsafe fn add(self, count: usize) -> Self;
}

impl PtrTuple for () {
    type PtrArray = [*mut u8; 0];

    fn null_ptr_slice() -> Self::PtrArray {
        []
    }

    fn null_ptr() -> Self {
        ()
    }

    unsafe fn from_ptr_slice(ptrs: &[*mut u8]) -> Self {
        ()
    }

    unsafe fn offset(self, count: isize) -> Self {
        ()
    }

    unsafe fn add(self, count: usize) -> Self {
        ()
    }
}
