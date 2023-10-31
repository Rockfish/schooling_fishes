#[derive(Debug)]
pub enum Error {
    FileError(std::io::Error),
    ShaderError(String),
    ImageError(image::ImageError),
    ModelError(russimp::RussimpError),
    TextureError(String),
    UnknownError(&'static str),
}

impl From<std::io::Error> for Error {
    fn from(s: std::io::Error) -> Self {
        Error::FileError(s)
    }
}

impl From<image::ImageError> for Error {
    fn from(s: image::ImageError) -> Self {
        Error::ImageError(s)
    }
}

impl From<russimp::RussimpError> for Error {
    fn from(s: russimp::RussimpError) -> Self {
        Error::ModelError(s)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::UnknownError(s)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
