#![allow(dead_code)]

use nom::{error::ContextError, Err, Parser};

use crate::{
    ast::Spanned,
    parser::{
        char_categories::is_ws,
        error::{BaseErrorKind, ErrorTree, Expectation},
        spanned, IResult, Input, InputParseError, OutputResult,
    },
};

#[inline]
pub fn base_err<T>(input: Input, expectation: Expectation) -> IResult<T> {
    Err(nom::Err::Error(ErrorTree::expected(input, expectation)))
}

#[inline]
pub fn base_err_res<T>(input: Input, expectation: Expectation) -> OutputResult<T> {
    Err(nom::Err::Error(ErrorTree::expected(input, expectation)))
}

pub fn delimited<'a, F, G, H, O, OI1, OI2>(
    first: F,
    second: G,
    third: H,
) -> impl FnMut(Input<'a>) -> IResult<O>
where
    F: FnMut(Input<'a>) -> IResult<OI1>,
    G: FnMut(Input<'a>) -> IResult<O>,
    H: FnMut(Input<'a>) -> IResult<OI2>,
{
    terminated(preceded(first, second), third)
}

pub fn pair<'a, F, G, O1, O2>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResult<'a, (O1, O2)>
where
    F: FnMut(Input<'a>) -> IResult<'a, O1>,
    G: FnMut(Input<'a>) -> IResult<'a, O2>,
{
    move |input: Input| {
        let (i, r) = first(input)?;

        second(i).map(|(i, r2)| (i, (r, r2)))
    }
}

pub fn preceded<'a, F, G, O, OI>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResult<'a, O>
where
    F: FnMut(Input<'a>) -> IResult<'a, OI>,
    G: FnMut(Input<'a>) -> IResult<'a, O>,
{
    move |input: Input| {
        let (i, _) = first(input)?;

        second(i)
    }
}

pub fn terminated<'a, F, G, O, OI>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResult<'a, O>
where
    F: FnMut(Input<'a>) -> IResult<'a, O>,
    G: FnMut(Input<'a>) -> IResult<'a, OI>,
{
    move |input: Input| {
        let (i, r) = first(input)?;
        second(i.clone()).map(|(i, _)| (i, r))
    }
}

pub fn recognize<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>>
where
    F: FnMut(Input<'a>) -> IResult<O>,
{
    move |input: Input| {
        let i = input.clone();
        match parser.parse(i) {
            Ok((i, _)) => {
                let index = input.offset_to(&i);
                Ok((i, input.slice(..index)))
            }
            Err(e) => Err(e),
        }
    }
}

pub fn cut<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResult<'a, O>
where
    F: FnMut(Input<'a>) -> IResult<'a, O>,
{
    move |input: Input| match parser.parse(input) {
        Err(Err::Error(e)) => Err(Err::Failure(e)),
        rest => rest,
    }
}

pub fn alt2<'a, F, G, O>(mut f: F, mut g: G) -> impl FnMut(Input<'a>) -> IResult<'a, O>
where
    F: FnMut(Input<'a>) -> IResult<'a, O>,
    G: FnMut(Input<'a>) -> IResult<'a, O>,
{
    move |input: Input| match f(input.clone()) {
        Err(Err::Error(first)) => match g(input.clone()) {
            Err(Err::Error(second)) => Err(Err::Error(ErrorTree::alt(first, second))),
            res => res,
        },
        res => res,
    }
}

pub fn opt<'a, O, F>(mut f: F) -> impl FnMut(Input<'a>) -> IResult<'a, Option<O>>
where
    F: FnMut(Input<'a>) -> IResult<'a, O>,
{
    move |input: Input| {
        let i = input.clone();
        match f.parse(input) {
            // TODO: shouldn't this slice i?
            Ok((i, o)) => Ok((i, Some(o))),
            Err(Err::Error(_)) => Ok((i, None)),
            Err(e) => Err(e),
        }
    }
}

pub fn context<'a, F, O>(context: &'static str, mut f: F) -> impl FnMut(Input<'a>) -> IResult<'a, O>
where
    F: FnMut(Input<'a>) -> IResult<'a, O>,
{
    move |i: Input| match f.parse(i.clone()) {
        Ok(o) => Ok(o),
        Err(Err::Incomplete(i)) => Err(Err::Incomplete(i)),
        Err(Err::Error(e)) => Err(Err::Error(InputParseError::add_context(i, context, e))),
        Err(Err::Failure(e)) => Err(Err::Failure(InputParseError::add_context(i, context, e))),
    }
}

pub fn many0<'a, O, F>(mut f: F) -> impl FnMut(Input<'a>) -> IResult<'a, Vec<O>>
where
    F: FnMut(Input<'a>) -> IResult<'a, O>,
{
    move |mut i: Input| {
        let mut acc = Vec::with_capacity(4);
        loop {
            let len = i.len();
            match f.parse(i.clone()) {
                Err(Err::Error(_)) => return Ok((i, acc)),
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    // infinite loop check: the parser must always consume
                    if i1.len() == len {
                        unimplemented!("infinite loop - parser not consuming?");
                    }

                    i = i1;
                    acc.push(o);
                }
            }
        }
    }
}

pub fn multispace0(input: Input) -> IResult<Input> {
    take_while(is_ws)(input)
}

pub fn multispace1(input: Input) -> IResult<Input> {
    take_while1(is_ws, Expectation::Multispace)(input)
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
    expectation: Expectation,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>> {
    map_res(take_while(condition), move |m: Input| match m.is_empty() {
        true => base_err_res(m, expectation),
        false => Ok(m),
    })
}

pub fn take_while_m_n<'a>(
    m: usize,
    n: usize,
    condition: impl Fn(char) -> bool + Clone,
    expectation: Expectation,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>> {
    assert!(m <= n);

    let mut counter = 0;

    map_res(
        take_while(move |c| {
            if dbg!(counter) == n {
                false
            } else {
                counter += 1;
                condition(c)
            }
        }),
        move |input: Input| {
            if input.len() < m {
                base_err_res(input.slice(input.len()..), expectation)
            } else {
                Ok(input)
            }
        },
    )
}

pub fn take_while(mut condition: impl FnMut(char) -> bool) -> impl FnMut(Input) -> IResult<Input> {
    move |input: Input| match input
        .char_indices()
        .skip_while(|(_ind, c)| condition(*c))
        .next()
    {
        Some((ind, _)) => Ok(input.take_split(ind)),
        None => Ok(input.take_split(input.len())),
    }
}

pub fn fold_many0<'a, O, F, G, H, R>(
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(Input<'a>) -> IResult<R>
where
    F: FnMut(Input<'a>) -> IResult<O>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
{
    move |i: Input| {
        let mut res = init();
        let mut input = i;

        loop {
            let i_ = input.clone();
            let len = input.len();
            match f.parse(i_) {
                Ok((i, o)) => {
                    // infinite loop check: the parser must always consume
                    if i.len() == len {
                        todo!()
                        //return Err(Err::Error(E::from_error_kind(input, ErrorKind::Many0)));
                    }

                    res = g(res, o);
                    input = i;
                }
                Err(Err::Error(_)) => {
                    return Ok((input, res));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub fn tag(tag: &'static str) -> impl Clone + Fn(Input) -> IResult<Input> {
    let tag_len = tag.len();

    move |input: Input| match input.fragment().starts_with(tag) {
        true => Ok(input.take_split(tag_len)),
        false => base_err(input, Expectation::Tag(tag)),
    }
}

pub fn take_if_c(
    condition: impl Fn(char) -> bool,
    expectations: &'static [Expectation],
) -> impl Fn(Input) -> IResult<Input> {
    move |input: Input| match input.chars().next().map(|t| (t, condition(t))) {
        Some((c, true)) => Ok((input.slice(c.len_utf8()..), input.slice(1..))),
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
    move |input: Input| match input.chars().next().map(|t| {
        let b = t == c;
        (&c, b)
    }) {
        Some((&c, true)) => Ok((input.slice(c.len_utf8()..), c)),
        _ => Err(nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::Char(c)),
        })),
    }
}

pub fn one_of_chars<O: Clone>(
    one_of: &'static str,
    mapping: &'static [O],
) -> impl Fn(Input) -> IResult<O> {
    move |input: Input| match input.chars().next().map(|t| {
        let b = one_of.chars().position(|c| c == t);
        (t, b)
    }) {
        Some((c, Some(i))) => Ok((input.slice(c.len_utf8()..), mapping[i].clone())),
        _ => Err(nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfChars(one_of)),
        })),
    }
}

pub fn one_of_tags<O: Clone>(
    one_of: &'static [&'static str],
    mapping: &'static [O],
) -> impl Fn(Input) -> IResult<O> {
    move |input: Input| match one_of
        .iter()
        .enumerate()
        .find(|(_, &t)| input.fragment().starts_with(t))
    {
        Some((i, tag)) => Ok((input.slice(tag.len()..), mapping[i].clone())),
        _ => Err(nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfTags(one_of)),
        })),
    }
}

pub fn comma_list0<'a, F: 'a, O: 'a>(f: F) -> impl FnMut(Input<'a>) -> IResult<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResult<O> + Clone,
{
    let with_trailing = many0(terminated(spanned(f.clone()), one_char(',')));

    map(
        pair(with_trailing, opt(spanned(f))),
        |(mut list, last): (Vec<_>, Option<_>)| {
            list.extend(last);
            list
        },
    )
}

pub fn comma_list1<'a, F: 'a, O: 'a>(f: F) -> impl FnMut(Input<'a>) -> IResult<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResult<O> + Clone,
{
    let comma = one_char(',');
    map(
        pair(spanned(f.clone()), opt(preceded(comma, comma_list0(f)))),
        |(head, tail): (_, Option<Vec<_>>)| match tail {
            None => vec![head],
            Some(mut tail) => {
                tail.insert(0, head);
                tail
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::eval;

    #[test]
    fn test_comma_list0() {
        assert_eq!(eval!(comma_list0(tag("a")), "").len(), 0);
        assert_eq!(eval!(comma_list0(tag("a")), "a").len(), 1);
        assert_eq!(eval!(comma_list0(tag("a")), "a,").len(), 1);
        assert_eq!(eval!(comma_list0(tag("a")), "a,a").len(), 2);
        assert_eq!(eval!(comma_list0(tag("a")), "a,a,").len(), 2);
    }

    #[test]
    fn test_comma_list0_ws() {
        assert_eq!(eval!(comma_list0(tag("a")), " a ").len(), 1);
        assert_eq!(eval!(comma_list0(tag("a")), " a ,").len(), 1);
        assert_eq!(eval!(comma_list0(tag("a")), "a , a ").len(), 2);
        assert_eq!(eval!(comma_list0(tag("a")), "a , a ,").len(), 2);
    }

    #[test]
    fn test_comma_list1() {
        assert!(eval!(@result comma_list1(tag("a")), "").is_err());
        assert!(eval!(@result comma_list1(tag("a")), ",").is_err());
        assert_eq!(eval!(comma_list1(tag("a")), "a").len(), 1);
        assert_eq!(eval!(comma_list1(tag("a")), "a,").len(), 1);
        assert_eq!(eval!(comma_list1(tag("a")), "a,a").len(), 2);
        assert_eq!(eval!(comma_list1(tag("a")), "a,a,").len(), 2);
        assert_eq!(eval!(comma_list1(tag("a")), "a,a,a").len(), 3);
        assert_eq!(eval!(comma_list1(tag("a")), "a,a,a,").len(), 3);
    }

    #[test]
    fn test_comma_list1_ws() {
        assert_eq!(eval!(comma_list1(tag("a")), " a ").len(), 1);
        assert_eq!(eval!(comma_list1(tag("a")), " a , ").len(), 1);
        assert_eq!(eval!(comma_list1(tag("a")), " a , a ").len(), 2);
        assert_eq!(eval!(comma_list1(tag("a")), " a , a ,").len(), 2);
        assert_eq!(eval!(comma_list1(tag("a")), " a , a , a ").len(), 3);
        assert_eq!(eval!(comma_list1(tag("a")), "a , a , a ,").len(), 3);
    }

    #[test]
    fn test_take_while() {
        assert_eq!(
            eval!(take_while(|c| c == 'a' || c == 'b'), "ababcabab").len(),
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

    #[test]
    fn test_take_while_m_n_limits() {
        assert_eq!(
            eval!(
                take_while_m_n(0, 3, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            3
        );

        assert_eq!(
            eval!(
                take_while_m_n(0, 0, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            0
        );

        assert_eq!(
            eval!(
                take_while_m_n(0, 1, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            1
        );

        assert_eq!(
            eval!(
                take_while_m_n(2, 4, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            4
        );
    }

    #[test]
    fn test_take_while_m_n_checks() {
        assert_eq!(
            eval!(
                take_while_m_n(0, 5, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            4
        );

        assert_eq!(
            eval!(
                take_while_m_n(4, 4, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            4
        );

        assert_eq!(
            eval!(
                take_while_m_n(0, 5, |_c| false, Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            0
        );

        assert_eq!(
            eval!(
                take_while_m_n(0, 1, |_c| false, Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            0
        );

        assert_eq!(
            eval!(
                take_while_m_n(1, 4, |c| c == 'a', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            1
        );
    }

    #[test]
    fn test_take_while_m_n_requires() {
        assert_eq!(
            eval!(
                take_while_m_n(3, 6, |c| c == 'a' || c == 'b', Expectation::Alpha),
                "ababcabab"
            )
            .len(),
            4
        );

        assert!(
            eval!(@result take_while_m_n(3, 6, |c| c == 'a' || c == 'b', Expectation::Alpha), "ab")
                .is_err()
        );
        assert!(
            eval!(@result take_while_m_n(3, 6, |c| c == 'a' || c == 'b', Expectation::Alpha), "")
                .is_err()
        );
        assert!(
            eval!(@result take_while_m_n(1, 1, |c| c == 'a' || c == 'b', Expectation::Alpha), "")
                .is_err()
        );
    }
}
