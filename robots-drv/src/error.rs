#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RobotS lib error: {0}")]
    Robots(#[from] robots_lib::Error),

    #[error("Channel SendError: {0}")]
    SendError(#[from] futures::channel::mpsc::SendError),

    #[error("Tokio serial error: {0}")]
    TokioSerial(#[from] tokio_serial::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
