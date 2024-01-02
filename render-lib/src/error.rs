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
        GraphicsAPI(err: String) {
            display("{}", err)
        }
        Shader(err: std::string::String) {
            display("{}", err)
        }
        Texture(err: std::string::String) {
            display("{}", err)
        }
        FrameBuffer(err: std::string::String) {
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

/// The result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;
