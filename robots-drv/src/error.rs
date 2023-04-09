use robots_lib::Cmd;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RobotS lib error: {0}")]
    Robots(#[from] robots_lib::Error),

    #[error("Async channel SendError: {0}")]
    SendError(#[from] async_channel::SendError<Cmd>),

    #[error("Async channel RecvError: {0}")]
    RecvError(#[from] async_channel::RecvError),

    #[error("Tokio serial error: {0}")]
    TokioSerial(#[from] tokio_serial::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
