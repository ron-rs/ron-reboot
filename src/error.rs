use std::fmt::{Display, Formatter};

use crate::parser::Location;
use crate::{
    error_fmt::ErrorTreeFmt,
    parser::{Input, InputParseError},
};

pub fn ron_err(kind: ErrorKind, start: Input, end: Input) -> Error {
    Error {
        kind,
        start: Some(start.into()),
        end: Some(end.into()),
    }
}

pub fn ron_err_custom(kind: ErrorKind, start: Option<Location>, end: Option<Location>) -> Error {
    Error { kind, start, end }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub start: Option<Location>,
    pub end: Option<Location>,
}

impl From<InputParseError<'_>> for Error {
    fn from(e: InputParseError) -> Self {
        Error {
            start: None,
            end: None,
            kind: ErrorKind::ParseError(ErrorTreeFmt::new(e).to_string()),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error {
            kind: ErrorKind::Custom(msg.to_string()),
            start: None,
            end: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.start.as_ref(), self.end.as_ref()) {
            (Some(start), Some(end)) => write!(
                f,
                "deserialization error at {} - {}: {}",
                start, end, self.kind
            ),
            (Some(start), None) => write!(f, "deserialization error at {}: {}", start, self.kind),
            _ => write!(f, "{}", self.kind),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    ExpectedBool,
    ExpectedString,
    ExpectedStrGotEscapes,
    ExpectedList,

    ParseError(String),

    Custom(String),

    #[doc(hidden)]
    __NonExhaustive,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ExpectedBool => write!(f, "expected bool"),
            ErrorKind::ExpectedStrGotEscapes => {
                write!(f, "expected zero-copy string which doesn't support escapes")
            }
            ErrorKind::ExpectedString => write!(f, "expected string"),
            ErrorKind::ExpectedList => write!(f, "expected list"),
            ErrorKind::ParseError(e) => write!(f, "parsing error: {}", e),
            ErrorKind::Custom(s) => write!(f, "{}", s),
            ErrorKind::__NonExhaustive => unimplemented!(),
        }
    }
}
