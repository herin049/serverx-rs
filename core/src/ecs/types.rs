use std::any::TypeId;

pub fn subset(x: &[TypeId], y: &[TypeId]) -> bool {
    let n = x.len();
    let m = y.len();
    for i in 0..n {
        let tyi = unsafe { *x.get_unchecked(i) };
        let mut found: bool = false;
        for j in 0..m {
            unsafe {
                if tyi == *y.get_unchecked(j) {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return false;
        }
    }
    true
}

pub fn subset_sorted(x: &[TypeId], y: &[TypeId]) -> bool {
    let n = x.len();
    let m = y.len();
    let mut j = 0;
    for i in 0..n {
        let tyi = unsafe { *x.get_unchecked(i) };
        while j < m {
            unsafe {
                if tyi == *y.get_unchecked(j) {
                    j += 1;
                    break;
                }
            }
            j += 1;
        }
    }
    true
}
pub fn disjoint(x: &[TypeId], y: &[TypeId]) -> bool {
    let n = x.len();
    let m = y.len();
    for i in 0..n {
        let tyi = unsafe { *x.get_unchecked(i) };
        for j in 0..m {
            unsafe {
                if tyi == *y.get_unchecked(j) {
                    return false;
                }
            }
        }
    }
    true
}
