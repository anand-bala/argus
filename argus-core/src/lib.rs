pub mod expr;
pub mod prelude;
pub mod signals;

use std::time::Duration;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("redeclaration of identifier")]
    IdentifierRedeclaration,
    #[error("insufficient number of arguments")]
    IncompleteArgs,

    #[error("cannot push value to non-sampled signal")]
    InvalidPushToSignal,
    #[error(
        "trying to create a non-monotonically signal, signal end time ({end_time:?}) > sample time point \
         ({current_sample:?})"
    )]
    NonMonotonicSignal { end_time: Duration, current_sample: Duration },

    #[error("invalid operation due to bad type")]
    InvalidOperation,

    #[error("name not in signal trace")]
    SignalNotPresent,

    #[error("incorrect signal type")]
    InvalidSignalType,
}

pub type ArgusError = Error;
pub type ArgusResult<T> = Result<T, Error>;
