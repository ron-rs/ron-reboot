#[cfg(feature = "utf8_parser_serde1")]
pub use self::serde::from_str;
use self::{
    containers::{list, r#struct, rmap, tuple},
    error::{BaseErrorKind, Expectation, InputParseErr},
    input::Input,
    primitive::{bool, decimal, escaped_string, signed_integer, unescaped_str, unsigned_integer},
    ron::expr,
};
pub use self::{
    error::{ErrorTree, InputParseError},
    ron::ron as ast_from_str,
};
use crate::ast;

//pub type IResultFatal<'a, O> = Result<(Input<'a>, O), InputParseError<'a>>;
type IResultLookahead<'a, O> = Result<(Input<'a>, O), InputParseErr<'a>>;
type OutputResult<'a, O> = Result<O, InputParseErr<'a>>;

/// Basic parsers which receive `Input`
mod basic;
/// Tables for fast lookup of char categories
mod char_categories;
/// Parser combinators which take one or more parsers and modify / combine them
mod combinators;
/// RON container parsers
mod containers;
/// Parser error collection
mod error;
pub mod error_fmt;
/// `Input` abstraction to slice the input that is being parsed and keep track of the line + column
mod input;
/// RON primitive parsers
mod primitive;
/// Parsers for arbitrary RON expression & top-level RON
mod ron;
#[cfg(feature = "utf8_parser_serde1")]
mod serde;
#[cfg(test)]
mod tests;
/// Utility functions for parsing
mod util;
// Integration tests cannot import this without the feature gate
// (not sure why that is...)
#[cfg(any(test, feature = "test"))]
pub mod test_util;
