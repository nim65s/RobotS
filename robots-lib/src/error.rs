/// Errors handling in the code: fallible fonctions will return a Result over this
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("postcard error {0}")]
    Postcard(postcard::Error),

    #[cfg(any(feature = "std", feature = "wasm"))]
    #[error("Serde Json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
