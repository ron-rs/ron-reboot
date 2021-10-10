#![allow(dead_code)]

use crate::ast::Spanned;
use crate::parser::error::{BaseErrorKind, ErrorTree, Expectation};
use crate::parser::{spanned, IResult, Input};

use nom::branch::alt;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{AsChar, InputIter, Parser, Slice};
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

pub fn take_if_c(
    condition: impl Fn(char) -> bool,
    expectations: &'static [Expectation],
) -> impl Fn(Input) -> IResult<Input> {
    move |input: Input| match input.iter_elements().next().map(|t| {
        (t, condition(t))
    }) {
        Some((c, true)) => Ok((input.slice(c.len()..), input.slice(1..))),
        _ => Err(nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfExpectations(expectations)),
        })),
    }
}

pub fn take_if_c_char(
    condition: impl Fn(char) -> bool + 'static,
    expectations: &'static [Expectation],
) -> impl Fn(Input) -> IResult<char> {
    move |input | take_if_c(&condition, expectations).map(|input: Input| input.fragment().chars().next().unwrap()).parse(input)
}

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

#[inline]
pub fn one_of_chars<O: Clone>(
    one_of: &'static str,
    mapping: &'static [O]
) -> impl Fn(Input) -> IResult<O> {
    move |input: Input| match input.iter_elements().next().map(|t| {
        let b = one_of.chars().position(|c| c == t);
        (t, b)
    }) {
        Some((c, Some(i))) => Ok((input.slice(c.len()..), mapping[i].clone())),
        _ => Err(nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfChars(one_of)),
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

pub fn comma_list1<'a, F: 'a, O: 'a>(f: F) -> impl FnMut(Input<'a>) -> IResult<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResult<O>,
{
    terminated(
        separated_list1(tag(","), spanned(f)),
        opt(tag(",").precedes(multispace0)),
    )
}
