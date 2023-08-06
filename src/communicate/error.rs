use serde_json::Error as SerdeError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid io operation")]
    Io(#[from] std::io::Error),
    #[error("system error")]
    System(#[from] nix::Error),
    #[error("serialize error")]
    Serde(#[from] SerdeError),
    #[error("unexpected file mode")]
    UnexpectedFileMode,
}

pub type Result<T> = std::result::Result<T, Error>;
