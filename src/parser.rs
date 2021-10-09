use crate::ast::{Expr, Ident, Integer, KeyValue, Sign, Spanned, Struct, UnsignedInteger};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, multispace0, one_of};
use nom::combinator::{map, map_res, opt};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;
use nom_locate::{position, LocatedSpan};
use std::str::FromStr;

pub type Input<'a> = LocatedSpan<&'a str>;

fn spanned<'a, F: 'a, O, E: ParseError<Input<'a>>>(
    mut inner: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Spanned<O>, E>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O, E>,
    O: 'a
{
    ws(move |input: Input<'a>| {
        let (input, start) = position(input)?;
        let (input, value) = inner(input)?;
        let (input, end) = position(input)?;

        Ok((input, Spanned { start, value, end }))
    })
}

fn ws<'a, F: 'a, O, E: ParseError<Input<'a>>>(
    inner: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, O, E>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn ident(input: Input) -> IResult<Input, Ident> {
    map_res(alphanumeric1, Ident::from_input)(input)
}

pub fn sign(input: Input) -> IResult<Input, Sign> {
    map_res(one_of("+-"), Sign::from_char)(input)
}

pub fn unsigned(input: Input) -> IResult<Input, UnsignedInteger> {
    map_res(digit1, |digits: Input| {
        u64::from_str(digits.fragment()).map(|number| UnsignedInteger { number })
    })(input)
}

pub fn integer(input: Input) -> IResult<Input, Integer> {
    let (input, sign) = opt(spanned(sign))(input)?;
    // Need to create temp var for borrow checker
    let x = map(spanned(unsigned), |number| Integer {
        sign: sign.clone(),
        number,
    })(input);

    x
}

fn ident_val_pair(input: Input) -> IResult<Input, KeyValue<Ident>> {
    let pair = separated_pair(spanned(ident), tag(":"), spanned(expr));
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

fn struct_inner<'a>(
    input: Input<'a>,
) -> IResult<Input<'a>, Vec<Spanned<'a, KeyValue<'a, Ident<'a>>>>> {
    separated_list1(tag(","), spanned(ident_val_pair))(input)
}

fn block<'a, F: 'a, O, E: ParseError<Input<'a>>>(
    start_tag: &'a str,
    inner: F,
    end_tag: &'a str,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, O, E>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O, E>,
{
    terminated(preceded(tag(start_tag), ws(inner)), tag(end_tag))
}

pub fn r#struct(input: Input) -> IResult<Input, Struct> {
    let (input, struct_ident) = opt(spanned(ident))(input)?;
    // Need to create temp var for borrow checker
    let x = map(spanned(block("(", struct_inner, ")")), |fields| Struct {
        fields,
        ident: struct_ident.clone(),
    })(input);

    x
}

pub fn expr(input: Input) -> IResult<Input, Expr> {
    alt((
        map(r#struct, Expr::from_struct),
        map(integer, Expr::Integer),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::ast::{Expr, Ident, Integer, Sign, Struct};
    use crate::parser::{expr, ident, integer, r#struct, sign, Input};

    macro_rules! eval {
        ($parser:ident,$input:expr) => {
            $parser(Input::new($input)).unwrap().1
        };
        (@result $parser:ident,$input:expr) => {
            $parser(Input::new($input))
        };
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
    fn basic_ident() {
        assert_eq!(eval!(ident, "Config"), Ident("Config"));
    }
}
