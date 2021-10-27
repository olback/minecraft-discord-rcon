#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serenity(#[from] serenity::prelude::SerenityError),
    #[error(transparent)]
    Rcon(#[from] rcon::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
