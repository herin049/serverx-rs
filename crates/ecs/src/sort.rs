use std::ptr;

pub fn insertion_sort<T: PartialOrd>(x: &mut [T]) {
    let len = x.len();
    let x_ptr = x.as_mut_ptr();
    for i in 0..len {
        unsafe {
            let i_ptr = x_ptr.add(i);
            for j in (i + 1)..x.len() {
                let j_ptr = x_ptr.add(j);
                if T::gt(&*i_ptr, &*j_ptr) {
                    ptr::swap(i_ptr, j_ptr);
                }
            }
        }
    }
}

pub fn insertion_sort_noalias<T: PartialOrd + PartialEq>(x: &mut [T]) {
    let len = x.len();
    let x_ptr = x.as_mut_ptr();
    for i in 0..len {
        unsafe {
            let i_ptr = x_ptr.add(i);
            for j in (i + 1)..x.len() {
                let j_ptr = x_ptr.add(j);
                if T::gt(&*i_ptr, &*j_ptr) {
                    ptr::swap(i_ptr, j_ptr);
                } else if T::eq(&*i_ptr, &*j_ptr) {
                    panic!("aliasing constraint violated");
                }
            }
        }
    }
}

pub fn insertion_cosort<T: PartialOrd, U>(x: &mut [T], y: &mut [U]) {
    assert!(x.len() == y.len());
    let len = x.len();
    let x_ptr = x.as_mut_ptr();
    let y_ptr = y.as_mut_ptr();
    for i in 0..len {
        unsafe {
            let xi_ptr = x_ptr.add(i);
            let yi_ptr = y_ptr.add(i);
            for j in (i + 1)..x.len() {
                let xj_ptr = x_ptr.add(j);
                let yj_ptr = y_ptr.add(j);
                if T::gt(&*xi_ptr, &*xj_ptr) {
                    ptr::swap(xi_ptr, xj_ptr);
                    ptr::swap(yi_ptr, yj_ptr);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sort::{insertion_cosort, insertion_sort_noalias};

    #[test]
    fn test() {
        let mut x: [u32; 4] = [1, 3, 2, 4];
        let mut y: [u32; 4] = [1, 2, 3, 4];
        insertion_cosort(x.as_mut_slice(), y.as_mut_slice());
        println!("{:?} {:?}", x, y);
    }
    #[test]
    fn test2() {
        let mut x: [u32; 4] = [1, 3, 2, 3];
        insertion_sort_noalias(x.as_mut_slice());
    }
}
