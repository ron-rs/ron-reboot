use crate::parser::{IResult, Input};
use nom::{AsChar, InputIter, Slice};
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation};

pub fn one_char(c: char) -> impl Fn(Input) -> IResult<Input, char> {
    move |input: Input| match input.iter_elements().next().map(|t| {
        let b = t.as_char() == c;
        (&c, b)
    }) {
        Some((c, true)) => Ok((input.slice(c.len()..), c.as_char())),
        _ => Err(nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::Char(c)),
        })),
    }
}
