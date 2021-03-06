use crate::utf8_parser::{
    basic::one_char,
    combinators::{comma_list0, lookahead, many0, preceded},
    containers::tagged,
    pt::{Expr, Integer, List, Map, Sign, Spanned, Struct, UnsignedInteger},
    test_util::eval,
    *,
};

#[test]
fn trailing_commas() {
    let input = "Transform(pos: 5,)";
    assert_eq!(
        eval!(tagged, input),
        Struct::new_tagged(
            "Transform",
            vec![("pos", UnsignedInteger::new(5).to_expr())]
        )
    );
}

#[test]
fn missing_colon() {
    let input = "Transform(pos 5)";
    assert_eq!(eval!(@result expr, input).unwrap().remaining.len(), 7);
}

#[test]
fn exprs_struct() {
    let input = "Pos(x:-3,y:4)";
    assert_eq!(Expr::Tagged(eval!(tagged, input)), eval!(expr, input));
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
        Expr::String(eval!(escaped_string, r#""\n""#)),
        eval!(expr, r#""\n""#)
    );
    assert_eq!(
        Expr::String(eval!(escaped_string, r#""So is /😂\\""#)),
        eval!(expr, r#""So is /😂\\""#)
    );
    assert_eq!(
        Expr::String(eval!(escaped_string, r#""\\So is \u{00AC}""#)),
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
        eval!(escaped_string, r#""Newlines are\n great!""#),
        "Newlines are\n great!"
    );
    assert_eq!(eval!(escaped_string, r#""So is /😂\\""#), "So is /😂\\");
    assert_eq!(
        eval!(escaped_string, r#""So is \u{00AC}""#),
        "So is \u{00AC}"
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
        List::new_test(vec![UnsignedInteger::new(1).to_expr(),])
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
        eval!(comma_list0(|input| lookahead(expr)(input)), "1,"),
        vec![Spanned::new_test(UnsignedInteger::new(1).to_expr())]
    );
}

#[test]
fn many0_empty() {
    assert_eq!(
        eval!(many0(preceded(lookahead(one_char('a')), one_char('b'))), ""),
        vec![]
    );
}

#[test]
fn comma_list0_empty() {
    assert_eq!(eval!(comma_list0(|input| expr(input)), ""), vec![]);
}

#[test]
fn expr_empty_recoverable() {
    assert!(eval!(@result expr, "").unwrap_err().is_recoverable());
}

#[test]
fn maps() {
    let int_n3: Integer = Integer::new_test(Some(Sign::Negative), 3);
    let int_4: Integer = Integer::new_test(None, 4);
    let expr_int_n3: Expr = int_n3.to_expr();
    let expr_int_4: Expr = int_4.to_expr();

    let basic_struct = Struct::new_tagged("Pos", vec![("x", expr_int_n3), ("y", expr_int_4)]);

    let basic_map = Map::new_test(vec![
        (
            Expr::Str("my map key :)"),
            Expr::Tagged(basic_struct.clone()),
        ),
        (Expr::Tagged(basic_struct), Expr::Bool(false)),
    ]);

    assert_eq!(
        eval!(
            rmap,
            r#"{
    "my map key :)": Pos(x: -3, y: 4),
    Pos(x: -3, y: 4): false,
}"#
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

    let basic_struct = Struct::new_test(vec![("x", expr_int_n3), ("y", expr_int_4)]);

    assert_eq!(eval!(untagged_struct, "(x:-3,y:4)"), basic_struct);
    assert_eq!(eval!(untagged_struct, "(x:-3,y:4,)"), basic_struct);
    assert_eq!(eval!(untagged_struct, "(x:-3,y:4,  )"), basic_struct);
    assert_eq!(
        eval!(untagged_struct, "(\t  x: -3, y       : 4\n\n)"),
        basic_struct
    );
}

#[test]
fn structs() {
    let int_n3: Integer = Integer::new_test(Some(Sign::Negative), 3);
    let int_4: Integer = Integer::new_test(None, 4);
    let expr_int_n3: Expr = int_n3.to_expr();
    let expr_int_4: Expr = int_4.to_expr();

    let basic_struct = Struct::new_tagged("Pos", vec![("x", expr_int_n3), ("y", expr_int_4)]);

    assert_eq!(eval!(tagged, "Pos(x:-3,y:4)"), basic_struct);
    assert_eq!(eval!(tagged, "Pos(x:-3,y:4,)"), basic_struct);
    assert_eq!(eval!(tagged, "Pos(x:-3,y:4,  )"), basic_struct);
    assert_eq!(
        eval!(tagged, "Pos  (\tx: -3, y       : 4\n\n)"),
        basic_struct
    );
}

#[test]
fn excl_mark() {
    let err = eval!(@result tagged, r#"Example(
    xyz: Asdf(
        x: 4, yalala: !
    ),
)"#)
    .unwrap_err();
    assert_eq!(
        format!("{}", err),
        r#"could not match "expression" at 3:23 (`!`) because
    expected one of an ascii letter or '_' at 3:23 (`!`)"#
    );
}
