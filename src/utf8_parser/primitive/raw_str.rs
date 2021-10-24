use crate::utf8_parser::{
    basic::{one_char, repeat_char},
    combinators::{context, cut, delimited, lookahead, many0, map, pair, take_until},
    input::Input,
    IResultLookahead,
};

pub fn raw_string_start(input: Input) -> IResultLookahead<usize> {
    map(
        delimited(
            one_char('r'),
            many0(lookahead(one_char('#'))),
            one_char('"'),
        ),
        |v| v.len(),
    )(input)
}

pub fn raw_string_inner(input: Input) -> IResultLookahead<&str> {
    let ok = lookahead(raw_string_start)(input)?;
    let num = ok.parsed;

    ok.and_then(
        cut(take_until(lookahead(pair(
            one_char('"'),
            repeat_char('#', num),
        )))),
        |_, inner: Input| inner.fragment(),
    )
}

pub fn parse_raw_string(input: Input) -> IResultLookahead<&str> {
    context("raw string", raw_string_inner)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf8_parser::{
        containers::list,
        pt::{Expr, List},
        test_util::eval,
    };

    #[test]
    fn test_raw0() {
        assert_eq!(
            eval!(parse_raw_string, r##"r"Very \ raw string""##),
            r##"Very \ raw string"##
        );
    }

    #[test]
    fn test_raw1() {
        assert_eq!(
            eval!(parse_raw_string, r##"r#"Very \ raw string"#"##),
            r##"Very \ raw string"##
        );
    }

    #[test]
    fn test_raw2() {
        assert_eq!(
            eval!(parse_raw_string, r###"r##"Very \ raw string"##"###),
            r##"Very \ raw string"##
        );
    }

    #[test]
    fn test_raw_remaining() {
        assert_eq!(
            eval!(list, r###"[ r##"Very \ raw string"##, "b"]"###),
            List::new_test(vec![Expr::Str(r##"Very \ raw string"##), Expr::Str("b")])
        );
    }
}
