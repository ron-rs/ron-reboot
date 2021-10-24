pub use self::{
    ident::ident,
    number::{decimal, signed_integer, unsigned_integer},
    raw_str::parse_raw_string as raw_str,
    str::unescaped_str,
    string::parse_string as escaped_string,
};
use crate::utf8_parser::{basic::one_of_tags, combinators::context, IResultLookahead, Input};

pub mod ident;
pub mod number;
mod raw_str;
mod str;
mod string;

pub fn bool(input: Input) -> IResultLookahead<bool> {
    context("bool", one_of_tags(&["true", "false"], &[true, false]))(input)
}
