use std::{
    error,
    fmt::{self, Display, Formatter},
};

use zip::result::ZipError;

#[derive(Debug)]
pub(crate) enum Error {
    Reqwest(reqwest::Error),
    Zip(ZipError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reqwest(err) => err.fmt(f),
            Self::Zip(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Reqwest(err) => err.source(),
            Self::Zip(err) => err.source(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<ZipError> for Error {
    fn from(value: ZipError) -> Self {
        Self::Zip(value)
    }
}
