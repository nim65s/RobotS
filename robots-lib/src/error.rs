/// Errors handling in the code: fallible fonctions will return a Result over this
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("numerical error {0}")]
    Num(#[from] core::num::TryFromIntError),

    #[error("postcard error {0}")]
    Postcard(postcard::Error),

    #[cfg(any(feature = "std", feature = "wasm"))]
    #[error("Serde Json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(feature = "esp32c3")]
    #[error("interrupt error {0:?}")]
    Interrupt(esp32c3_hal::interrupt::Error),

    #[cfg(feature = "esp32c3")]
    #[error("rmt error {0:?}")]
    Rmt(esp32c3_hal::rmt::Error),

    #[cfg(feature = "esp32c3")]
    #[error("uart error {0:?}")]
    Uart(esp32c3_hal::uart::Error),

    #[cfg(feature = "esp32c3")]
    #[error("led error {0:?}")]
    Led(esp_hal_smartled::LedAdapterError),
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "esp32c3")]
impl From<esp32c3_hal::interrupt::Error> for Error {
    fn from(e: esp32c3_hal::interrupt::Error) -> Self {
        Self::Interrupt(e)
    }
}

#[cfg(feature = "esp32c3")]
impl From<esp32c3_hal::rmt::Error> for Error {
    fn from(e: esp32c3_hal::rmt::Error) -> Self {
        Self::Rmt(e)
    }
}

#[cfg(feature = "esp32c3")]
impl From<esp32c3_hal::uart::Error> for Error {
    fn from(e: esp32c3_hal::uart::Error) -> Self {
        Self::Uart(e)
    }
}

#[cfg(feature = "esp32c3")]
impl From<esp_hal_smartled::LedAdapterError> for Error {
    fn from(e: esp_hal_smartled::LedAdapterError) -> Self {
        Self::Led(e)
    }
}
