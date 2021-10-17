use crate::utf8_parser::{
    basic,
    basic::{multispace0, one_char},
    input::position,
    pt::Spanned,
    util, BaseErrorKind, ErrorTree, Expectation, IOk, IResultLookahead, Input, InputParseErr,
    InputParseError, OutputResult,
};

pub fn delimited<'a, F, G, H, O, OI1, OI2>(
    first: F,
    second: G,
    third: H,
) -> impl FnMut(Input<'a>) -> IResultLookahead<O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<OI1>,
    G: FnMut(Input<'a>) -> IResultLookahead<O>,
    H: FnMut(Input<'a>) -> IResultLookahead<OI2>,
{
    terminated(preceded(first, second), third)
}

pub fn pair<'a, F, G, O1, O2>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, (O1, O2)>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O1>,
    G: FnMut(Input<'a>) -> IResultLookahead<'a, O2>,
{
    move |input: Input| first(input)?.and_then(&mut second, |first, second| (first, second))
}

pub fn preceded<'a, F, G, O, OI>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, OI>,
    G: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| first(input)?.and_then(&mut second, |_first, second| second)
}

pub fn terminated<'a, F, G, O, OI>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
    G: FnMut(Input<'a>) -> IResultLookahead<'a, OI>,
{
    move |input: Input| {
        let IOk {
            remaining,
            discarded_error,
            parsed,
        } = first(input)?;
        second(remaining).map(|ok| ok.prepend_err(discarded_error).replace(parsed))
    }
}

pub fn recognize<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResultLookahead<Input<'a>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    move |input: Input| {
        let copy_of_input = input;

        parser(copy_of_input)?.and_then(
            |remaining_input| {
                Ok((
                    remaining_input,
                    input.slice(..input.offset_to(&remaining_input)),
                )
                    .into())
            },
            |_first, second| second,
        )
    }
}

pub fn lookahead<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| match parser(input) {
        Err(InputParseErr::Recoverable(e)) | Err(InputParseErr::Fatal(e)) => {
            Err(InputParseErr::recoverable(e))
        }
        Ok(x) => Ok(x),
    }
}

pub fn cut<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| match parser(input) {
        Err(InputParseErr::Recoverable(e)) | Err(InputParseErr::Fatal(e)) => {
            Err(InputParseErr::fatal(e))
        }
        Ok(x) => Ok(x),
    }
}

pub fn alt2<'a, F, G, O>(mut f: F, mut g: G) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
    G: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| match f(input) {
        Err(InputParseErr::Recoverable(first)) => match g(input) {
            Err(InputParseErr::Recoverable(second)) => {
                Err(InputParseErr::recoverable(ErrorTree::alt(first, second)))
            }
            Err(InputParseErr::Fatal(second)) => {
                Err(InputParseErr::fatal(ErrorTree::alt(first, second)))
            }
            res => res,
        },
        res => res,
    }
}

/// Converts recoverable errors into `None`
pub fn opt<'a, O, F>(mut f: F) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, Option<O>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| {
        let i = input;
        match f(input) {
            Ok(ok) => Ok(ok.map(Some)),
            Err(InputParseErr::Recoverable(e)) => Ok(IOk {
                remaining: i,
                parsed: None,
                discarded_error: Some(e),
            }),
            Err(e) => Err(e),
        }
    }
}

pub fn context<'a, F, O>(
    context: &'static str,
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    context_final(context, false, f)
}

pub fn context_final<'a, F, O>(
    context: &'static str,
    is_final: bool,
    mut f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |i: Input| match f(i) {
        Ok(o) => Ok(o),
        Err(InputParseErr::Recoverable(e)) => Err(InputParseErr::recoverable(
            InputParseError::add_context(i, context, is_final, e),
        )),
        Err(InputParseErr::Fatal(e)) => Err(InputParseErr::fatal(InputParseError::add_context(
            i, context, is_final, e,
        ))),
    }
}

pub fn many0<'a, O, F>(mut f: F) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, Vec<O>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |mut i: Input| {
        let mut acc = Vec::new();
        loop {
            let len = i.len();
            match f(i) {
                Err(InputParseErr::Recoverable(e)) => {
                    return Ok(IOk {
                        remaining: i,
                        parsed: acc,
                        discarded_error: Some(e),
                    })
                }
                Err(e) => return Err(e),
                Ok(ok) => {
                    // infinite loop check: the utf8_parser must always consume
                    if ok.remaining.len() == len {
                        unimplemented!("infinite loop - utf8_parser not consuming?");
                    }

                    // TODO: if there was a discarded error, we forget it here
                    // TODO: is that correct?
                    i = ok.remaining;
                    acc.push(ok.parsed);
                }
            }
        }
    }
}

pub fn map<'a, O, O2>(
    mut parser: impl FnMut(Input<'a>) -> IResultLookahead<'a, O>,
    map: impl Fn(O) -> O2 + Clone,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O2> {
    move |input: Input| Ok(parser(input)?.map(&map))
}

pub fn map_res<'a, O, O2>(
    mut parser: impl FnMut(Input<'a>) -> IResultLookahead<'a, O>,
    map: impl Fn(O) -> OutputResult<'a, O2> + Clone,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O2> {
    move |input: Input| parser(input)?.map_res(&map)
}

pub fn take_while1<'a>(
    condition: impl Fn(char) -> bool + Clone,
    expectation: Expectation,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Input<'a>> {
    map_res(take_while(condition), move |m: Input| match m.is_empty() {
        true => util::base_err_res(m, expectation),
        false => Ok(m),
    })
}

pub fn take_while_m_n<'a>(
    m: usize,
    n: usize,
    condition: impl Fn(char) -> bool + Clone,
    expectation: Expectation,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Input<'a>> {
    assert!(m <= n);

    let mut counter = 0;

    map_res(
        take_while(move |c| {
            if counter == n {
                false
            } else {
                counter += 1;
                condition(c)
            }
        }),
        move |input: Input| {
            if input.len() < m {
                util::base_err_res(input.slice(input.len()..), expectation)
            } else {
                Ok(input)
            }
        },
    )
}

pub fn take_while(
    mut condition: impl FnMut(char) -> bool,
) -> impl FnMut(Input) -> IResultLookahead<Input> {
    move |input: Input| match input.char_indices().find(|(_ind, c)| !condition(*c)) {
        Some((ind, _)) => Ok(input.take_split(ind)),
        None => Ok(input.take_split(input.len())),
    }
}

pub fn fold_many0<'a, O, F, G, H, R>(
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(Input<'a>) -> IResultLookahead<R>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
{
    move |input: Input| {
        let mut res = init();
        let mut input = input;

        loop {
            let copy_of_input = input;
            let len = input.len();
            match f(copy_of_input) {
                Ok(ok) => {
                    // infinite loop check: the utf8_parser must always consume
                    if ok.remaining.len() == len {
                        todo!()
                        //return Err(InputParseErr::Error(E::from_error_kind(input, ErrorKind::Many0)));
                    }

                    // TODO: again, forgetting discarded error
                    res = g(res, ok.parsed);
                    input = ok.remaining;
                }
                Err(InputParseErr::Recoverable(e)) => {
                    return Ok(IOk {
                        remaining: input,
                        parsed: res,
                        discarded_error: Some(e),
                    });
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub fn take1_if(
    condition: impl Fn(char) -> bool,
    expectation: Expectation,
) -> impl Fn(Input) -> IResultLookahead<Input> {
    move |input: Input| match input.chars().next().map(|t| (t, condition(t))) {
        Some((c, true)) => Ok(input.take_split(c.len_utf8())),
        _ => Err(InputParseErr::fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(expectation),
        })),
    }
}

pub fn comma_list0<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<O>>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    let with_trailing = many0(terminated(
        spanned(f.clone()),
        lookahead(basic::one_char(',')),
    ));

    map(
        pair(with_trailing, opt(spanned(f))),
        |(mut list, last): (Vec<_>, Option<_>)| {
            list.extend(last);
            list
        },
    )
}

#[cfg(test)]
pub fn comma_list0_lookahead<'a, F: 'a, O: std::fmt::Debug + 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<O>>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    comma_list0(move |input| lookahead(f.clone())(input))
}

pub fn comma_list1<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<O>>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    let comma = basic::one_char(',');
    map(
        pair(
            spanned(f.clone()),
            opt(preceded(lookahead(comma), comma_list0(f))),
        ),
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
pub fn comma_list1_lookahead<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<O>>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    comma_list1(move |input| lookahead(f.clone())(input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf8_parser::{basic::tag, test_util::eval};

    #[test]
    fn test_comma_list0() {
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "").len(), 0);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "a").len(), 1);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "a,").len(), 1);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "a,a").len(), 2);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "a,a,").len(), 2);
    }

    #[test]
    fn test_comma_list0_ws() {
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), " a ").len(), 1);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), " a ,").len(), 1);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "a , a ").len(), 2);
        assert_eq!(eval!(comma_list0_lookahead(tag("a")), "a , a ,").len(), 2);
    }

    #[test]
    fn test_comma_list1() {
        assert!(eval!(@result comma_list1_lookahead(tag("a")), "").is_err());
        assert!(eval!(@result comma_list1_lookahead(tag("a")), ",").is_err());
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a").len(), 1);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a,").len(), 1);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a,a").len(), 2);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a,a,").len(), 2);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a,a,a").len(), 3);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a,a,a,").len(), 3);
    }

    #[test]
    fn test_comma_list1_ws() {
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), " a ").len(), 1);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), " a , ").len(), 1);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), " a , a ").len(), 2);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), " a , a ,").len(), 2);
        assert_eq!(
            eval!(comma_list1_lookahead(tag("a")), " a , a , a ").len(),
            3
        );
        assert_eq!(
            eval!(comma_list1_lookahead(tag("a")), "a , a , a ,").len(),
            3
        );
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
                .parsed
                .len(),
            0
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new(""))
                .unwrap()
                .parsed
                .len(),
            0
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new("c"))
                .unwrap()
                .parsed
                .len(),
            0
        );
        assert_eq!(
            take_while(|c| c == 'a' || c == 'b')(Input::new("b"))
                .unwrap()
                .parsed
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

pub fn spanned<'a, F: 'a, O>(mut inner: F) -> impl FnMut(Input<'a>) -> IResultLookahead<Spanned<O>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
    O: 'a,
{
    ws(move |input: Input<'a>| {
        position(input)?.then_res(&mut inner, |start, second: IResultLookahead<O>| {
            second?.and_then(position, |value, end| Spanned { start, value, end })
        })
    })
}

pub fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(Input<'a>) -> IResultLookahead<O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    delimited(multispace0, inner, multispace0)
}

/// Like
///
/// ```text
/// delimited(
///     one_char(start_tag),
///     inner,
///     one_char(end_tag),
/// )
/// ```
///
/// but respects the discarded error of `inner` in case of error
/// and forgets it in case of success.
pub fn block<'a, F: 'a, O>(
    start_tag: char,
    mut inner: F,
    end_tag: char,
) -> impl FnMut(Input<'a>) -> IResultLookahead<O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    move |input| {
        let ok = preceded(lookahead(one_char(start_tag)), &mut inner)(input)?;
        match one_char(end_tag)(ok.remaining) {
            Ok(ok_end) => Ok(ok_end.replace(ok.parsed)),
            Err(e) => Err(ok
                .discarded_error
                .map(InputParseErr::Recoverable)
                .unwrap_or(e)), // TODO: maybe alt?
        }
    }
}
