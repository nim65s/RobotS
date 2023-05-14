#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RobotS lib error: {0}")]
    Robots(#[from] robots_drv::Error),

    #[error("Async channel SendError: {0}")]
    SendError(#[from] async_channel::SendError<robots_drv::Cmd>),

    #[error("Async channel RecvError: {0}")]
    RecvError(#[from] async_channel::RecvError),
}

impl From<Error> for leptos::ServerFnError {
    fn from(error: Error) -> Self {
        Self::ServerError(error.to_string())
    }
}

pub type Result<T> = core::result::Result<T, Error>;
