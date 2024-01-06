use image::ImageError;
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

/// The result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;
