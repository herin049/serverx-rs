use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;

pub type ComponentId = usize;
pub unsafe trait Component: Debug + Sized + Send + Sync + 'static {
    const ID: usize;
}

pub const MAX_COMPONENTS: usize = 128;
pub const COMPONENT_SET_LEN: usize = (MAX_COMPONENTS + 63) / 64;

#[derive(Clone)]
pub struct ComponentSet(pub [u64; COMPONENT_SET_LEN]);

impl From<[u64; COMPONENT_SET_LEN]> for ComponentSet {
    fn from(value: [u64; COMPONENT_SET_LEN]) -> Self {
        Self(value)
    }
}

impl ComponentSet {
    pub fn zeros() -> Self {
        ComponentSet([0u64; COMPONENT_SET_LEN])
    }

    pub fn ones() -> Self {
        ComponentSet([!0u64; COMPONENT_SET_LEN])
    }

    pub fn clear(&mut self) {
        for x in self.0.iter_mut() {
            *x = 0;
        }
    }

    pub fn count_ones(&self) -> usize {
        let mut result = 0usize;
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                result += self.0.get_unchecked(i).count_ones() as usize;
            }
        }
        result
    }

    pub fn add(&mut self, component_id: ComponentId) {
        let data_index = component_id as usize / 64;
        let data_offset = component_id as usize % 64;
        self.0[data_index] = (self.0[data_index] | (1u64 << data_offset));
    }

    pub fn remove(&mut self, component_id: ComponentId) {
        let data_index = component_id as usize / 64;
        let data_offset = component_id as usize % 64;
        self.0[data_index] = self.0[data_index] & !(1u64 << data_offset);
    }

    pub fn contains(&self, component_id: ComponentId) -> bool {
        let data_index = component_id as usize / 64;
        let data_offset = component_id as usize % 64;
        (self.0[data_index] & (1 << data_offset)) != 0
    }

    pub fn disjoint(&self, other: &ComponentSet) -> bool {
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                if (*self.0.get_unchecked(i) & *other.0.get_unchecked(i)) != 0 {
                    return false;
                }
            }
        }
        true
    }

    pub fn subset(&self, other: &ComponentSet) -> bool {
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                if *self.0.get_unchecked(i) & *other.0.get_unchecked(i) != *self.0.get_unchecked(i)
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn subset2(&self, other1: &ComponentSet, other2: &ComponentSet) -> bool {
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                if *self.0.get_unchecked(i)
                    & (*other1.0.get_unchecked(i) | *other2.0.get_unchecked(i))
                    != *self.0.get_unchecked(i)
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn subset3(
        &self,
        other1: &ComponentSet,
        other2: &ComponentSet,
        other3: &ComponentSet,
    ) -> bool {
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                if *self.0.get_unchecked(i)
                    & (*other1.0.get_unchecked(i)
                        | *other2.0.get_unchecked(i)
                        | *other3.0.get_unchecked(i))
                    != *self.0.get_unchecked(i)
                {
                    return false;
                }
            }
        }
        true
    }
}

impl Display for ComponentSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            (0..(self.0.len() * 64))
                .filter(|i| self.contains(*i as ComponentId))
                .format(", ")
        )
    }
}

impl Debug for ComponentSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::component::{ComponentId, ComponentSet};

    #[test]
    pub fn test() {
        let mut set = ComponentSet::zeros();
        println!("{}", set);
        set.add(4);
        println!("{}", set);
        set.add(4);
        set.add(100);
        set.add(42);
        set.add(68);
        set.add(3);
        println!("{}", set);
        set.remove(42);
        set.remove(68);
        println!("{}", set);
    }

    #[test]
    pub fn test2() {
        let mut set1 = ComponentSet::zeros();
        set1.add(1);
        set1.add(2);
        set1.add(64);

        let mut set2 = ComponentSet::zeros();
        set2.add(64);
        set2.add(1);

        println!("{} {}", set2.subset(&set1), set1.subset(&set2));
    }
}
