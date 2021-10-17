use crate::utf8_parser::{
    basic::{one_char, one_of_chars, one_of_tags, tag},
    bool,
    char_categories::is_ident_first_char,
    combinators,
    combinators::{
        alt2, comma_list1, context, context_final, cut, delimited, lookahead, many0, map, pair,
        preceded, take1_if,
    },
    containers::tagged,
    decimal, escaped_string, list,
    pt::{Attribute, Expr, Extension, Ron, SignedInteger, UnsignedInteger},
    rmap, signed_integer, tuple, unescaped_str, unsigned_integer, untagged_struct, ErrorTree,
    Expectation, IResultLookahead, Input, InputParseErr, InputParseError,
};

fn extension_name(input: Input) -> IResultLookahead<Extension> {
    one_of_tags(
        &["unwrap_newtypes", "implicit_some"],
        &[Extension::UnwrapNewtypes, Extension::ImplicitSome],
    )(input)
}

fn attribute_enable(input: Input) -> IResultLookahead<Attribute> {
    let start = preceded(tag("enable"), combinators::ws(one_char('(')));
    let end = one_char(')');

    delimited(
        start,
        map(
            combinators::spanned(comma_list1(extension_name)),
            Attribute::Enable,
        ),
        end,
    )(input)
}

fn attribute(input: Input) -> IResultLookahead<Attribute> {
    let start = preceded(
        preceded(lookahead(one_char('#')), combinators::ws(one_char('!'))),
        combinators::ws(one_char('[')),
    );
    let end = one_char(']');

    context(
        "attribute",
        delimited(start, combinators::ws(attribute_enable), end),
    )(input)
}

#[derive(Clone, Debug)]
enum ExprClass {
    StructTuple,
    Map,
    StrString,
    List,
    Bool,
    /// Signed or Decimal
    SignedDec,
    Dec,
    /// Unsigned or Decimal
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
                ExprClass::SignedDec,
                ExprClass::SignedDec,
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
    let expr_class = ExprClass::parse(input)?.parsed;

    // We could just directly try parsing all of these variants without determining an expr class
    // beforehand. However, for error collection & possibly performance reasons this seems to be
    // the better solution right now.
    match expr_class {
        ExprClass::StructTuple => cut(alt2(
            map(untagged_struct, Expr::Struct),
            map(tuple, Expr::Tuple),
        ))(input),
        ExprClass::Map => map(rmap, Expr::Map)(input),
        ExprClass::StrString => alt2(
            map(lookahead(unescaped_str), Expr::Str),
            map(escaped_string, Expr::String),
        )(input),
        ExprClass::List => map(list, Expr::List)(input),
        ExprClass::Bool => map(bool, Expr::Bool)(input),
        ExprClass::SignedDec => alt2(
            map(decimal, Expr::Decimal),
            map(signed_integer, SignedInteger::to_expr),
        )(input),
        ExprClass::Dec => map(decimal, Expr::Decimal)(input),
        ExprClass::UnsignedDec => alt2(
            map(decimal, Expr::Decimal),
            map(unsigned_integer, UnsignedInteger::to_expr),
        )(input),
        ExprClass::LeadingIdent => map(tagged, Expr::Tagged)(input),
    }
}

pub fn expr(input: Input) -> IResultLookahead<Expr> {
    cut(context_final("expression", true, expr_inner))(input)
}

fn ron_inner(input: Input) -> IResultLookahead<Ron> {
    map(
        pair(
            many0(combinators::spanned(attribute)),
            combinators::spanned(expr),
        ),
        |(attributes, expr)| Ron { attributes, expr },
    )(input)
}

pub fn ron(input: &str) -> Result<Ron, InputParseError> {
    let input = Input::new(input);

    match ron_inner(input) {
        Ok(ok) if ok.remaining.is_empty() => Ok(ok.parsed),
        Ok(ok) => Err(ErrorTree::expected(ok.remaining, Expectation::Eof)),
        Err(InputParseErr::Fatal(e)) | Err(InputParseErr::Recoverable(e)) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf8_parser::test_util::eval;

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
}
