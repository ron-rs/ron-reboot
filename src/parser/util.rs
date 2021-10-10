use crate::ast::Spanned;
use crate::parser::{spanned, IResult, Input};
use nom::branch::alt;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{AsChar, InputIter, Slice};
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation};
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

pub fn one_char(c: char) -> impl Fn(Input) -> IResult<char> {
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

pub fn comma_list0<'a, F: 'a, O: Clone + 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResult<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResult<O>,
{
    alt((comma_list1(f), multispace0.value(vec![])))
}

pub fn comma_list1<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResult<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResult<O>,
{
    terminated(
        separated_list1(tag(","), spanned(f)),
        opt(tag(",").precedes(multispace0)),
    )
}
