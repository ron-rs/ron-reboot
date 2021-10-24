//! Implements AST -> Value

use crate::{
    ast,
    ast::{Expr, Untagged},
    value::{Float, Number},
    Value,
};

impl Value {
    pub fn from_ast(ast: ast::Ron) -> Self {
        ast.into()
    }
}

#[cfg(feature = "utf8_parser")]
impl std::str::FromStr for Value {
    type Err = crate::Error;

    /// Creates a value from a string reference.
    fn from_str(s: &str) -> Result<Self, crate::Error> {
        crate::utf8_parser::ast_from_str(s).map(Into::into)
    }
}

impl<'a> From<ast::Ron<'a>> for Value {
    fn from(e: ast::Ron) -> Self {
        e.expr.value.into()
    }
}

impl<'a> From<ast::Expr<'a>> for Value {
    fn from(e: ast::Expr) -> Self {
        match e {
            Expr::Unit => Value::Unit(None),
            Expr::Optional(o) => Value::Option(o.map(|s| s.value.into()).map(Box::new)),
            Expr::Tagged(ast::Tagged { ident, untagged }) => match untagged.value {
                Untagged::Unit => Value::Unit(Some(ident.value.0.to_owned())),
                Untagged::Struct(s) => Value::Struct(
                    Some(ident.value.0.to_owned()),
                    s.fields
                        .into_iter()
                        .map(|s| (s.value.key.value, s.value.value.value))
                        .map(|(k, v)| (k.into_string(), v.into()))
                        .collect(),
                ),
                Untagged::Tuple(t) => Value::Tuple(
                    Some(ident.value.0.to_owned()),
                    t.elements.into_iter().map(Into::into).collect(),
                ),
            },
            Expr::Bool(b) => Value::Bool(b),
            Expr::Tuple(t) => Value::Tuple(None, t.elements.into_iter().map(Into::into).collect()),
            Expr::List(l) => Value::List(l.elements.into_iter().map(Into::into).collect()),
            Expr::Map(m) => Value::Map(
                m.entries
                    .into_iter()
                    .map(|s| (s.value.key.value, s.value.value.value))
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            ),
            Expr::Struct(s) => Value::Struct(
                None,
                s.fields
                    .into_iter()
                    .map(|s| (s.value.key.value, s.value.value.value))
                    .map(|(k, v)| (k.into_string(), v.into()))
                    .collect(),
            ),
            Expr::Integer(i) => Value::Number(Number::Integer(i.into_i64())),
            Expr::Str(s) => Value::String(s.to_owned()),
            Expr::String(s) => Value::String(s),
            Expr::Decimal(d) => Value::Number(Number::Float(Float::new(d.into()))),
        }
    }
}

impl<'a> From<ast::Spanned<ast::Expr<'a>>> for Value {
    fn from(e: ast::Spanned<ast::Expr<'a>>) -> Self {
        e.value.into()
    }
}
