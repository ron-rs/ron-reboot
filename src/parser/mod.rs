use std::str::FromStr;

use crate::{
    ast::{
        Attribute, Decimal, Expr, Extension, Ident, KeyValue, List, Map, Ron, Sign, SignedInteger,
        Spanned, Struct, UnsignedInteger,
    },
    parser::{
        char_categories::{is_digit, is_digit_first, is_ident_first_char, is_ident_other_char},
        input::position,
        util::{
            alt2, comma_list0, comma_list1, context, cut, delimited, lookahead, many0, map,
            map_res, multispace0, one_char, one_of_chars, one_of_tags, opt, pair, preceded,
            recognize, tag, take1_if, take_while, terminated,
        },
    },
};

//pub type IResultFatal<'a, O> = Result<(Input<'a>, O), InputParseError<'a>>;
pub type IResultLookahead<'a, O> = Result<(Input<'a>, O), InputParseErr<'a>>;
pub type OutputResult<'a, O> = Result<O, InputParseErr<'a>>;

mod char_categories;
mod error;
mod input;
mod string;
mod util;

pub use self::{
    error::{BaseErrorKind, ErrorTree, Expectation, InputParseErr, InputParseError},
    input::{Input, Location, Offset},
    string::parse_string as string,
};

pub fn spanned<'a, F: 'a, O>(mut inner: F) -> impl FnMut(Input<'a>) -> IResultLookahead<Spanned<O>>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
    O: 'a,
{
    ws(move |input: Input<'a>| {
        let (input, start) = position(input)?;
        let (input, value) = inner(input)?;
        let (input, end) = position(input)?;

        Ok((input, Spanned { start, value, end }))
    })
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(Input<'a>) -> IResultLookahead<O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    delimited(multispace0, inner, multispace0)
}

fn ident_first_char(input: Input) -> IResultLookahead<Input> {
    take1_if(
        is_ident_first_char,
        Expectation::OneOfExpectations(&[Expectation::Alpha, Expectation::Char('_')]),
    )(input)
}

fn ident_inner(input: Input) -> IResultLookahead<Input> {
    recognize(preceded(ident_first_char, take_while(is_ident_other_char)))(input)
}

pub fn ident(input: Input) -> IResultLookahead<Ident> {
    context("ident", map(ident_inner, Ident::from_input))(input)
}

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
pub fn integer(input: Input) -> IResultLookahead<crate::ast::Integer> {
    context(
        "integer",
        alt2(
            map(signed_integer, crate::ast::Integer::Signed),
            map(unsigned, crate::ast::Integer::Unsigned),
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

fn decimal(input: Input) -> IResultLookahead<Decimal> {
    context("decimal", alt2(decimal_frac, decimal_std))(input)
}

fn ident_val_pair(input: Input) -> IResultLookahead<KeyValue<Ident>> {
    let pair = pair(
        lookahead(terminated(spanned(ident), one_char(':'))),
        spanned(expr),
    );
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

fn block<'a, F: 'a, O>(
    start_tag: char,
    inner: F,
    end_tag: char,
) -> impl FnMut(Input<'a>) -> IResultLookahead<O>
where
    F: FnMut(Input<'a>) -> IResultLookahead<O>,
{
    #[allow(unused_parens)]
    delimited(
        one_char(start_tag),
        inner,
        /*TODO: conditional cut*/ (one_char(end_tag)),
    )
}

fn opt_ident(input: Input) -> IResultLookahead<Option<Spanned<Ident>>> {
    opt(spanned(lookahead(ident)))(input)
}

pub fn r#struct(input: Input) -> IResultLookahead<Struct> {
    let untagged_struct = spanned(block('(', ws(comma_list1(ident_val_pair)), ')'));
    // Need to create temp var for borrow checker
    let x = map(
        context("struct", pair(opt_ident, untagged_struct)),
        |(ident, fields)| Struct { fields, ident },
    )(input);

    x
}

fn key_val_pair(input: Input) -> IResultLookahead<KeyValue<Expr>> {
    let pair = pair(terminated(lookahead(spanned(expr)), cut(one_char(':'))), spanned(expr));
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

pub fn rmap(input: Input) -> IResultLookahead<Map> {
    map(
        context(
            "map",
            spanned(block('{', ws(comma_list0(key_val_pair)), '}')),
        ),
        |fields| Map { entries: fields },
    )(input)
}

pub fn list(input: Input) -> IResultLookahead<List> {
    context(
        "list",
        block(
            '[',
            map(ws(comma_list0(|input| lookahead(expr)(input))), |elements| List { elements }),
            ']',
        ),
    )(input)
}

pub fn tuple(input: Input) -> IResultLookahead<List> {
    context(
        "tuple",
        block(
            '(',
            map(comma_list0(expr), |elements| List { elements }),
            ')',
        ),
    )(input)
}

pub fn bool(input: Input) -> IResultLookahead<bool> {
    context("bool", one_of_tags(&["true", "false"], &[true, false]))(input)
}

fn inner_str(input: Input) -> IResultLookahead<&str> {
    map(take_while(|c| c != '"' && c != '\\'), |x: Input| {
        x.fragment()
    })(input)
}

pub fn unescaped_str(input: Input) -> IResultLookahead<&str> {
    delimited(tag("\""), inner_str, tag("\""))(input)
}

fn extension_name(input: Input) -> IResultLookahead<Extension> {
    one_of_tags(
        &["unwrap_newtypes", "implicit_some"],
        &[Extension::UnwrapNewtypes, Extension::ImplicitSome],
    )(input)
}

fn attribute_enable(input: Input) -> IResultLookahead<Attribute> {
    let start = preceded(tag("enable"), ws(one_char('(')));
    let end = one_char(')');

    delimited(
        start,
        map(spanned(comma_list1(extension_name)), Attribute::Enable),
        end,
    )(input)
}

pub fn attribute(input: Input) -> IResultLookahead<Attribute> {
    let start = preceded(
        preceded(lookahead(one_char('#')), ws(one_char('!'))),
        ws(one_char('[')),
    );
    let end = one_char(']');

    context("attribute", delimited(start, ws(attribute_enable), end))(input)
}

#[derive(Clone, Debug)]
pub enum ExprClass {
    StructTuple,
    Map,
    StrString,
    List,
    Bool,
    Signed,
    Dec,
    UnsignedDec,
    LeadingIdent,
}

impl ExprClass {
    pub fn parse(input: Input) -> IResultLookahead<Self> {
        let all_but_ident = one_of_chars(
            "({\"[tf+-.0123456789",
            &[
                ExprClass::StructTuple,
                ExprClass::Map,
                ExprClass::StrString,
                ExprClass::List,
                ExprClass::Bool,
                ExprClass::Bool,
                ExprClass::Signed,
                ExprClass::Signed,
                ExprClass::Dec,
                ExprClass::Dec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
                ExprClass::UnsignedDec,
            ],
        );

        alt2(
            lookahead(all_but_ident),
            map(
                take1_if(
                    is_ident_first_char,
                    Expectation::OneOfExpectations(&[Expectation::Alpha, Expectation::Char('_')]),
                ),
                |_| ExprClass::LeadingIdent,
            ),
        )(input)
    }
}

fn expr_inner(input: Input) -> IResultLookahead<Expr> {
    // Copy input and discard its offset ("peek")
    let (_, expr_class): (Input, ExprClass) = ExprClass::parse(input)?;

    match expr_class {
        ExprClass::StructTuple => cut(alt2(map(r#struct, Expr::Struct), map(tuple, Expr::Tuple)))(input),
        ExprClass::Map => map(rmap, Expr::Map)(input),
        ExprClass::StrString => alt2(
            map(lookahead(unescaped_str), Expr::Str),
            map(string, Expr::String),
        )(input),
        ExprClass::List => map(list, Expr::List)(input),
        ExprClass::Bool => map(bool, Expr::Bool)(input),
        ExprClass::Signed => map(signed_integer, SignedInteger::to_expr)(input),
        ExprClass::Dec => map(decimal, Expr::Decimal)(input),
        ExprClass::UnsignedDec => alt2(
            map(unsigned, UnsignedInteger::to_expr),
            map(decimal, Expr::Decimal),
        )(input),
        ExprClass::LeadingIdent => map(r#struct, Expr::Struct)(input),
    }
}

pub fn expr(input: Input) -> IResultLookahead<Expr> {
    cut(context("expression", expr_inner))(input)
}

fn ron_inner(input: Input) -> IResultLookahead<Ron> {
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
        Err(InputParseErr::Fatal(e)) | Err(InputParseErr::Recoverable(e)) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::Integer, test_util::eval};

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
    fn exprs_str() {
        assert_eq!(
            Expr::Str(eval!(unescaped_str, r#""Hello strings!""#)),
            eval!(expr, r#""Hello strings!""#)
        );
    }

    #[test]
    fn exprs_string() {
        assert_eq!(
            Expr::String(eval!(string, r#""\n""#)),
            eval!(expr, r#""\n""#)
        );
        assert_eq!(
            Expr::String(eval!(string, r#""So is /😂\\""#)),
            eval!(expr, r#""So is /😂\\""#)
        );
        assert_eq!(
            Expr::String(eval!(string, r#""\\So is \u{00AC}""#)),
            eval!(expr, r#""\\So is \u{00AC}""#)
        );
    }

    #[test]
    fn strings() {
        assert_eq!(
            eval!(unescaped_str, r#""Hello strings!""#),
            "Hello strings!"
        );
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
        // TODO: find out what lookahead is missing
        assert_eq!(
            eval!(list, "[1,]"),
            List::new_test(vec![
                UnsignedInteger::new(1).to_expr(),
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
    fn lists_inner() {
        assert_eq!(
            eval!(comma_list0(|input| lookahead(expr)(input)), "1,"), vec![Spanned::new_test(UnsignedInteger::new(1).to_expr())]);
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
    fn untagged_structs() {
        let int_n3: Integer = Integer::new_test(Some(Sign::Negative), 3);
        let int_4: Integer = Integer::new_test(None, 4);
        let expr_int_n3: Expr = int_n3.to_expr();
        let expr_int_4: Expr = int_4.to_expr();

        let basic_struct = Struct::new_test(None, vec![("x", expr_int_n3), ("y", expr_int_4)]);

        assert_eq!(eval!(r#struct, "(x:-3,y:4)"), basic_struct);
        assert_eq!(eval!(r#struct, "(x:-3,y:4,)"), basic_struct);
        assert_eq!(eval!(r#struct, "(x:-3,y:4,  )"), basic_struct);
        assert_eq!(eval!(r#struct, "(\t  x: -3, y       : 4\n\n)"), basic_struct);
    }

    #[test]
    fn opt_idents() {
        let s = Spanned::new_test;

        assert_eq!(eval!(opt_ident, "Pos"), Some(s(Ident("Pos"))));
        assert_eq!(eval!(opt_ident, "_0"), Some(s(Ident("_0"))));
        assert_eq!(eval!(opt_ident, ""), None);
        assert_eq!(eval!(opt_ident, "!not an ident"), None);
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
    fn excl_mark() {
        let err = eval!(@result r#struct, r#"Example(
    xyz: Asdf(
        x: 4, yalala: !
    ),
)"#).unwrap_err();
        assert_eq!(format!("{}", err), r#"could not match "struct" at 1:1 (`E`) because
could not match "expression" at 2:11 (`A`) because
could not match "struct" at 2:11 (`A`) because
could not match "expression" at 3:24 (`!`) because
    expected one of an ascii letter or '_' at 3:24 (`!`)"#);
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
        assert_eq!(
            eval!(decimal, ".123E-3"),
            Decimal::new(None, None, 123, Some((Some(Sign::Negative), 3)))
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
