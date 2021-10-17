use crate::{
    utf8_parser,
    utf8_parser::{
        basic::{nothing, one_char},
        combinators,
        combinators::{
            alt2, comma_list0, comma_list1, context, cut, lookahead, map, pair, spanned,
            terminated,
        },
        primitive::ident,
        pt::{Expr, Ident, KeyValue, List, Map, Spanned, Struct, Tagged, Tuple, Untagged},
        IResultLookahead, Input,
    },
};

fn ident_val_pair(input: Input) -> IResultLookahead<KeyValue<Ident>> {
    let pair = pair(
        lookahead(terminated(
            combinators::spanned(ident::ident),
            one_char(':'),
        )),
        combinators::spanned(cut(utf8_parser::expr)),
    );
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

fn untagged_struct_inner(input: Input) -> IResultLookahead<Vec<Spanned<KeyValue<Ident>>>> {
    combinators::block('(', combinators::ws(comma_list1(ident_val_pair)), ')')(input)
}

pub fn untagged_struct(input: Input) -> IResultLookahead<Struct> {
    map(
        context("untagged struct", untagged_struct_inner),
        |fields| Struct { fields },
    )(input)
}

fn key_val_pair(input: Input) -> IResultLookahead<KeyValue<Expr>> {
    let pair = pair(
        terminated(
            lookahead(combinators::spanned(utf8_parser::expr)),
            cut(one_char(':')),
        ),
        combinators::spanned(utf8_parser::expr),
    );
    map(pair, |(k, v)| KeyValue { key: k, value: v })(input)
}

pub fn rmap(input: Input) -> IResultLookahead<Map> {
    map(
        context(
            "map",
            combinators::block('{', combinators::ws(comma_list0(key_val_pair)), '}'),
        ),
        |fields| Map { entries: fields },
    )(input)
}

pub fn list(input: Input) -> IResultLookahead<List> {
    context(
        "list",
        combinators::block(
            '[',
            map(
                combinators::ws(comma_list0(|input| lookahead(utf8_parser::expr)(input))),
                |elements| List { elements },
            ),
            ']',
        ),
    )(input)
}

pub fn tagged(input: Input) -> IResultLookahead<Tagged> {
    context(
        "tagged expr",
        map(
            pair(
                spanned(ident),
                spanned(alt2(
                    map(untagged_struct, Untagged::Struct),
                    alt2(
                        map(tuple, Untagged::Tuple),
                        map(nothing, |_| Untagged::Unit),
                    ),
                )),
            ),
            |(ident, untagged)| Tagged { ident, untagged },
        ),
    )(input)
}

pub fn tuple(input: Input) -> IResultLookahead<Tuple> {
    context(
        "tuple",
        map(
            combinators::block('(', comma_list0(utf8_parser::expr), ')'),
            |elements| Tuple { elements },
        ),
    )(input)
}
