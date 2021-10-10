#![allow(dead_code)]

use nom::{
    branch::alt, combinator::opt, multi::separated_list1, sequence::terminated, AsChar, InputIter,
    InputTake, Parser, Slice,
};
use nom_supreme::ParserExt;

use crate::parser::char_categories::is_ws;
use crate::{
    ast::Spanned,
    parser::{
        error::{BaseErrorKind, ErrorTree, Expectation},
        spanned, IResult, Input, OutputResult,
    },
};

#[inline]
fn base_err<T>(input: Input, expectation: Expectation) -> IResult<T> {
    Err(nom::Err::Error(ErrorTree::Base {
        location: input,
        kind: BaseErrorKind::Expected(expectation),
    }))
}

#[inline]
fn base_err_res<T>(input: Input, expectation: Expectation) -> OutputResult<T> {
    Err(nom::Err::Error(ErrorTree::Base {
        location: input,
        kind: BaseErrorKind::Expected(expectation),
    }))
}

pub fn multispace0(input: Input) -> IResult<Input> {
    take_while(is_ws)(input)
}

pub fn map<'a, O, O2>(
    mut parser: impl FnMut(Input<'a>) -> IResult<'a, O>,
    map: impl Fn(O) -> O2 + Clone,
) -> impl FnMut(Input<'a>) -> IResult<'a, O2> {
    move |input: Input| {
        let (input, o1) = parser(input)?;
        Ok((input, map(o1)))
    }
}

pub fn map_res<'a, O, O2>(
    mut parser: impl FnMut(Input<'a>) -> IResult<'a, O>,
    map: impl Fn(O) -> OutputResult<'a, O2> + Clone,
) -> impl FnMut(Input<'a>) -> IResult<'a, O2> {
    move |input: Input| {
        let (input, o1) = parser(input)?;
        Ok((input, map(o1)?))
    }
}

pub fn take_while1<'a>(
    condition: impl Fn(char) -> bool + Clone,
    expectations: &'static [Expectation],
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>> {
    map_res(take_while(condition), move |m: Input| match m.is_empty() {
        true => base_err_res(m, Expectation::OneOfExpectations(expectations)),
        false => Ok(m),
    })
}

pub fn take_while(
    condition: impl Fn(char) -> bool + Clone,
) -> impl Clone + Fn(Input) -> IResult<Input> {
    move |input: Input| match input
        .char_indices()
        .skip_while(|(_ind, c)| condition(*c))
        .next()
    {
        Some((ind, _)) => Ok(input.take_split(ind)),
        None => Ok(input.take_split(input.len())),
    }
}

pub fn tag(tag: &'static str) -> impl Clone + Fn(Input) -> IResult<Input> {
    let tag_len = tag.len();

    move |input: Input| match input.starts_with(tag) {
        true => Ok(input.take_split(tag_len)),
        false => base_err(input, Expectation::Tag(tag)),
    }
}

pub fn take_if_c(
    condition: impl Fn(char) -> bool,
    expectations: &'static [Expectation],
) -> impl Fn(Input) -> IResult<Input> {
    move |input: Input| match input.iter_elements().next().map(|t| (t, condition(t))) {
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
    move |input| {
        take_if_c(&condition, expectations)
            .map(|input: Input| input.fragment().chars().next().unwrap())
            .parse(input)
    }
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
    mapping: &'static [O],
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
        separated_list1(one_char(','), spanned(f)),
        opt(one_char(',').precedes(multispace0)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_while() {
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new("ababcabab"))
                .unwrap()
                .1
                .len(),
            4
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new("cababcabab"))
                .unwrap()
                .1
                .len(),
            0
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new(""))
                .unwrap()
                .1
                .len(),
            0
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new("c"))
                .unwrap()
                .1
                .len(),
            0
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new("b"))
                .unwrap()
                .1
                .len(),
            1
        );
    }
}
