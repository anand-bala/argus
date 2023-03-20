pub mod expr;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("redeclaration of identifier")]
    IdentifierRedeclaration,
    #[error("insufficient number of arguments")]
    IncompleteArgs,
}

pub type ArgusError = Error;
pub type ArgusResult<T> = Result<T, Error>;

