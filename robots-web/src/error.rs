#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(feature = "ssr")]
    #[error("RobotS drv error: {0}")]
    Robots(#[from] robots_drv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Leptos error: {0}")]
    LeptosConfig(#[from] leptos::leptos_config::errors::LeptosConfigError),

    #[error("uart driver error")]
    UartDriver,
}

pub type Result<T> = core::result::Result<T, Error>;
