use crate::parser::{Expectation, Input, IResultLookahead};
use crate::parser::ast::Ident;
use crate::parser::char_categories::{is_ident_first_char, is_ident_other_char};
use crate::parser::combinators::{context, map, preceded, recognize, take1_if, take_while};

fn ident_first_char(input: Input) -> IResultLookahead<Input> {
    take1_if(
        is_ident_first_char,
        Expectation::OneOfExpectations(&[Expectation::Alpha, Expectation::Char('_')]),
    )(input)
}

fn ident_inner(input: Input) -> IResultLookahead<Input> {
    recognize(preceded(ident_first_char, take_while(is_ident_other_char)))(input)
}

pub fn ident(input: Input) -> IResultLookahead<Ident> {
    context("ident", map(ident_inner, Ident::from_input))(input)
}
