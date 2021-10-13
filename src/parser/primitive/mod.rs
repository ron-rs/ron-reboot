use crate::parser::{Input, IResultLookahead};
use crate::parser::basic::one_of_tags;
use crate::parser::combinators::context;

pub mod ident;
pub mod number;
pub mod string;
pub mod str;

pub fn bool(input: Input) -> IResultLookahead<bool> {
    context("bool", one_of_tags(&["true", "false"], &[true, false]))(input)
}
