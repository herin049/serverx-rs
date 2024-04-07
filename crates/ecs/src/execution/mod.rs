mod handler;
pub mod iter;
pub mod pipeline;
pub mod run;

#[cfg(test)]
mod tests {
    use std::cell::UnsafeCell;

    struct Cell<T> {
        inner: UnsafeCell<T>,
    }

    impl<T> Cell<T> {
        pub const fn new(value: T) -> Self {
            Self {
                inner: UnsafeCell::new(value),
            }
        }

        pub fn mutate<F: FnOnce(&mut T) + 'static>(&self, f: F) {
            unsafe {
                f(&mut *self.inner.get());
            }
        }
    }

    fn foobar(t: &'static Cell<i32>) {
        println!("hello");
    }

    #[test]
    fn test() {
        // const c: Cell<i32> = Cell::new(1i32);
        // foobar(&c);
    }
}
