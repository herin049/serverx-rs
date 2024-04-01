use std::{any::TypeId, collections::BTreeSet};

use crate::registry::Registry;

pub trait Runnable<'r> {
    fn extend_local_read(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn extend_local_write(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn extend_global_read(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn extend_global_write(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn prepare(&self, _registry: &mut Registry) {}
    fn finalize(&self, _registry: &mut Registry) {}
    fn run(&mut self, registry: &'r mut Registry);
}

pub trait RunnablePar<'r>: Sync {
    fn extend_local_read(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn extend_local_write(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn extend_global_read(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn extend_global_write(&self, _type_ids: &mut BTreeSet<TypeId>) {}
    fn prepare(&self, _registry: &mut Registry) {}
    fn finalize(&self, _registry: &mut Registry) {}
    fn run(&mut self, registry: &'r mut Registry);
}
