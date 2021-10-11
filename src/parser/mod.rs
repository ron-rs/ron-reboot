use std::str::FromStr;

use crate::parser::input::position;
use crate::{
    ast::{
        Attribute, Decimal, Expr, Extension, Ident, Integer, KeyValue, List, Map, Ron, Sign,
        SignedInteger, Spanned, Struct, UnsignedInteger,
    },
    parser::{
        char_categories::{is_digit, is_digit_first, is_ident_first_char, is_ident_other_char},
        util::{
            alt2, comma_list0, comma_list1, context, cut, delimited, many0, map, map_res,
            multispace0, one_char, one_of_chars, one_of_tags, opt, pair, preceded, recognize, tag,
            take_if_c, take_while, terminated,
        },
    },
};

pub type Input<'a> = LocatedSpan<'a>;
pub type InputParseError<'a> = ErrorTree<Input<'a>>;
pub type IResult<'a, O> = nom::IResult<Input<'a>, O, ErrorTree<Input<'a>>>;
pub type OutputResult<'a, O> = Result<O, nom::Err<InputParseError<'a>>>;

mod char_categories;
mod error;
mod input;
mod string;
mod util;

pub use self::{
    error::{BaseErrorKind, ErrorTree, Expectation},
    input::{LocatedSpan, Location, Offset},
    string::parse_string as string,
};

pub fn spanned<'a, F: 'a, O>(mut inner: F) -> impl FnMut(Input<'a>) -> IResult<Spanned<O>>
where
    F: FnMut(Input<'a>) -> IResult<O>,
    O: 'a,
{
    ws(move |input: Input<'a>| {
        let (input, start) = position(input)?;
        let (input, value) = inner(input)?;
        let (input, end) = position(input)?;

        Ok((input, Spanned { start, value, end }))
    })
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(Input<'a>) -> IResult<O>
where
    F: FnMut(Input<'a>) -> IResult<O>,
{
    delimited(multispace0, inner, multispace0)
}

fn ident_first_char(input: Input) -> IResult<Input> {
    take_if_c(
        is_ident_first_char,
        &[Expectation::Alpha, Expectation::Char('_')],
    )(input)
}

fn ident_inner(input: Input) -> IResult<Input> {
    recognize(preceded(ident_first_char, take_while(is_ident_other_char)))(input)
}

pub fn ident(input: Input) -> IResult<Ident> {
    context("ident", map(ident_inner, Ident::from_input))(input)
}

pub fn sign(input: Input) -> IResult<Sign> {
    one_of_chars("+-", &[Sign::Positive, Sign::Negative])(input)
}

fn parse_u64(input: Input) -> OutputResult<u64> {
    u64::from_str(input.fragment()).map_err(|e| {
        nom::Err::Error(ErrorTree::Base {
            location: input,
            kind: BaseErrorKind::External(Box::new(e)),
        })
    })
}

fn decimal_unsigned(input: Input) -> IResult<u64> {
    map_res(take_while(is_digit), parse_u64)(input)
}

fn decimal_unsigned_no_start_with_zero(input: Input) -> IResult<u64> {
    map_res(
        recognize(preceded(
            take_if_c(is_digit_first, &[Expectation::DigitFirst]),
            take_while(is_digit),
        )),
        parse_u64,
    )(input)
}

pub fn unsigned(input: Input) -> IResult<UnsignedInteger> {
    map(decimal_unsigned_no_start_with_zero, |number| {
        UnsignedInteger { number }
    })(input)
}

pub fn signed_integer(input: Input) -> IResult<SignedInteger> {
    let (input, sign) = sign(input)?;
    // Need to create temp var for borrow checker
    let x = map(decimal_unsigned, |number| SignedInteger {
        sign: sign.clone(),
        number,
    })(input);

    x
}

pub fn integer(input: Input) -> IResult<Integer> {
    context(
        "integer",
        alt2(
            map(unsigned, Integer::Unsigned),
            map(signed_integer, Integer::Signed),
        ),
    )(input)
}

fn decimal_exp(input: Input) -> IResult<Option<(Option<Sign>, u16)>> {
    opt(preceded(
        one_of_chars("eE", &[(), ()]),
        pair(opt(sign), map(decimal_unsigned, |n| n as u16)),
    ))(input)
}

/// e.g.
///
/// * `+1.23e3`
/// * `-5.0`
/// * `1222.00`
fn decimal_std(input: Input) -> IResult<Decimal> {
    let (input, sign) = opt(sign)(input)?;
    // Need to create temp var for borrow checker
    let x = map(
        pair(
            terminated(decimal_unsigned, one_char('.')),
            pair(decimal_unsigned, decimal_exp),
        ),
        |(whole, (fractional, exp))| Decimal::new(sign.clone(), Some(whole), fractional, exp),
    )(input);

    x
}

/// A decimal without a whole part e.g. `.01`
fn decimal_frac(input: Input) -> IResult<Decimal> {
    // Need to create temp var for borrow checker
    let x = map(
        preceded(one_char('.'), pair(decimal_unsigned, decimal_exp)),
        |(fractional, exp)| Decimal::new(None, None, fractional, exp),
    )(input);

    x
}

fn decimal(input: Input) -> IResult<Decimal> {
    context("decimal", alt2(decimal_std, decimal_frac))(input)
}

fn ident_val_pair(input: Input) -> IResult<KeyValue<Ident>> {
    let pair = pair(
        terminated(spanned(ident), cut(one_char(':'))),
        spanned(expr),
    );
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

fn block<'a, F: 'a, O>(
    start_tag: char,
    inner: F,
    end_tag: char,
) -> impl FnMut(Input<'a>) -> IResult<O>
where
    F: FnMut(Input<'a>) -> IResult<O>,
{
    #[allow(unused_parens)]
    delimited(
        one_char(start_tag),
        inner,
        /*TODO: conditional cut*/ (one_char(end_tag)),
    )
}

pub fn r#struct(input: Input) -> IResult<Struct> {
    let ident_struct = opt(spanned(ident));
    let untagged_struct = spanned(block('(', ws(comma_list0(ident_val_pair)), ')'));
    // Need to create temp var for borrow checker
    let x = map(
        context("struct", pair(ident_struct, untagged_struct)),
        |(ident, fields)| Struct { fields, ident },
    )(input);

    x
}

fn key_val_pair(input: Input) -> IResult<KeyValue<Expr>> {
    let pair = pair(terminated(spanned(expr), cut(one_char(':'))), spanned(expr));
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

pub fn rmap(input: Input) -> IResult<Map> {
    map(
        context(
            "map",
            spanned(block('{', ws(comma_list0(key_val_pair)), '}')),
        ),
        |fields| Map { entries: fields },
    )(input)
}

pub fn list(input: Input) -> IResult<List> {
    context(
        "list",
        block(
            '[',
            map(ws(comma_list0(expr)), |elements| List { elements }),
            ']',
        ),
    )(input)
}

pub fn tuple(input: Input) -> IResult<List> {
    context(
        "tuple",
        block(
            '(',
            map(comma_list0(expr), |elements| List { elements }),
            ')',
        ),
    )(input)
}

pub fn bool(input: Input) -> IResult<bool> {
    context("bool", one_of_tags(&["true", "false"], &[true, false]))(input)
}

fn inner_str(input: Input) -> IResult<&str> {
    map(take_while(|c| c != '"' && c != '\\'), |x: Input| {
        x.fragment()
    })(input)
}

pub fn unescaped_str(input: Input) -> IResult<&str> {
    delimited(tag("\""), inner_str, tag("\""))(input)
}

fn extension_name(input: Input) -> IResult<Extension> {
    one_of_tags(
        &["unwrap_newtypes", "implicit_some"],
        &[Extension::UnwrapNewtypes, Extension::ImplicitSome],
    )(input)
}

fn attribute_enable(input: Input) -> IResult<Attribute> {
    let start = preceded(tag("enable"), ws(one_char('(')));
    let end = one_char(')');

    delimited(
        start,
        map(spanned(comma_list1(extension_name)), Attribute::Enable),
        end,
    )(input)
}

pub fn attribute(input: Input) -> IResult<Attribute> {
    let start = preceded(
        preceded(one_char('#'), ws(one_char('!'))),
        ws(one_char('[')),
    );
    let end = one_char(']');

    context("attribute", delimited(start, ws(attribute_enable), end))(input)
}

macro_rules! alt {
    ($p1:expr) => { $p1 };
    ($p1:expr, $p2:expr) => { alt2($p1, $p2) };
    ($p1:expr, $p2:expr, $($p:expr),+) => {
        alt!(alt2($p1, $p2), alt!($($p),*))
    };
}

fn expr_inner(input: Input) -> IResult<Expr> {
    alt!(
        map(bool, Expr::Bool),
        map(tuple, Expr::Tuple),
        map(list, Expr::List),
        map(rmap, Expr::Map),
        map(r#struct, Expr::Struct),
        map(integer, Expr::Integer),
        map(decimal, Expr::Decimal),
        map(unescaped_str, Expr::Str),
        map(string, Expr::String)
    )(input)
}

pub fn expr(input: Input) -> IResult<Expr> {
    context("expression", expr_inner)(input)
}

fn ron_inner(input: Input) -> IResult<Ron> {
    map(
        pair(many0(spanned(attribute)), spanned(expr)),
        |(attributes, expr)| Ron { attributes, expr },
    )(input)
}

pub fn ron(input: &str) -> Result<Ron, InputParseError> {
    let input = Input::new(input);

    match ron_inner(input) {
        Ok((i, ron)) if i.is_empty() => Ok(ron),
        Ok((i, _)) => Err(ErrorTree::expected(i, Expectation::Eof)),
        Err(nom::Err::Failure(e)) | Err(nom::Err::Error(e)) => Err(e),
        Err(nom::Err::Incomplete(_e)) => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::eval;

    #[test]
    fn trailing_commas() {
        let input = "Transform(pos: 5,)";
        assert_eq!(
            eval!(r#struct, input),
            Struct::new_test(
                Some("Transform"),
                vec![("pos", UnsignedInteger::new(5).to_expr())]
            )
        );
    }

    #[test]
    fn missing_colon() {
        let input = "Transform(pos 5)";
        assert!(eval!(@result expr, input).is_err());
    }

    #[test]
    fn exprs_struct() {
        let input = "Pos(x:-3,y:4)";
        assert_eq!(Expr::Struct(eval!(r#struct, input)), eval!(expr, input));
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
    fn attributes() {
        assert_eq!(
            eval!(attribute, "#![enable(implicit_some)]"),
            Attribute::enables_test(vec![Extension::ImplicitSome])
        );
        assert_eq!(
            eval!(attribute, "# ! [  enable (  implicit_some   ) ]  "),
            Attribute::enables_test(vec![Extension::ImplicitSome])
        );

        assert_eq!(
            eval!(
                attribute,
                "# ! [  enable (  implicit_some  , unwrap_newtypes   ) ]  "
            ),
            Attribute::enables_test(vec![Extension::ImplicitSome, Extension::UnwrapNewtypes])
        );
    }

    #[test]
    fn lists() {
        assert_eq!(
            eval!(list, "[1, 2]"),
            List::new_test(vec![
                UnsignedInteger::new(1).to_expr(),
                UnsignedInteger::new(2).to_expr()
            ])
        );
        assert_eq!(
            eval!(list, "[ 1, 2, ]"),
            List::new_test(vec![
                UnsignedInteger::new(1).to_expr(),
                UnsignedInteger::new(2).to_expr()
            ])
        );
        assert_eq!(eval!(list, "[  ]"), List::new_test(vec![]));
    }

    #[test]
    fn maps() {
        let int_n3: Integer = Integer::new_test(Some(Sign::Negative), 3);
        let int_4: Integer = Integer::new_test(None, 4);
        let expr_int_n3: Expr = int_n3.to_expr();
        let expr_int_4: Expr = int_4.to_expr();

        let basic_struct =
            Struct::new_test(Some("Pos"), vec![("x", expr_int_n3), ("y", expr_int_4)]);

        let basic_map = Map::new_test(vec![
            (
                Expr::Str("my map key :)"),
                Expr::Struct(basic_struct.clone()),
            ),
            (Expr::Struct(basic_struct), Expr::Bool(false)),
        ]);

        assert_eq!(
            eval!(
                rmap,
                r#"
{
    "my map key :)": Pos(x: -3, y: 4),
    Pos(x: -3, y: 4): false,
}
"#
            ),
            basic_map
        );
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
        assert_eq!(eval!(r#struct, "Pos(x:-3,y:4,)"), basic_struct);
        assert_eq!(eval!(r#struct, "Pos(x:-3,y:4,  )"), basic_struct);
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
    fn ident_underscore() {
        assert_eq!(eval!(ident, "_start"), Ident("_start"));
        assert_eq!(eval!(ident, "ends_"), Ident("ends_"));
        assert_eq!(
            eval!(ident, "_very_many_underscores_"),
            Ident("_very_many_underscores_")
        );
        assert_eq!(
            eval!(ident, "sane_identifier_for_a_change"),
            Ident("sane_identifier_for_a_change")
        );
    }

    #[test]
    fn invalid_ident() {
        assert!(eval!(@result ident, "1hello").is_err());
    }

    #[test]
    fn basic_ident() {
        assert_eq!(eval!(ident, "Config"), Ident("Config"));
        assert_eq!(
            eval!(ident, "doesany1usenumbers"),
            Ident("doesany1usenumbers")
        );
    }
}
