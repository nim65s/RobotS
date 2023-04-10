use robots_drv::{Receiver, Sender};

lazy_static::lazy_static! {
    pub static ref RX: (Sender, Receiver) = async_channel::unbounded();
    pub static ref TX: (Sender, Receiver) = async_channel::unbounded();
}
