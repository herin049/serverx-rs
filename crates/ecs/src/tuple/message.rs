use crate::{
    message::{Message, Messages, UnsafeMessagesCell},
    storage::channel::Sender,
    tuple::value::ValueTuple,
};

pub trait SenderTuple<'a> {
    type MessageType: ValueTuple;

    fn register(messages: &mut Messages);
    unsafe fn sync(messages: &mut Messages);
    unsafe fn sender<'b>(messages: &UnsafeMessagesCell<'b>) -> Self
    where
        'b: 'a;
    unsafe fn sender_tl<'b>(messages: &UnsafeMessagesCell<'b>) -> Self
    where
        'b: 'a;
    unsafe fn flush(messages: &UnsafeMessagesCell<'_>);
}

impl<'a> SenderTuple<'a> for () {
    type MessageType = ();

    fn register(_messages: &mut Messages) {}

    unsafe fn sender<'b>(messages: &UnsafeMessagesCell<'b>) -> Self
    where
        'b: 'a,
    {
        ()
    }

    unsafe fn sender_tl<'b>(messages: &UnsafeMessagesCell<'b>) -> Self
    where
        'b: 'a,
    {
        ()
    }

    unsafe fn sync(_messages: &mut Messages) {}

    unsafe fn flush(_messages: &UnsafeMessagesCell<'_>) {}
}

#[cfg(test)]
mod tests {
    use crate::{
        message::{Message, Messages},
        storage::channel::Sender,
        tuple::message::SenderTuple,
    };

    #[derive(Debug)]
    pub struct MessageA;

    impl Message for MessageA {}

    #[derive(Debug)]
    pub struct MessageB;

    impl Message for MessageB {}

    #[test]
    fn test() {
        let mut messages = Messages::new();
        messages.register::<MessageA>();
        messages.register::<MessageB>();
        unsafe {
            let (mut a, mut b) = <(Sender<MessageA>, Sender<MessageB>) as SenderTuple>::sender_tl(
                &messages.unsafe_cell(),
            );
            a.send(MessageA);
            a.send(MessageA);
            a.send(MessageA);
            a.send(MessageA);
            a.send(MessageA);
            b.send(MessageB);
            <(Sender<MessageA>, Sender<MessageB>) as SenderTuple>::sync(&mut messages);
        }
        println!("{:?}", messages.messages::<MessageA>());
        println!("{:?}", messages.messages::<MessageB>());
    }
}
