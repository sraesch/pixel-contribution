use image::ImageError;
use pixel_contrib_types::Error as TypeError;
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

impl From<ImageError> for Error {
    fn from(error: ImageError) -> Self {
        Error::IO(format!("Image Error: {}", error))
    }
}

impl From<TypeError> for Error {
    fn from(error: TypeError) -> Self {
        match error {
            TypeError::IO(err) => Error::IO(err),
            TypeError::Internal(err) => Error::Internal(err),
            TypeError::InvalidArgument(err) => Error::InvalidArgument(err),
        }
    }
}

/// The result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;
