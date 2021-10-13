use crate::parser::{basic::one_of_tags, combinators::context, IResultLookahead, Input};

pub mod ident;
pub mod number;
mod str;
mod string;

pub use self::{
    ident::ident,
    number::{decimal, signed_integer, unsigned_integer},
    str::unescaped_str,
    string::parse_string as escaped_string,
};

pub fn bool(input: Input) -> IResultLookahead<bool> {
    context("bool", one_of_tags(&["true", "false"], &[true, false]))(input)
}
