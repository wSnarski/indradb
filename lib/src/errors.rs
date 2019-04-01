use serde_json::Error as JsonError;
use std::result::Result as StdResult;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "json error: {}", inner)]
    Json { inner: JsonError },
    #[fail(display = "UUID already taken")]
    UuidTaken,
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Json { inner: err }
    }
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Fail)]
pub enum ValidationError {
	#[fail(display = "invalid value")]
    InvalidValue,
    #[fail(display = "value too long")]
    ValueTooLong,
    #[fail(display = "could not increment the UUID")]
    CannotIncrementUuid,
}

pub type ValidationResult<T> = StdResult<T, ValidationError>;
