use crate::ast::{
    Decimal, Expr, Ident, Integer, KeyValue, Sign, SignedInteger, Spanned, Struct, UnsignedInteger,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, multispace0, one_of};
use nom::combinator::{cut, map, map_res, opt};
use nom::error::{context, ParseError, VerboseError};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom_locate::{position, LocatedSpan};
use std::str::FromStr;
use crate::parser::util::one_char;

pub type Input<'a> = LocatedSpan<&'a str>;
pub type IResult<'a, I, O> = nom::IResult<I, O, crate::error::Error>;

mod string;
mod util;

pub use self::string::parse_string as string;

pub fn spanned<'a, F: 'a, O>(
    mut inner: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Spanned<O>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
    O: 'a,
{
    ws(move |input: Input<'a>| {
        let (input, start) = position(input)?;
        let (input, value) = inner(input)?;
        let (input, end) = position(input)?;

        Ok((input, Spanned { start, value, end }))
    })
}

fn ws<'a, F: 'a, O>(
    inner: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, O>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn ident(input: Input) -> IResult<Input, Ident> {
    map_res(alphanumeric1, Ident::from_input)(input)
}

pub fn sign(input: Input) -> IResult<Input, Sign> {
    map_res(one_of("+-"), Sign::from_char)(input)
}

fn decimal_unsigned(input: Input) -> IResult<Input, u64> {
    map_res(digit1, |digits: Input| u64::from_str(digits.fragment()))(input)
}

pub fn unsigned(input: Input) -> IResult<Input, UnsignedInteger> {
    map(decimal_unsigned, |number| UnsignedInteger { number })(input)
}

pub fn signed_integer(input: Input) -> IResult<Input, SignedInteger> {
    let (input, sign) = sign(input)?;
    // Need to create temp var for borrow checker
    let x = map(decimal_unsigned, |number| SignedInteger {
        sign: sign.clone(),
        number,
    })(input);

    x
}

pub fn integer(input: Input) -> IResult<Input, Integer> {
    alt((
        map(unsigned, Integer::Unsigned),
        map(signed_integer, Integer::Signed),
    ))(input)
}

fn decimal_exp(input: Input) -> IResult<Input, Option<(Option<Sign>, u16)>> {
    opt(preceded(
        alt((one_char('e'), one_char('E'))),
        pair(opt(sign), map(decimal_unsigned, |n| n as u16)),
    ))(input)
}

/// e.g.
///
/// * `+1.23e3`
/// * `-5.0`
/// * `1222.00`
fn decimal_std(input: Input) -> IResult<Input, Decimal> {
    let (input, sign) = opt(sign)(input)?;
    // Need to create temp var for borrow checker
    let x = map(
        separated_pair(
            decimal_unsigned,
            one_char('.'),
            pair(decimal_unsigned, decimal_exp),
        ),
        |(whole, (fractional, exp))| Decimal::new(sign.clone(), Some(whole), fractional, exp),
    )(input);

    x
}

/// A decimal without a whole part e.g. `.01`
fn decimal_frac(input: Input) -> IResult<Input, Decimal> {
    // Need to create temp var for borrow checker
    let x = map(
        preceded(one_char('.'), pair(decimal_unsigned, decimal_exp)),
        |(fractional, exp)| Decimal::new(None, None, fractional, exp),
    )(input);

    x
}

fn decimal(input: Input) -> IResult<Input, Decimal> {
    alt((decimal_std, decimal_frac))(input)
}

fn ident_val_pair(input: Input) -> IResult<Input, KeyValue<Ident>> {
    let pair = separated_pair(spanned(ident), context("colon", cut(one_char(':'))), spanned(expr));
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

fn struct_inner(input: Input) -> IResult<Input, Vec<Spanned<KeyValue<Ident>>>> {
    separated_list1(tag(","), spanned(ident_val_pair))(input)
}

fn block<'a, F: 'a, O>(
    start_tag: char,
    inner: F,
    end_tag: char,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, O>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    delimited(
        one_char(start_tag),
        inner,
        context("closing bracket", cut(one_char(end_tag))),
    )
}

pub fn r#struct(input: Input) -> IResult<Input, Struct> {
    let (input, struct_ident) = opt(spanned(ident))(input)?;
    // Need to create temp var for borrow checker
    let x = map(
        spanned(block('(', struct_inner, ')')),
        |fields| Struct {
            fields,
            ident: struct_ident.clone(),
        },
    )(input);

    x
}

pub fn expr(input: Input) -> IResult<Input, Expr> {
    alt((
        map(r#struct, Expr::from_struct),
        map(integer, Expr::Integer),
        map(decimal, Expr::Decimal),
        map(string, Expr::String),
    ))(input)
}

#[cfg(test)]
mod tests {
    //use crate::ast::{Expr, Ident, Integer, Sign, Struct};
    use super::*;

    macro_rules! eval {
        ($parser:ident,$input:expr) => {
            $parser(Input::new($input)).unwrap().1
        };
        (@result $parser:ident,$input:expr) => {
            $parser(Input::new($input))
        };
    }

    #[test]
    fn good_missing_colon_error() {
        let input = "Transform(pos 5)";
        assert_eq!(
            eval!(@result expr, input).unwrap_err().to_string(),
            r#"Parsing Failure: Chain { error: ContextError { offset: Offset(1, 14), context: "colon" }, cause: ExpectedChar(':') }"#.to_owned()
        );
    }

    #[test]
    fn exprs_struct() {
        let input = "Pos(x:-3,y:4)";
        assert_eq!(
            Expr::from_struct(eval!(r#struct, input)),
            eval!(expr, input)
        );
    }

    #[test]
    fn strings() {
        assert_eq!(eval!(string, r#""Hello strings!""#), "Hello strings!");
        assert_eq!(
            eval!(string, r#""Newlines are\n great!""#),
            "Newlines are\n great!"
        );
        assert_eq!(eval!(string, r#""So is /😂\\""#), "So is /😂\\");
        assert_eq!(eval!(string, r#""So is \u{00AC}""#), "So is \u{00AC}");
    }

    #[test]
    fn exprs_int() {
        for input in ["-4123", "111", "+821"] {
            assert_eq!(eval!(integer, input).to_expr(), eval!(expr, input));
        }
    }

    #[test]
    fn structs() {
        let int_n3: Integer = Integer::new_test(Some(Sign::Negative), 3);
        let int_4: Integer = Integer::new_test(None, 4);
        let expr_int_n3: Expr = int_n3.to_expr();
        let expr_int_4: Expr = int_4.to_expr();

        let basic_struct =
            Struct::new_test(Some("Pos"), vec![("x", expr_int_n3), ("y", expr_int_4)]);

        assert_eq!(eval!(r#struct, "Pos(x:-3,y:4)"), basic_struct);
        assert_eq!(
            eval!(r#struct, "Pos  (\tx: -3, y       : 4\n\n)"),
            basic_struct
        );
    }

    #[test]
    fn signs() {
        assert_eq!(eval!(sign, "+"), Sign::Positive);
        assert_eq!(eval!(sign, "-"), Sign::Negative);
        assert!(eval!(@result sign, "*").is_err());
    }

    #[test]
    fn integers() {
        assert_eq!(
            eval!(integer, "-1"),
            Integer::new_test(Some(Sign::Negative), 1)
        );
        assert_eq!(eval!(integer, "123"), Integer::new_test(None, 123));
        assert_eq!(
            eval!(integer, "+123"),
            Integer::new_test(Some(Sign::Positive), 123)
        );
    }

    #[test]
    fn decimals() {
        assert_eq!(
            eval!(decimal, "-1.0"),
            Decimal::new(Some(Sign::Negative), Some(1), 0, None)
        );
        assert_eq!(
            eval!(decimal, "123.00"),
            Decimal::new(None, Some(123), 0, None)
        );
        assert_eq!(
            eval!(decimal, "+1.23e+2"),
            Decimal::new(
                Some(Sign::Positive),
                Some(1),
                23,
                Some((Some(Sign::Positive), 2))
            )
        );
        assert_eq!(
            eval!(decimal, ".123e3"),
            Decimal::new(None, None, 123, Some((None, 3)))
        );
    }

    #[test]
    fn basic_ident() {
        assert_eq!(eval!(ident, "Config"), Ident("Config"));
    }
}
