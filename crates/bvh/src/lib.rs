#![feature(portable_simd)]


use std::{
    cmp,
    fmt::{Debug, Formatter},
    mem::MaybeUninit,
    ops::{Range, Sub},
    ptr, slice,
};
use std::ops::Add;
use std::simd::f32x4;
use std::simd::prelude::SimdFloat;

use rayon::{
    iter::ParallelIterator,
    prelude::{IntoParallelIterator, ParallelSliceMut},
};

struct SendPtr<T>(*mut T);

impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for SendPtr<T> {}

unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub fn foo(t: f32x4) {
    println!("{:?}", t);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    value: f32x4
}

impl Vec3 {
    pub const INFINITY: Self = Self {
        value: f32x4::from_array([f32::INFINITY, f32::INFINITY, f32::INFINITY, 0.0]),
    };
    pub const NEG_INFINITY: Self = Self {
        value: f32x4::from_array([f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 0.0]),
    };

    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            value: f32x4::from_array([x, y, z, 0.0])
        }
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.value[0]
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.value[1]
    }

    #[inline(always)]
    pub fn z(&self) -> f32 {
        self.value[2]
    }

    #[inline(always)]
    pub fn min(&self, other: Vec3) -> Self {
        Self {
            value: self.value.simd_min(other.value)
        }
    }

    #[inline(always)]
    pub fn max(&self, other: Vec3) -> Self {
        Self {
            value: self.value.simd_min(other.value)
        }
    }

    #[inline(always)]
    pub fn add(&self, other: Vec3) -> Self {
        Self {
            value: self.value.add(other.value)
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: Vec3) -> Self {
        Self {
            value: self.value.sub(other.value)
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Aabb {
    pub from: Vec3,
    pub to: Vec3,
}

impl Aabb {
    pub fn from_iter<I: Iterator<Item = Aabb>>(iter: I) -> Self {
        let mut from  = Vec3::INFINITY;
        let mut to = Vec3::NEG_INFINITY;
        for bb in iter {
            from = from.min(bb.from);
            to = to.max(bb.to);
        }
        Self { from, to }
    }

    #[inline(always)]
    pub fn longest_axis(&self) -> Axis {
        let dims = self.to.sub(self.from);
        if dims.x() >= dims.y() && dims.x() >= dims.z() {
            Axis::X
        } else if dims.y() >= dims.x() && dims.y() >= dims.z() {
            Axis::Y
        } else {
            Axis::Z
        }
    }
}

pub struct Bvh<T: Debug + Send> {
    volumes: Vec<Aabb>,
    primitives: Vec<(Aabb, T)>,
    leaf_size: usize,
}

impl<T: Send + Debug> Bvh<T> {
    pub fn new(leaf_size: usize) -> Self {
        assert_ne!(leaf_size, 0);
        Self {
            volumes: Vec::new(),
            primitives: Vec::new(),
            leaf_size: leaf_size.next_power_of_two(),
        }
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    pub fn insert(&mut self, value: T, bounds: Aabb) {
        self.primitives.push((bounds, value));
    }

    pub fn build_par(&mut self) {
        if self.primitives.is_empty() {
            return;
        }
        unsafe {
            let max_height = usize::try_from(
                self.primitives.len().next_power_of_two().ilog2() - self.leaf_size.ilog2(),
            )
            .unwrap();
            let node_count = (1usize << (max_height + 1)) - 1;
            self.volumes.resize(node_count, Aabb::default());
            let range = (0usize, self.primitives.len());
            Self::build_in_par(
                SendPtr(self.volumes.as_mut_ptr()),
                0,
                SendPtr(self.primitives.as_mut_ptr()),
                range,
                self.leaf_size,
            );
        }
    }

    unsafe fn build_in_par(
        volumes: SendPtr<Aabb>,
        index: usize,
        elements: SendPtr<(Aabb, T)>,
        (start, end): (usize, usize),
        leaf_size: usize,
    ) {
        let middle = (start + end) / 2;
        let elements_slice =
            unsafe { slice::from_raw_parts_mut(elements.0.add(start), end - start) };
        let bounds = Aabb::from_iter(elements_slice.iter().map(|e| e.0));
        ptr::write(volumes.0.add(index), bounds);
        if elements_slice.len() >= 2 * leaf_size {
            match bounds.longest_axis() {
                Axis::X => elements_slice
                    .select_nth_unstable_by(middle - start, |x, y| x.0.from.x().total_cmp(&y.0.from.x())),
                Axis::Y => elements_slice
                    .select_nth_unstable_by(middle - start, |x, y| x.0.from.y().total_cmp(&y.0.from.y())),
                Axis::Z => elements_slice
                    .select_nth_unstable_by(middle - start, |x, y| x.0.from.z().total_cmp(&y.0.from.z())),
            };
            let left = (start, middle);
            let right = (middle, end);
            unsafe {
                rayon::join(
                    move || Self::build_in_par(volumes, index * 2 + 1, elements, left, leaf_size),
                    move || Self::build_in_par(volumes, index * 2 + 2, elements, right, leaf_size),
                );
            }
        }
    }

    pub fn build(&mut self) {
        if self.primitives.is_empty() {
            return;
        }
        unsafe {
            let max_height = usize::try_from(
                self.primitives.len().next_power_of_two().ilog2() - self.leaf_size.ilog2(),
            )
            .unwrap();
            let node_count = (1usize << (max_height + 1)) - 1;
            self.volumes.resize(node_count, Aabb::default());
            let range = (0usize, self.primitives.len());
            Self::build_in(
                self.volumes.as_mut_ptr(),
                0,
                self.primitives.as_mut_ptr(),
                range,
                self.leaf_size,
            );
        }
    }

    unsafe fn build_in(
        volumes: *mut Aabb,
        index: usize,
        elements: *mut (Aabb, T),
        (start, end): (usize, usize),
        leaf_size: usize,
    ) {
        let middle = (start + end) / 2;
        let elements_slice = unsafe { slice::from_raw_parts_mut(elements.add(start), end - start) };
        let bounds = Aabb::from_iter(elements_slice.iter().map(|e| e.0));
        ptr::write(volumes.add(index), bounds);
        if elements_slice.len() >= 2 * leaf_size {
            match bounds.longest_axis() {
                Axis::X => elements_slice
                    .select_nth_unstable_by(middle - start, |x, y| x.0.from.x().total_cmp(&y.0.from.x())),
                Axis::Y => elements_slice
                    .select_nth_unstable_by(middle - start, |x, y| x.0.from.y().total_cmp(&y.0.from.y())),
                Axis::Z => elements_slice
                    .select_nth_unstable_by(middle - start, |x, y| x.0.from.z().total_cmp(&y.0.from.z())),
            };
            let left = (start, middle);
            let right = (middle, end);
            unsafe {
                Self::build_in(volumes, index * 2 + 1, elements, left, leaf_size);
                Self::build_in(volumes, index * 2 + 2, elements, right, leaf_size);
            }
        }
    }
}

impl<T: Debug + Send> Debug for Bvh<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        DebugBvhNode::<'_, T>::fmt(
            &DebugBvhNode {
                bvh: self,
                index: 0,
                range: (0, self.primitives.len()),
            },
            f,
        )
    }
}

pub struct DebugBvhNode<'a, T: Debug + Send> {
    bvh: &'a Bvh<T>,
    index: usize,
    range: (usize, usize),
}

impl<'a, T: Debug + Send> Debug for DebugBvhNode<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("DebugBvhNode");
        let range_len = self.range.1 - self.range.0;
        debug_struct.field("bounds", &self.bvh.volumes[self.index]);
        if range_len >= 2 * self.bvh.leaf_size {
            debug_struct.field("left", &DebugBvhNode {
                bvh: self.bvh,
                index: self.index * 2 + 1,
                range: (self.range.0, range_len / 2),
            });
            debug_struct.field("right", &DebugBvhNode {
                bvh: self.bvh,
                index: self.index * 2 + 2,
                range: (self.range.0, range_len / 2),
            });
        } else {
            debug_struct.field("entries", &&self.bvh.primitives[self.range.0..self.range.1]);
        }
        debug_struct.finish()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;


    use super::*;

    #[test]
    fn test_longest_axis() {
        let bb = Aabb {
            from: Vec3::new(0.1, -0.0003, 0.0),
            to: Vec3::new(3.1415923423423423, 1.0, 1.0),
        };
        println!("{:?}", bb.longest_axis());

        unsafe {
            let v: Vec<MaybeUninit<i32>> = Vec::new();
        }
    }

    #[test]
    fn test_bvh() {
        let mut bvh = Bvh::new(2);
        for i in 0..5 {
            for j in 0..5 {
                bvh.insert("primitive", Aabb {
                    from: Vec3::new(i as f32, j as f32, 0.0),
                    to: Vec3::new(i as f32 + 0.5, j as f32 + 0.5, 0.0)
                });
            }
        }
        bvh.build();
        println!("{:#?}", bvh);
    }
}
