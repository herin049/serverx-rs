pub trait PtrTuple: Copy + Clone {
    type PtrArray: AsMut<[*mut u8]> + AsRef<[*mut u8]>;
    fn null_ptr_slice() -> Self::PtrArray;
    fn from_ptr_slice(ptrs: &[*mut u8]) -> Self;
    unsafe fn offset(self, count: isize) -> Self;
}

impl PtrTuple for () {
    type PtrArray = [*mut u8; 0];

    fn null_ptr_slice() -> Self::PtrArray {
        []
    }

    fn from_ptr_slice(ptrs: &[*mut u8]) -> Self {
        ()
    }

    unsafe fn offset(self, count: isize) -> Self {
        ()
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::tuple::ptr::PtrTuple;

    #[test]
    fn test() {
        let mut a: i32 = 1;
        let mut b: f32 = 2.0;
        let mut c: u8 = 3;
        unsafe {
            println!(
                "{:?}",
                <(*mut i32, *mut f32, *mut u8)>::from_ptr_slice(&[
                    &mut a as *mut i32 as *mut u8,
                    &mut b as *mut f32 as *mut u8,
                    &mut c as *mut u8
                ])
                .offset(0)
            );
        }
    }
}
