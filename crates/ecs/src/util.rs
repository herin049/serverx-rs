use std::ptr;

pub fn assert_no_alias<T: PartialEq>(x: &[T], msg: &str) {
    let len = x.len();
    let x_ptr = x.as_ptr();
    for i in 0..len {
        unsafe {
            let i_ptr = x_ptr.add(i);
            for j in (i + 1)..x.len() {
                let j_ptr = x_ptr.add(j);
                if T::eq(&*i_ptr, &*j_ptr) {
                    panic!("{}", msg);
                }
            }
        }
    }
}

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

pub fn insertion_sort_no_alias<T: PartialOrd + PartialEq>(x: &mut [T]) {
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

pub fn subset<T: PartialEq>(x: &[T], y: &[T]) -> bool {
    let x_len = x.len();
    let y_len = y.len();
    let x_ptr = x.as_ptr();
    let y_ptr = y.as_ptr();
    'outer: for i in 0..x_len {
        unsafe {
            let i_ptr = x_ptr.add(i);
            for j in 0..y_len {
                let j_ptr = y_ptr.add(j);
                if T::eq(&*i_ptr, &*j_ptr) {
                    continue 'outer;
                }
            }
            return false;
        }
    }
    true
}

pub fn disjoint<T: PartialEq>(x: &[T], y: &[T]) -> bool {
    let x_len = x.len();
    let y_len = y.len();
    let x_ptr = x.as_ptr();
    let y_ptr = y.as_ptr();
    for i in 0..x_len {
        unsafe {
            let i_ptr = x_ptr.add(i);
            for j in 0..y_len {
                let j_ptr = y_ptr.add(j);
                if T::eq(&*i_ptr, &*j_ptr) {
                    return false;
                }
            }
        }
    }
    true
}
