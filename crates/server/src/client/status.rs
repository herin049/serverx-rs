#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ClientStatus {
    Init,
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
}
