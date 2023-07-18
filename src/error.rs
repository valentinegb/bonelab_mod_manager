use std::{
    error,
    fmt::{self, Display, Formatter},
    io,
};

use crate::app_data;

#[derive(Debug)]
pub(super) enum Error {
    AppData(app_data::Error),
    Modio(modio::Error),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppData(err) => err.fmt(f),
            Self::Modio(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::AppData(err) => err.source(),
            Self::Modio(err) => err.source(),
            Self::Io(err) => err.source(),
        }
    }
}

impl From<app_data::Error> for Error {
    fn from(value: app_data::Error) -> Self {
        Self::AppData(value)
    }
}

impl From<modio::Error> for Error {
    fn from(value: modio::Error) -> Self {
        Self::Modio(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
