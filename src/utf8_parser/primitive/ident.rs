use crate::utf8_parser::{
    ast::Ident,
    char_categories::{is_ident_first_char, is_ident_other_char},
    combinators::{context, map, preceded, recognize, take1_if, take_while},
    Expectation, IResultLookahead, Input,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::eval;

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
