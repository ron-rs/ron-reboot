use crate::parser::Input;
use err_derive::Error;
use nom::error::{ContextError, FromExternalError, ParseError};
use std::fmt::{Display, Formatter};

#[derive(Debug, Error)]
pub enum PhantomError {}

#[derive(Debug)]
pub struct Offset(pub u32, pub usize);

impl From<Input<'_>> for Offset {
    fn from(i: Input<'_>) -> Self {
        Offset(i.location_line(), i.location_offset())
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug, Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "{}", error)]
    Chain {
        error: Box<Self>,
        #[error(cause)]
        cause: Box<Self>,
    },
    //#[error(display = "parse error")]
    //NomError(#[error(source, from)] nom::error::Error<Input<'a>>),
    #[error(display = "parse error at {} ({:?})", offset, code)]
    ParseError {
        offset: Offset,
        code: nom::error::ErrorKind,
    },
    #[error(display = "parse error at {} ({})", offset, context)]
    ContextError {
        offset: Offset,
        context: &'static str,
    },
    #[error(display = "expected '{}'", _0)]
    ExpectedChar(char),
    #[error(display = "error: {}", _0)]
    AnyError(String),
}

impl ParseError<Input<'_>> for Error {
    fn from_error_kind(input: Input, kind: nom::error::ErrorKind) -> Self {
        Error::ParseError {
            offset: input.into(),
            code: kind,
        }
    }

    fn append(input: Input, code: nom::error::ErrorKind, other: Self) -> Self {
        Error::Chain {
            error: Box::new(Error::ParseError {
                offset: input.into(),
                code,
            }),
            cause: Box::new(other),
        }
    }
}

impl ContextError<Input<'_>> for Error {
    fn add_context(input: Input, context: &'static str, other: Self) -> Self {
        Error::Chain {
            error: Box::new(Error::ContextError {
                offset: input.into(),
                context,
            }),
            cause: Box::new(other),
        }
    }
}

impl<E: std::error::Error> FromExternalError<Input<'_>, E> for Error {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: Input, kind: nom::error::ErrorKind, e: E) -> Self {
        Error::Chain {
            error: Box::new(Error::AnyError(e.to_string())),
            cause: Box::new(Error::ParseError {
                offset: input.into(),
                code: kind,
            }),
        }
    }
}
