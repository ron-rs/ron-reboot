use crate::parser;
use crate::parser::{combinators, Input, IResultLookahead};
use crate::parser::ast::{Expr, Ident, KeyValue, List, Map, Spanned, Struct};
use crate::parser::basic::one_char;
use crate::parser::combinators::{comma_list0, comma_list1, context, cut, lookahead, map, opt, pair, terminated};
use crate::parser::primitive::ident;

fn ident_val_pair(input: Input) -> IResultLookahead<KeyValue<Ident>> {
    let pair = pair(
        lookahead(terminated(combinators::spanned(ident::ident), one_char(':'))),
        combinators::spanned(parser::expr),
    );
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

fn opt_ident(input: Input) -> IResultLookahead<Option<Spanned<Ident>>> {
    opt(combinators::spanned(lookahead(ident::ident)))(input)
}

pub fn r#struct(input: Input) -> IResultLookahead<Struct> {
    let untagged_struct = combinators::spanned(combinators::block('(', combinators::ws(comma_list1(ident_val_pair)), ')'));
    // Need to create temp var for borrow checker
    let x = map(
        context("struct", pair(opt_ident, untagged_struct)),
        |(ident, fields)| Struct { fields, ident },
    )(input);

    x
}

fn key_val_pair(input: Input) -> IResultLookahead<KeyValue<Expr>> {
    let pair = pair(terminated(lookahead(combinators::spanned(parser::expr)), cut(one_char(':'))), combinators::spanned(parser::expr));
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

pub fn rmap(input: Input) -> IResultLookahead<Map> {
    map(
        context(
            "map",
            combinators::spanned(combinators::block('{', combinators::ws(comma_list0(key_val_pair)), '}')),
        ),
        |fields| Map { entries: fields },
    )(input)
}

pub fn list(input: Input) -> IResultLookahead<List> {
    context(
        "list",
        combinators::block(
            '[',
            map(combinators::ws(comma_list0(|input| lookahead(parser::expr)(input))), |elements| List { elements }),
            ']',
        ),
    )(input)
}

pub fn tuple(input: Input) -> IResultLookahead<List> {
    context(
        "tuple",
        combinators::block(
            '(',
            map(comma_list0(parser::expr), |elements| List { elements }),
            ')',
        ),
    )(input)
}
