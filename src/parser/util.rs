use crate::{
    parser::{
        error::{ErrorTree, Expectation},
        Input, InputParseErr, IResultLookahead, OutputResult,
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
