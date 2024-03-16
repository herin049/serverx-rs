use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;

use crate::ecs::ComponentId;

pub unsafe trait Component: Debug + Sized + Send + Sync + 'static {
    const ID: ComponentId;
}

pub const MAX_COMPONENTS: usize = 128;
pub const COMPONENT_SET_LEN: usize = (MAX_COMPONENTS + 63) / 64;

#[derive(Clone, PartialEq, Eq, Hash)]
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
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                *self.0.get_unchecked_mut(i) = 0;
            }
        }
    }

    pub fn count_ones(&self) -> usize {
        let mut count = 0usize;
        for i in 0..COMPONENT_SET_LEN {
            unsafe {
                count += self.0.get_unchecked(i).count_ones() as usize;
            }
        }
        count
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

    pub fn iter(&self) -> ComponentSetIter {
        ComponentSetIter {
            component_set: self,
            curr: 0,
        }
    }
}

pub struct ComponentSetIter<'a> {
    component_set: &'a ComponentSet,
    curr: ComponentId,
}

impl<'a> Iterator for ComponentSetIter<'a> {
    type Item = ComponentId;

    fn next(&mut self) -> Option<Self::Item> {
        while (self.curr as usize) < MAX_COMPONENTS {
            let curr = self.curr;
            self.curr += 1;
            if self.component_set.contains(curr) {
                return Some(curr);
            }
        }
        None
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

        let mut set3 = ComponentSet::ones();

        println!("{}", set3);
    }
}
