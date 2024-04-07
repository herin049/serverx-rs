use core::fmt::Debug;
use std::any::{Any, TypeId};

use crate::{
    entity::Entity,
    storage::channel::{Channel, Sender},
};

pub trait Message: 'static + Sized + Debug + Send + Sync {
    const TARGETED: bool = false;

    fn target(&self) -> Entity {
        Entity::default()
    }
}

pub struct Messages {
    channels: Vec<(TypeId, Box<dyn Any>)>,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
        }
    }

    pub fn register<T: Message>(&mut self) {
        if let Err(i) = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0)
        {
            self.channels
                .insert(i, (TypeId::of::<T>(), Box::new(Channel::<T>::new())));
        }
    }

    pub fn get_or_register<T: Message>(&mut self) -> &Channel<T> {
        let search = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0);
        match search {
            Ok(i) => unsafe {
                &*(self.channels.get_unchecked_mut(i).1.as_mut() as *const dyn Any
                    as *const Channel<T>)
            },
            Err(i) => {
                self.channels
                    .insert(i, (TypeId::of::<T>(), Box::new(Channel::<T>::new())));
                unsafe {
                    &*(self.channels.get_unchecked_mut(i).1.as_mut() as *const dyn Any
                        as *const Channel<T>)
                }
            }
        }
    }

    pub fn get<T: Message>(&self) -> &Channel<T> {
        let channel_idx = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0)
            .unwrap();
        unsafe {
            &*(self.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                as *const Channel<T>)
        }
    }

    pub fn send<T: Message>(&mut self, message: T) {
        let channel = self.get_or_register();
        unsafe {
            channel.send(message);
        }
    }

    pub fn send_tl<T: Message>(&self, message: T) {
        let channel = self.get();
        unsafe {
            channel.send_tl(message);
        }
    }

    pub fn sender<T: Message>(&mut self) -> Sender<'_, T> {
        let channel_idx = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0)
            .unwrap();
        unsafe {
            (&*(self.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                as *const Channel<T>))
                .sender()
        }
    }

    pub fn sender_tl<T: Message>(&self) -> Sender<'_, T> {
        let channel_idx = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0)
            .unwrap();
        unsafe {
            (&*(self.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                as *const Channel<T>))
                .sender_tl()
        }
    }

    pub fn messages<T: Message>(&self) -> &[T] {
        let search = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0);
        if let Ok(channel_idx) = search {
            unsafe {
                (&*(self.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                    as *const Channel<T>))
                    .messages()
            }
        } else {
            &[]
        }
    }

    pub fn flush<T: Message>(&mut self) {
        let search = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0);
        if let Ok(channel_idx) = search {
            unsafe {
                (&*(self.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                    as *const Channel<T>))
                    .flush()
            }
        }
    }

    pub fn sync<T: Message>(&mut self) {
        let search = self
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0);
        if let Ok(channel_idx) = search {
            unsafe {
                (&mut *(self.channels.get_unchecked_mut(channel_idx).1.as_mut() as *mut dyn Any
                    as *mut Channel<T>))
                    .sync()
            }
        }
    }

    pub fn unsafe_cell(&self) -> UnsafeMessagesCell<'_> {
        UnsafeMessagesCell(self)
    }
}

pub struct UnsafeMessagesCell<'a>(&'a Messages);

impl<'a> UnsafeMessagesCell<'a> {
    pub fn new(messages: &'a Messages) -> Self {
        Self(messages)
    }

    pub fn get<'b, T: Message>(&self) -> &'b Channel<T>
    where
        'a: 'b,
    {
        self.0.get()
    }

    pub unsafe fn send<T: Message>(&self, message: T) {
        self.get().send(message)
    }

    pub unsafe fn send_tl<T: Message>(&self, message: T) {
        self.send_tl(message)
    }

    pub unsafe fn messages<'b, T: Message>(&self) -> &'b [T]
    where
        'a: 'b,
    {
        self.0.messages()
    }

    pub unsafe fn flush<T: Message>(&self) {
        let search = self
            .0
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0);
        if let Ok(channel_idx) = search {
            unsafe {
                (&*(self.0.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                    as *const Channel<T>))
                    .flush()
            }
        }
    }

    pub unsafe fn sender<'b, T: Message>(&self) -> Sender<'b, T>
    where
        'a: 'b,
    {
        let channel_idx = self
            .0
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0)
            .unwrap();
        unsafe {
            (&*(self.0.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                as *const Channel<T>))
                .sender()
        }
    }

    pub unsafe fn sender_tl<'b, T: Message>(&self) -> Sender<'b, T>
    where
        'a: 'b,
    {
        let channel_idx = self
            .0
            .channels
            .binary_search_by_key(&TypeId::of::<T>(), |x| x.0)
            .unwrap();
        unsafe {
            (&*(self.0.channels.get_unchecked(channel_idx).1.as_ref() as *const dyn Any
                as *const Channel<T>))
                .sender_tl()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::{Message, Messages};

    #[derive(Debug)]
    pub struct TestMessage(&'static str);

    impl Message for TestMessage {}

    #[test]
    fn test() {
        let mut messages = Messages::new();
        messages.register::<TestMessage>();
        println!("{:?}", messages.messages::<TestMessage>());
        messages.send(TestMessage("hello"));
        messages.send(TestMessage("world"));
        messages.send(TestMessage("!"));
        println!("{:?}", messages.messages::<TestMessage>());
    }

    #[test]
    fn test_sync() {
        let mut messages = Messages::new();
        messages.register::<TestMessage>();
        println!("{:?}", messages.messages::<TestMessage>());
        messages.send_tl(TestMessage("hello"));
        messages.send_tl(TestMessage("tl"));
        println!("{:?}", messages.messages::<TestMessage>());
        messages.send(TestMessage("!"));
        println!("{:?}", messages.messages::<TestMessage>());
        messages.sync::<TestMessage>();
        println!("{:?}", messages.messages::<TestMessage>());
    }
}
