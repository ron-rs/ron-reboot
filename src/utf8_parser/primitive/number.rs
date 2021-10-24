use std::str::FromStr;

use crate::utf8_parser::{
    basic::{one_char, one_of_chars},
    char_categories::{is_digit, is_digit_first},
    combinators::{
        alt2, context, lookahead, map, map_res, opt, pair, preceded, recognize, take1_if,
        take_while, terminated,
    },
    pt::{Decimal, Sign, SignedInteger, UnsignedInteger},
    BaseErrorKind, ErrorTree, Expectation, IResultLookahead, Input, InputParseErr, OutputResult,
};

pub fn sign(input: Input) -> IResultLookahead<Sign> {
    one_of_chars("+-", &[Sign::Positive, Sign::Negative])(input)
}

fn parse_u64_radix(radix_input: (u32, Input)) -> OutputResult<u64> {
    u64::from_str_radix(radix_input.1.fragment(), radix_input.0).map_err(|e| {
        InputParseErr::fatal(ErrorTree::Base {
            location: radix_input.1,
            kind: BaseErrorKind::External(Box::new(e)),
        })
    })
}

fn parse_u64_dec(input: Input) -> OutputResult<u64> {
    u64::from_str(input.fragment()).map_err(|e| {
        InputParseErr::fatal(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::External(Box::new(e)),
        })
    })
}

fn decimal_unsigned(input: Input) -> IResultLookahead<u64> {
    map_res(take_while(is_digit), parse_u64_dec)(input)
}

fn fractional_part(input: Input) -> IResultLookahead<(u64, u16)> {
    map_res(take_while(is_digit), |input| {
        Ok((parse_u64_dec(input)?, input.len() as u16))
    })(input)
}

fn decimal_unsigned_no_leading_zero(input: Input) -> IResultLookahead<u64> {
    map_res(
        recognize(alt2(
            recognize(lookahead(one_char('0'))),
            preceded(
                take1_if(is_digit_first, Expectation::DigitFirst),
                take_while(is_digit),
            ),
        )),
        parse_u64_dec,
    )(input)
}

fn alt_radix_unsigned(input: Input) -> IResultLookahead<u64> {
    map_res(
        pair(
                lookahead(preceded(one_char('0'), one_of_chars("box", &[2, 8, 16]))),
            take_while(|c| c.is_ascii_hexdigit()),
        ),
        parse_u64_radix,
    )(input)
}

pub fn unsigned_integer(input: Input) -> IResultLookahead<UnsignedInteger> {
    map(alt2(alt_radix_unsigned, decimal_unsigned_no_leading_zero), |number| UnsignedInteger {
        number,
    })(input)
}

pub fn signed_integer(input: Input) -> IResultLookahead<SignedInteger> {
    map(pair(lookahead(sign), decimal_unsigned), |(sign, number)| {
        SignedInteger { sign, number }
    })(input)
}

#[cfg(test)]
pub fn integer(input: Input) -> IResultLookahead<crate::utf8_parser::pt::Integer> {
    context(
        "integer",
        alt2(
            map(signed_integer, crate::utf8_parser::pt::Integer::Signed),
            map(unsigned_integer, crate::utf8_parser::pt::Integer::Unsigned),
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
    map(
        pair(
            opt(lookahead(sign)),
            pair(
                terminated(decimal_unsigned, lookahead(one_char('.'))),
                pair(fractional_part, decimal_exp),
            ),
        ),
        |(sign, (whole, ((fractional, fractional_digits), exp)))| {
            Decimal::new(sign, Some(whole), fractional, fractional_digits, exp)
        },
    )(input)
}

/// A decimal without a whole part e.g. `.01`
fn decimal_frac(input: Input) -> IResultLookahead<Decimal> {
    // Need to create temp var for borrow checker
    let x = map(
        preceded(lookahead(one_char('.')), pair(fractional_part, decimal_exp)),
        |((fractional, fractional_digits), exp)| {
            Decimal::new(None, None, fractional, fractional_digits, exp)
        },
    )(input);

    x
}

pub fn decimal(input: Input) -> IResultLookahead<Decimal> {
    context("decimal", alt2(decimal_frac, decimal_std))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf8_parser::{expr, pt::Expr, test_util::eval};

    #[test]
    fn exprs_decimals() {
        for input in ["-41.23", "11.1", ".1E-4"] {
            assert_eq!(Expr::Decimal(eval!(decimal, input)), eval!(expr, input));
        }
    }

    #[test]
    fn exprs_int() {
        for input in ["-4123", "111", "+821", "0"] {
            assert_eq!(eval!(integer, input).to_expr(), eval!(expr, input));
        }
    }

    #[test]
    fn signs() {
        assert_eq!(eval!(sign, "+"), Sign::Positive);
        assert_eq!(eval!(sign, "-"), Sign::Negative);
        assert!(eval!(@result sign, "*").is_err());
    }

    #[test]
    fn radix_integers() {
        assert_eq!(
            eval!(integer, "0x0"),
            crate::utf8_parser::pt::Integer::new_test(None, 0)
        );
        assert_eq!(
            eval!(integer, "0x1"),
            crate::utf8_parser::pt::Integer::new_test(None, 1)
        );
        assert_eq!(
            eval!(integer, "0x1B"),
            crate::utf8_parser::pt::Integer::new_test(None, 27)
        );

        assert_eq!(
            eval!(integer, "0o17"),
            crate::utf8_parser::pt::Integer::new_test(None, 15)
        );

        assert_eq!(
            eval!(integer, "0b0101"),
            crate::utf8_parser::pt::Integer::new_test(None, 5)
        );
    }

    #[test]
    fn integers() {
        assert_eq!(
            eval!(integer, "-1"),
            crate::utf8_parser::pt::Integer::new_test(Some(Sign::Negative), 1)
        );
        assert_eq!(
            eval!(integer, "123"),
            crate::utf8_parser::pt::Integer::new_test(None, 123)
        );
        assert_eq!(
            eval!(integer, "+123"),
            crate::utf8_parser::pt::Integer::new_test(Some(Sign::Positive), 123)
        );
    }

    #[test]
    fn decimals() {
        assert_eq!(
            eval!(decimal, "-1.0"),
            Decimal::new(Some(Sign::Negative), Some(1), 0, 1, None)
        );
        assert_eq!(
            eval!(decimal, "123.00"),
            Decimal::new(None, Some(123), 0, 2, None)
        );
        assert_eq!(
            eval!(decimal, "+1.23e+2"),
            Decimal::new(
                Some(Sign::Positive),
                Some(1),
                23,
                2,
                Some((Some(Sign::Positive), 2))
            )
        );
        assert_eq!(
            eval!(decimal, ".123e3"),
            Decimal::new(None, None, 123, 3, Some((None, 3)))
        );
        assert_eq!(
            eval!(decimal, ".123E-3"),
            Decimal::new(None, None, 123, 3, Some((Some(Sign::Negative), 3)))
        );
    }
}
