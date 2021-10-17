use crate::utf8_parser::{
    char_categories::is_ws, combinators, util, BaseErrorKind, ErrorTree, Expectation,
    IResultLookahead, Input, InputParseErr,
};

pub fn multispace0(input: Input) -> IResultLookahead<Input> {
    combinators::take_while(is_ws)(input)
}

pub fn multispace1(input: Input) -> IResultLookahead<Input> {
    combinators::take_while1(is_ws, Expectation::Multispace)(input)
}

pub fn tag(tag: &'static str) -> impl Clone + Fn(Input) -> IResultLookahead<Input> {
    let tag_len = tag.len();

    move |input: Input| match input.fragment().starts_with(tag) {
        true => Ok(input.take_split(tag_len)),
        false => util::base_err(input, Expectation::Tag(tag)),
    }
}

pub fn one_char(c: char) -> impl Fn(Input) -> IResultLookahead<char> {
    move |input: Input| match input.chars().next().map(|t| {
        let b = t == c;
        (&c, b)
    }) {
        Some((&c, true)) => Ok((input.slice(c.len_utf8()..), c)),
        _ => Err(InputParseErr::fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::Char(c)),
        })),
    }
}

pub fn one_of_chars<O: Clone>(
    one_of: &'static str,
    mapping: &'static [O],
) -> impl Fn(Input) -> IResultLookahead<O> {
    assert_eq!(one_of.len(), mapping.len());

    move |input: Input| match input.chars().next().map(|t| {
        let b = one_of.chars().position(|c| c == t);
        (t, b)
    }) {
        Some((c, Some(i))) => Ok((input.slice(c.len_utf8()..), mapping[i].clone())),
        _ => Err(InputParseErr::fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfChars(one_of)),
        })),
    }
}

pub fn one_of_tags<O: Clone>(
    one_of: &'static [&'static str],
    mapping: &'static [O],
) -> impl Fn(Input) -> IResultLookahead<O> {
    move |input: Input| match one_of
        .iter()
        .enumerate()
        .find(|(_, &t)| input.fragment().starts_with(t))
    {
        Some((i, tag)) => Ok((input.slice(tag.len()..), mapping[i].clone())),
        _ => Err(InputParseErr::fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfTags(one_of)),
        })),
    }
}
