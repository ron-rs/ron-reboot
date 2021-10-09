use crate::ast::{Ident, Integer, Sign, Struct};
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, multispace0, one_of};
use nom::combinator::{map_res, opt};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;
use std::str::FromStr;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn ident(input: &str) -> IResult<&str, Ident> {
    map_res(alphanumeric1, Ident::new)(input)
}

pub fn sign(input: &str) -> IResult<&str, Sign> {
    map_res(one_of("+-"), Sign::from_char)(input)
}

pub fn integer(input: &str) -> IResult<&str, Integer> {
    let (input, sign) = opt(sign)(input)?;
    // Need to create temp var for borrow checker
    let x = map_res(digit1, |digits| {
        u64::from_str(digits).map(|number| Integer {
            number,
            sign: sign.clone(),
        })
    })(input);

    x
}

fn key_val_pair(input: &str) -> IResult<&str, (Ident, Integer)> {
    separated_pair(ws(ident), tag(":"), ws(integer))(input)
}

fn struct_inner<'a>(
    input: &'a str,
    struct_ident: Option<Ident<'a>>,
) -> IResult<&'a str, Struct<'a>> {
    map_res(separated_list1(tag(","), key_val_pair), move |fields| {
        Struct::new(struct_ident.clone(), fields)
    })(input)
}

pub fn r#struct(input: &str) -> IResult<&str, Struct> {
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

#[cfg(test)]
mod tests {
    use crate::ast::{Ident, Integer, Sign, Struct};
    use crate::parser::{ident, integer, r#struct, sign};
    use nom::error::ErrorKind::OneOf;

    #[test]
    fn structs() {
        let basic_struct = Struct {
            ident: Some(Ident("Pos")),
            fields: vec![
                (Ident("x"), Integer::new_test(Some(Sign::Negative), 3)),
                (Ident("y"), Integer::new_test(None, 4)),
            ],
        };

        assert_eq!(r#struct("Pos(x:-3,y:4)").unwrap().1, basic_struct);
        assert_eq!(
            r#struct("Pos  (\tx: -3, y       : 4\n\n)").unwrap().1,
            basic_struct
        );
    }

    #[test]
    fn signs() {
        assert_eq!(sign("+").unwrap().1, Sign::Positive);
        assert_eq!(sign("-").unwrap().1, Sign::Negative);
        assert_eq!(
            sign("*"),
            Err(nom::Err::Error(nom::error::Error {
                input: "*",
                code: OneOf
            }))
        );
    }

    #[test]
    fn integers() {
        assert_eq!(
            integer("-1").unwrap().1,
            Integer::new_test(Some(Sign::Negative), 1)
        );
        assert_eq!(integer("123").unwrap().1, Integer::new_test(None, 123));
        assert_eq!(
            integer("+123").unwrap().1,
            Integer::new_test(Some(Sign::Positive), 123)
        );
    }

    #[test]
    fn basic_ident() {
        assert_eq!(ident("Config").unwrap().1, Ident("Config"));
    }
}
