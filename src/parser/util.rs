use crate::{
    ast::Spanned,
    parser::{
        char_categories::is_ws,
        error::{BaseErrorKind, ErrorTree, Expectation},
        spanned, IResultLookahead, Input, InputParseErr, InputParseError, OutputResult,
    },
};

#[inline]
pub fn base_err<T>(input: Input, expectation: Expectation) -> IResultLookahead<T> {
    Err(InputParseErr::Fatal(ErrorTree::expected(
        input,
        expectation,
    )))
}

#[inline]
pub fn base_err_res<T>(input: Input, expectation: Expectation) -> OutputResult<T> {
    Err(InputParseErr::Fatal(ErrorTree::expected(
        input,
        expectation,
    )))
}

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
    move |input: Input| {
        let (i, r) = first(input)?;

        second(i).map(|(i, r2)| (i, (r, r2)))
    }
}

pub fn preceded<'a, F, G, O, OI>(
    mut first: F,
    mut second: G,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, OI>,
    G: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| {
        let (i, _) = first(input)?;

        second(i)
    }
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
        let (i, r) = first(input)?;
        second(i).map(|(i, _)| (i, r))
    }
}

pub fn recognize<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResultLookahead<Input<'a>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    move |input: Input| {
        let i = input;
        match parser(i) {
            Ok((i, _)) => {
                let index = input.offset_to(&i);
                Ok((i, input.slice(..index)))
            }
            Err(e) => Err(e),
        }
    }
}

pub fn lookahead<'a, O, F>(mut parser: F) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |input: Input| match parser(input) {
        Err(InputParseErr::Recoverable(e)) | Err(InputParseErr::Fatal(e)) => {
            Err(InputParseErr::Recoverable(e))
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
            Err(InputParseErr::Fatal(e))
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
                Err(InputParseErr::Recoverable(ErrorTree::alt(first, second)))
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
            Ok((i, o)) => Ok((i, Some(o))),
            Err(InputParseErr::Recoverable(_)) => Ok((i, None)),
            Err(e) =>Err(e),
        }
    }
}

pub fn context<'a, F, O>(
    context: &'static str,
    mut f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |i: Input| match f(i) {
        Ok(o) => Ok(o),
        Err(InputParseErr::Recoverable(e)) => Err(InputParseErr::Recoverable(
            InputParseError::add_context(i, context, e),
        )),
        Err(InputParseErr::Fatal(e)) => Err(InputParseErr::Fatal(InputParseError::add_context(
            i, context, e,
        ))),
    }
}

pub fn many0<'a, O, F>(mut f: F) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, Vec<O>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<'a, O>,
{
    move |mut i: Input| {
        let mut acc = Vec::with_capacity(4);
        loop {
            let len = i.len();
            match f(i) {
                Err(InputParseErr::Recoverable(_)) => return Ok((i, acc)),
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

pub fn multispace0(input: Input) -> IResultLookahead<Input> {
    take_while(is_ws)(input)
}

pub fn multispace1(input: Input) -> IResultLookahead<Input> {
    take_while1(is_ws, Expectation::Multispace)(input)
}

pub fn map<'a, O, O2>(
    mut parser: impl FnMut(Input<'a>) -> IResultLookahead<'a, O>,
    map: impl Fn(O) -> O2 + Clone,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O2> {
    move |input: Input| {
        let (input, o1) = parser(input)?;
        Ok((input, map(o1)))
    }
}

pub fn map_res<'a, O, O2>(
    mut parser: impl FnMut(Input<'a>) -> IResultLookahead<'a, O>,
    map: impl Fn(O) -> OutputResult<'a, O2> + Clone,
) -> impl FnMut(Input<'a>) -> IResultLookahead<'a, O2> {
    move |input: Input| {
        let (input, o1) = parser(input)?;
        Ok((input, map(o1)?))
    }
}

pub fn take_while1<'a>(
    condition: impl Fn(char) -> bool + Clone,
    expectation: Expectation,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Input<'a>> {
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
                base_err_res(input.slice(input.len()..), expectation)
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
    move |i: Input| {
        let mut res = init();
        let mut input = i;

        loop {
            let i_ = input;
            let len = input.len();
            match f(i_) {
                Ok((i, o)) => {
                    // infinite loop check: the parser must always consume
                    if i.len() == len {
                        todo!()
                        //return Err(InputParseErr::Error(E::from_error_kind(input, ErrorKind::Many0)));
                    }

                    res = g(res, o);
                    input = i;
                }
                Err(InputParseErr::Recoverable(_)) => {
                    return Ok((input, res));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub fn tag(tag: &'static str) -> impl Clone + Fn(Input) -> IResultLookahead<Input> {
    let tag_len = tag.len();

    move |input: Input| match input.fragment().starts_with(tag) {
        true => Ok(input.take_split(tag_len)),
        false => base_err(input, Expectation::Tag(tag)),
    }
}

pub fn take1_if(
    condition: impl Fn(char) -> bool,
    expectation: Expectation,
) -> impl Fn(Input) -> IResultLookahead<Input> {
    move |input: Input| match input.chars().next().map(|t| (t, condition(t))) {
        Some((c, true)) => Ok(input.take_split(c.len_utf8())),
        _ => Err(InputParseErr::Fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(expectation),
        })),
    }
}

pub fn one_char(c: char) -> impl Fn(Input) -> IResultLookahead<char> {
    move |input: Input| match input.chars().next().map(|t| {
        let b = t == c;
        (&c, b)
    }) {
        Some((&c, true)) => Ok((input.slice(c.len_utf8()..), c)),
        _ => Err(InputParseErr::Fatal(ErrorTree::Base {
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
        _ => Err(InputParseErr::Fatal(ErrorTree::Base {
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
        _ => Err(InputParseErr::Fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::Expected(Expectation::OneOfTags(one_of)),
        })),
    }
}

pub fn comma_list0<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    let with_trailing = many0(terminated(spanned(f.clone()), lookahead(one_char(','))));

    map(
        pair(with_trailing, opt( spanned(f))),
        |(mut list, last): (Vec<_>, Option<_>)| {
            list.extend(last);
            list
        },
    )
}

pub fn comma_list0_lookahead<'a, F: 'a, O: std::fmt::Debug + 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<'a, O>>>
    where
        F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    comma_list0(move |input| lookahead(f.clone())(input))
}

pub fn comma_list1<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<'a, O>>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    let comma = one_char(',');
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


pub fn comma_list1_lookahead<'a, F: 'a, O: 'a>(
    f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<Vec<Spanned<'a, O>>>
    where
        F: FnMut(Input<'a>) -> IResultLookahead<O> + Clone,
{
    comma_list1(move |input| lookahead(f.clone())(input))
}

pub fn dbg<'a, F: 'a, O: std::fmt::Debug + 'a>(
    s: &'static str,
    mut f: F,
) -> impl FnMut(Input<'a>) -> IResultLookahead<O>
    where
        F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    move |input| {
        let res = f(input);
        println!("{}: {:?}", s, res);

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::eval;

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
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), " a , a , a ").len(), 3);
        assert_eq!(eval!(comma_list1_lookahead(tag("a")), "a , a , a ,").len(), 3);
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
