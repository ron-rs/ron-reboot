#![allow(unused)]

macro_rules! eval {
    ($parser:expr,$input:expr) => {
        $crate::test_util::unwrap_pr1(eval!(@result $parser, $input))
    };
    (@result $parser:expr,$input:expr) => {
        ($parser)($crate::parser::Input::new($input))
    };
}

pub(crate) use eval;

use crate::parser::{Input, InputParseErr, InputParseError};

pub fn unwrap_pr1<T>(r: Result<(Input, T), InputParseErr>) -> T {
    match r {
        Ok((_, the_value)) => the_value,
        Err(InputParseErr::Error(e) | InputParseErr::Failure(e)) => {
            panic!("{}", e)
        }
        Err(e) => {
            panic!("{}", e)
        }
    }
}

pub fn unwrap_display<T>(r: Result<T, crate::error::Error>) -> T {
    match r {
        Ok(the_value) => the_value,
        Err(r) => panic!("{}", r),
    }
}
