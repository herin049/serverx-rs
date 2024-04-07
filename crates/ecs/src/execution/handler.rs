use std::{any::TypeId, collections::BTreeSet};

use crate::{
    execution::run::Runnable,
    message::Message,
    registry::{
        access::{Accessor, IterAccessor},
        Registry, UnsafeRegistryCell,
    },
    tuple::{
        borrow::BorrowTuple, component::ComponentBorrowTuple, message::SenderTuple,
        value::ValueTuple,
    },
    util,
    util::assert_no_alias,
};

pub trait Handler: Sized {
    type Target: Message;
    type Global<'g>: ComponentBorrowTuple<'g>;
    type Send<'s>: SenderTuple<'s>;

    fn runnable(&mut self) -> HandlerRunnable<'_, Self> {
        HandlerRunnable { handler: self }
    }

    fn handle(
        &mut self,
        event: &Self::Target,
        accessor: &mut impl Accessor,
        send: &mut Self::Send<'_>,
    );
}

pub struct HandlerRunnable<'a, T: Handler> {
    handler: &'a mut T,
}

impl<'a, T: Handler> Runnable for HandlerRunnable<'a, T> {
    fn extend_global_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global<'_> as BorrowTuple<'_>>::ReadType::type_ids().as_ref());
    }

    fn extend_global_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Global<'_> as BorrowTuple<'_>>::WriteType::type_ids().as_ref());
    }

    fn extend_message_write(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.extend(<T::Send<'_> as SenderTuple<'_>>::MessageType::type_ids().as_ref());
    }

    fn extend_message_read(&self, type_ids: &mut BTreeSet<TypeId>) {
        type_ids.insert(TypeId::of::<T::Target>());
    }

    fn prepare(&self, registry: &mut Registry) {
        T::Send::register(&mut registry.messages_mut());
    }

    fn run(&mut self, registry: &mut Registry) {
        if <T::Send<'_> as SenderTuple<'_>>::MessageType::type_ids()
            .as_ref()
            .iter()
            .find(|t| **t == TypeId::of::<T::Target>())
            .is_some()
        {
            panic!("target message exists in send group");
        }
        self.prepare(registry);
        let registry_cell = UnsafeRegistryCell(registry);
        let mut send = unsafe { T::Send::sender(&registry_cell.messages().unsafe_cell()) };
        let mut accessor =
            IterAccessor::<(), T::Global<'static>>::new(UnsafeRegistryCell(registry));
        for e in registry_cell.messages().messages::<T::Target>() {
            self.handler.handle(e, &mut accessor, &mut send);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        component::Component,
        entity::Entity,
        execution::{handler::Handler, iter::RegistryIter, run::Runnable},
        registry::{
            access::{Accessor, IterAccessor},
            Registry,
        },
        storage::channel::Sender,
    };

    #[derive(Debug)]
    pub struct Position(i64, i64, i64);
    #[derive(Debug)]
    pub struct Velocity(i64, i64, i64);

    impl Component for Position {}
    impl Component for Velocity {}

    pub struct SimpleIter<'a>(&'a i32);

    impl<'b> RegistryIter for SimpleIter<'b> {
        type Global<'g> = ();
        type Local<'l> = (&'l mut Position, &'l Velocity, &'l Entity);
        type Send<'s> = (Sender<'s, String>,);

        fn iter(
            &mut self,
            (p, v, e): Self::Local<'_>,
            accessor: &mut impl Accessor,
            send: &mut Self::Send<'_>,
        ) {
            send.0.send(format!("{:?}", e));
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    pub struct SimpleHandler<'a>(&'a i32);

    impl<'a> Handler for SimpleHandler<'a> {
        type Global<'g> = ();
        type Send<'s> = ();
        type Target = String;

        fn handle(
            &mut self,
            event: &Self::Target,
            accessor: &mut IterAccessor<(), Self::Global<'static>>,
            send: &mut Self::Send<'_>,
        ) {
            println!("received {}", event);
        }
    }

    #[test]
    fn test() {
        let mut reg = Registry::new();
        for i in 0..20 {
            reg.push((Position(i, i, i), Velocity(1, -1, 0)));
        }
        println!("{:#?}", reg);
        let i = 123;
        let mut s = SimpleIter(&i);
        let mut h = SimpleHandler(&i);
        s.runnable().run(&mut reg);
        h.runnable().run(&mut reg);
    }
}
