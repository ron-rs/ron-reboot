pub use self::error::{ErrorTree, InputParseError};
#[cfg(feature = "utf8_parser_serde1")]
pub use self::serde::from_str;
use self::{
    containers::{list, rmap, tuple, untagged_struct},
    error::{BaseErrorKind, Expectation, InputParseErr},
    input::Input,
    primitive::{bool, decimal, escaped_string, signed_integer, unescaped_str, unsigned_integer},
    ron::expr,
};
use crate::{ast, ast::Ron, utf8_parser::ok::IOk};

//pub type IResultFatal<'a, O> = Result<(Input<'a>, O), InputParseError<'a>>;
type IResultLookahead<'a, O> = Result<IOk<'a, O>, InputParseErr<'a>>;
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
mod error_fmt;
/// `Input` abstraction to slice the input that is being parsed and keep track of the line + column
mod input;
mod ok;
/// RON primitive parsers
mod primitive;
/// IR for parsing which will then be converted to the AST
mod pt;
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

pub fn ast_from_str(input: &str) -> Result<Ron, crate::error::Error> {
    let pt: pt::Ron = ron::ron(input).map_err(ErrorTree::calc_locations)?;
    let ast: ast::Ron = pt.into();

    Ok(ast)
}
