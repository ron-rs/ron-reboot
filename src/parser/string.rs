use crate::parser::{
    util::{
        alt2, base_err_res, context, cut, delimited, fold_many0, map, map_res, multispace1,
        one_char, one_of_chars, preceded, take_while, take_while_m_n,
    },
    BaseErrorKind, ErrorTree, Expectation, IResult, Input, InputParseErr
};

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals. We will combine this later with parse_escaped_char
/// to parse sequences like \u{00AC}.
fn parse_unicode(input: Input) -> IResult<char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit(), Expectation::HexDigit);

    dbg!(input);

    let parse_delimited_hex = preceded(
        one_char('u'),
        cut(delimited(one_char('{'), parse_hex, one_char('}'))),
    );

    map_res(parse_delimited_hex, move |hex: Input| {
        let parsed_u32 = u32::from_str_radix(hex.fragment(), 16).map_err(|e| {
            InputParseErr::Error(ErrorTree::Base {
                location: input,
                kind: BaseErrorKind::External(Box::new(e)),
            })
        })?;

        std::char::from_u32(parsed_u32).ok_or(InputParseErr::Error(ErrorTree::expected(
            input,
            Expectation::UnicodeHexSequence { got: parsed_u32 },
        )))
    })(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char(input: Input) -> IResult<char> {
    preceded(
        one_char('\\'),
        cut(alt2(
            parse_unicode,
            one_of_chars(
                "nrtbf\\/\"",
                &['\n', '\r', '\t', '\u{08}', '\u{0C}', '\\', '/', '"'],
            ),
        )),
    )(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a>(input: Input<'a>) -> IResult<Input<'a>> {
    preceded(one_char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a>(input: Input<'a>) -> IResult<Input<'a>> {
    // `is_not` parses a string of 0 or more characters that aren't one of the
    // given characters.
    let not_quote_slash = take_while(|c| c != '"' && c != '\\');

    // `verify` runs a parser, then runs a verification function on the output of
    // the parser. The verification function accepts out output only if it
    // returns true. In this case, we want to ensure that the output of is_not
    // is non-empty.
    map_res(not_quote_slash, |s| {
        if !s.fragment().is_empty() {
            Ok(s)
        } else {
            base_err_res(s, Expectation::Something)
        }
    })(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a>(input: Input<'a>) -> IResult<StringFragment<'a>> {
    alt2(
        // The `map` combinator runs a parser, then applies a function to the output
        // of that parser.
        map(parse_literal, |i| StringFragment::Literal(i.fragment())),
        alt2(
            map(parse_escaped_char, StringFragment::EscapedChar),
            map(parse_escaped_whitespace, |_| StringFragment::EscapedWS),
        ),
    )(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
pub fn parse_string(input: Input) -> IResult<String> {
    // fold_many0 is the equivalent of iterator::fold. It runs a parser in a loop,
    // and for each output value, calls a folding function on each output value.
    let build_string = fold_many0(
        // Our parser function– parses a single string fragment
        parse_fragment,
        // Our init value, an empty string
        String::new,
        // Our folding function. For each fragment, append the fragment to the
        // string.
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    // Finally, parse the string. Note that, if `build_string` could accept a raw
    // " character, the closing delimiter " would never match. When using
    // `delimited` with a looping parser (like fold_many0), be sure that the
    // loop won't accidentally match your closing delimiter!
    context(
        "string",
        delimited(one_char('"'), build_string, one_char('"')),
    )(input)
}
