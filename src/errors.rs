use failure::{Backtrace, Context, Fail};
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::result::Result as StdResult;

pub type Result<SuccessT> = StdResult<SuccessT, Error>;

#[derive(Debug, Clone, Eq, PartialEq, Fail)]
pub enum ErrorKind {
    #[fail(display = "No endpoints to connect to.")]
    NoEndpoints,

    #[fail(display = "Address resolution error.")]
    AddressResolution,

    #[fail(display = "Attempted to read from a closed cursor.")]
    ReadFromClosedCursor,

    #[fail(display = "Failed to deserialize expected response.")]
    UnexpectedResponse,

    #[fail(display = "Timed out while waiting for result in iteration.")]
    IteratorTimeout,

    #[fail(display = "Connection error: {}", _0)]
    Connection(Cow<'static, str>),

    #[fail(
        display = "{} error code={}, span={:?}: {}",
        kind,
        code,
        span,
        message
    )]
    Server {
        kind: ServerErrorKind,
        code: i32,
        span: Box<[u32]>,
        message: Box<str>,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ServerErrorKind {
    Runtime,
    Compile,
    Client,
    Unknown,
}

impl Display for ServerErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerErrorKind::Runtime => Display::fmt("Runtime", f),
            ServerErrorKind::Compile => Display::fmt("Compile", f),
            ServerErrorKind::Client => Display::fmt("Client", f),
            ServerErrorKind::Unknown => Display::fmt("Unknown server", f),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}
