use crate::parser::{ast, BaseErrorKind, ErrorTree, Expectation, Input, InputParseErr, IResultLookahead, OutputResult};
use crate::parser::ast::{Decimal, Sign, SignedInteger, UnsignedInteger};
use crate::parser::basic::{one_char, one_of_chars};
use crate::parser::char_categories::{is_digit, is_digit_first};
use crate::parser::combinators::{alt2, context, lookahead, map, map_res, opt, pair, preceded, recognize, take1_if, take_while, terminated};

pub fn sign(input: Input) -> IResultLookahead<Sign> {
    one_of_chars("+-", &[Sign::Positive, Sign::Negative])(input)
}

fn parse_u64(input: Input) -> OutputResult<u64> {
    u64::from_str(input.fragment()).map_err(|e| {
        InputParseErr::Fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::External(Box::new(e)),
        })
    })
}

fn decimal_unsigned(input: Input) -> IResultLookahead<u64> {
    map_res(take_while(is_digit), parse_u64)(input)
}

fn decimal_unsigned_no_start_with_zero(input: Input) -> IResultLookahead<u64> {
    map_res(
        recognize(preceded(
            take1_if(is_digit_first, Expectation::DigitFirst),
            take_while(is_digit),
        )),
        parse_u64,
    )(input)
}

pub fn unsigned(input: Input) -> IResultLookahead<UnsignedInteger> {
    map(decimal_unsigned_no_start_with_zero, |number| {
        UnsignedInteger { number }
    })(input)
}

pub fn signed_integer(input: Input) -> IResultLookahead<SignedInteger> {
    map(pair(lookahead(sign), decimal_unsigned), |(sign, number)| {
        SignedInteger { sign, number }
    })(input)
}

#[cfg(test)]
pub fn integer(input: Input) -> IResultLookahead<ast::Integer> {
    context(
        "integer",
        alt2(
            map(signed_integer, ast::Integer::Signed),
            map(unsigned, ast::Integer::Unsigned),
        ),
    )(input)
}

fn decimal_exp(input: Input) -> IResultLookahead<Option<(Option<Sign>, u16)>> {
    opt(lookahead(preceded(
        one_of_chars("eE", &[(), ()]),
        pair(opt(lookahead(sign)), map(decimal_unsigned, |n| n as u16)),
    )))(input)
}

/// e.g.
///
/// * `+1.23e3`
/// * `-5.0`
/// * `1222.00`
fn decimal_std(input: Input) -> IResultLookahead<Decimal> {
    let (input, sign) = opt(lookahead(sign))(input)?;
    // Need to create temp var for borrow checker
    let x = map(
        pair(
            terminated(decimal_unsigned, one_char('.')),
            pair(decimal_unsigned, decimal_exp),
        ),
        |(whole, (fractional, exp))| Decimal::new(sign, Some(whole), fractional, exp),
    )(input);

    x
}

/// A decimal without a whole part e.g. `.01`
fn decimal_frac(input: Input) -> IResultLookahead<Decimal> {
    // Need to create temp var for borrow checker
    let x = map(
        preceded(
            lookahead(one_char('.')),
            pair(decimal_unsigned, decimal_exp),
        ),
        |(fractional, exp)| Decimal::new(None, None, fractional, exp),
    )(input);

    x
}

pub fn decimal(input: Input) -> IResultLookahead<Decimal> {
    context("decimal", alt2(decimal_frac, decimal_std))(input)
}
