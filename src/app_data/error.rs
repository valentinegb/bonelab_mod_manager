use std::{
    env::VarError,
    error,
    fmt::{self, Display, Formatter},
    io,
};

#[derive(Debug)]
pub(crate) enum Error {
    Io(io::Error),
    Postcard(postcard::Error),
    Var(VarError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Postcard(err) => err.fmt(f),
            Self::Var(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(err) => err.source(),
            Self::Postcard(err) => err.source(),
            Self::Var(err) => err.source(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<postcard::Error> for Error {
    fn from(value: postcard::Error) -> Self {
        Self::Postcard(value)
    }
}

impl From<VarError> for Error {
    fn from(value: VarError) -> Self {
        Self::Var(value)
    }
}
