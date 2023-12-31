use cad_import::Error as CADError;
use quick_error::quick_error;
use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IO(err: String) {
            display("{}", err)
        }
        Internal(err: String) {
            display("{}", err)
        }
        InvalidMatrix(err: String) {
            display("{}", err)
        }
        InvalidArgument(err: String) {
            display("{}", err)
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(format!("{}", error))
    }
}

impl From<CADError> for Error {
    fn from(error: CADError) -> Self {
        match error {
            CADError::IO(err) => Error::IO(err),
            CADError::InvalidFormat(err) => Error::IO(err),
            CADError::Indices(err) => Error::IO(err),
            _ => Error::Internal(error.to_string()),
        }
    }
}

/// The result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;
