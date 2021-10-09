use crate::ast::{Expr, Ident, Integer, Sign, Struct};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, multispace0, one_of};
use nom::combinator::{map, map_res, opt};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;
use nom_locate::LocatedSpan;
use std::str::FromStr;

pub type Input<'a> = LocatedSpan<&'a str>;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
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

pub fn integer(input: Input) -> IResult<Input, Integer> {
    let (input, sign) = opt(sign)(input)?;
    // Need to create temp var for borrow checker
    let x = map_res(digit1, |digits: Input| {
        u64::from_str(digits.fragment()).map(|number| Integer {
            number,
            sign: sign.clone(),
        })
    })(input);

    x
}

fn ident_val_pair(input: Input) -> IResult<Input, (Ident, Expr)> {
    separated_pair(ws(ident), tag(":"), ws(expr))(input)
}

fn struct_inner<'a>(
    input: Input<'a>,
    struct_ident: Option<Ident<'a>>,
) -> IResult<Input<'a>, Struct<'a>> {
    map_res(separated_list1(tag(","), ident_val_pair), move |fields| {
        Struct::new(struct_ident.clone(), fields)
    })(input)
}

pub fn r#struct(input: Input) -> IResult<Input, Struct> {
    let (input, struct_ident) = opt(ident)(input)?;
    // Need to create temp var for borrow checker
    let x = ws(terminated(
        preceded(
            tag("("),
            ws(move |input| struct_inner(input, struct_ident.clone())),
        ),
        tag(")"),
    ))(input);

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

    const INT_N3: Integer = Integer::new_test(Some(Sign::Negative), 3);
    const INT_4: Integer = Integer::new_test(None, 4);

    const EXPR_INT_N3: Expr = INT_N3.to_expr();
    const EXPR_INT_4: Expr = INT_4.to_expr();

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
        let basic_struct = Struct {
            ident: Some(Ident("Pos")),
            fields: vec![(Ident("x"), EXPR_INT_N3), (Ident("y"), EXPR_INT_4)],
        };

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
