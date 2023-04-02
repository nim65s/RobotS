/// Normally Open Relay on a GPIO
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Deserialize, Serialize, MaxSize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Relay {
    Close,
    Open,
}

impl From<bool> for Relay {
    fn from(val: bool) -> Self {
        if val {
            Self::Close
        } else {
            Self::Open
        }
    }
}
