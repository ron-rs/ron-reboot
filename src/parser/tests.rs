#[cfg(test)]
mod tests {
    use crate::parser::ast::Integer;
    use crate::test_util::eval;

    use super::*;

    #[test]
    fn trailing_commas() {
        let input = "Transform(pos: 5,)";
        assert_eq!(
            eval!(r#struct, input),
            Struct::new_test(
                Some("Transform"),
                vec![("pos", UnsignedInteger::new(5).to_expr())]
            )
        );
    }

    #[test]
    fn missing_colon() {
        let input = "Transform(pos 5)";
        assert!(eval!(@result expr, input).is_err());
    }

    #[test]
    fn exprs_struct() {
        let input = "Pos(x:-3,y:4)";
        assert_eq!(Expr::Struct(eval!(r#struct, input)), eval!(expr, input));
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
            Expr::String(eval!(string, r#""\n""#)),
            eval!(expr, r#""\n""#)
        );
        assert_eq!(
            Expr::String(eval!(string, r#""So is /😂\\""#)),
            eval!(expr, r#""So is /😂\\""#)
        );
        assert_eq!(
            Expr::String(eval!(string, r#""\\So is \u{00AC}""#)),
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
            eval!(string, r#""Newlines are\n great!""#),
            "Newlines are\n great!"
        );
        assert_eq!(eval!(string, r#""So is /😂\\""#), "So is /😂\\");
        assert_eq!(eval!(string, r#""So is \u{00AC}""#), "So is \u{00AC}");
    }

    #[test]
    fn exprs_int() {
        for input in ["-4123", "111", "+821"] {
            assert_eq!(eval!(integer, input).to_expr(), eval!(expr, input));
        }
    }

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
            List::new_test(vec![
                UnsignedInteger::new(1).to_expr(),
            ])
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
            eval!(comma_list0(|input| lookahead(expr)(input)), "1,"), vec![Spanned::new_test(UnsignedInteger::new(1).to_expr())]);
    }

    #[test]
    fn maps() {
        let int_n3: Integer = Integer::new_test(Some(Sign::Negative), 3);
        let int_4: Integer = Integer::new_test(None, 4);
        let expr_int_n3: Expr = int_n3.to_expr();
        let expr_int_4: Expr = int_4.to_expr();

        let basic_struct =
            Struct::new_test(Some("Pos"), vec![("x", expr_int_n3), ("y", expr_int_4)]);

        let basic_map = Map::new_test(vec![
            (
                Expr::Str("my map key :)"),
                Expr::Struct(basic_struct.clone()),
            ),
            (Expr::Struct(basic_struct), Expr::Bool(false)),
        ]);

        assert_eq!(
            eval!(
                rmap,
                r#"
{
    "my map key :)": Pos(x: -3, y: 4),
    Pos(x: -3, y: 4): false,
}
"#
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

        let basic_struct = Struct::new_test(None, vec![("x", expr_int_n3), ("y", expr_int_4)]);

        assert_eq!(eval!(r#struct, "(x:-3,y:4)"), basic_struct);
        assert_eq!(eval!(r#struct, "(x:-3,y:4,)"), basic_struct);
        assert_eq!(eval!(r#struct, "(x:-3,y:4,  )"), basic_struct);
        assert_eq!(eval!(r#struct, "(\t  x: -3, y       : 4\n\n)"), basic_struct);
    }

    #[test]
    fn opt_idents() {
        let s = Spanned::new_test;

        assert_eq!(eval!(opt_ident, "Pos"), Some(s(Ident("Pos"))));
        assert_eq!(eval!(opt_ident, "_0"), Some(s(Ident("_0"))));
        assert_eq!(eval!(opt_ident, ""), None);
        assert_eq!(eval!(opt_ident, "!not an ident"), None);
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
        assert_eq!(eval!(r#struct, "Pos(x:-3,y:4,)"), basic_struct);
        assert_eq!(eval!(r#struct, "Pos(x:-3,y:4,  )"), basic_struct);
        assert_eq!(
            eval!(r#struct, "Pos  (\tx: -3, y       : 4\n\n)"),
            basic_struct
        );
    }

    #[test]
    fn excl_mark() {
        let err = eval!(@result r#struct, r#"Example(
    xyz: Asdf(
        x: 4, yalala: !
    ),
)"#).unwrap_err();
        assert_eq!(format!("{}", err), r#"could not match "struct" at 1:1 (`E`) because
could not match "expression" at 2:11 (`A`) because
could not match "struct" at 2:11 (`A`) because
could not match "expression" at 3:24 (`!`) because
    expected one of an ascii letter or '_' at 3:24 (`!`)"#);
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
    fn decimals() {
        assert_eq!(
            eval!(decimal, "-1.0"),
            Decimal::new(Some(Sign::Negative), Some(1), 0, None)
        );
        assert_eq!(
            eval!(decimal, "123.00"),
            Decimal::new(None, Some(123), 0, None)
        );
        assert_eq!(
            eval!(decimal, "+1.23e+2"),
            Decimal::new(
                Some(Sign::Positive),
                Some(1),
                23,
                Some((Some(Sign::Positive), 2))
            )
        );
        assert_eq!(
            eval!(decimal, ".123e3"),
            Decimal::new(None, None, 123, Some((None, 3)))
        );
        assert_eq!(
            eval!(decimal, ".123E-3"),
            Decimal::new(None, None, 123, Some((Some(Sign::Negative), 3)))
        );
    }

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