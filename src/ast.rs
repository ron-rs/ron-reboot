use std::mem::replace;

#[cfg(feature = "serde1_ast_derives")]
use serde::Serialize;

use crate::parser::Input;

/// IMPORTANT: Equality operators do NOT compare the start & end spans!
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
#[cfg_attr(feature = "serde1_ast_derives", serde(transparent))]
pub struct Spanned<'a, T>
where
    T: 'a,
{
    #[cfg_attr(feature = "serde1_ast_derives", serde(skip))]
    pub start: Input<'a>,
    pub value: T,
    #[cfg_attr(feature = "serde1_ast_derives", serde(skip))]
    pub end: Input<'a>,
}

/// IMPORTANT: Equality operators do NOT compare the start & end spans!
impl<'a, T> PartialEq for Spanned<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<'a, T> Spanned<'a, T>
where
    T: 'a,
{
    #[cfg(test)]
    pub fn new_test(value: T) -> Self {
        Spanned {
            start: Input::new(""),
            value,
            end: Input::new(""),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Ron<'a> {
    pub attributes: Vec<Spanned<'a, Attribute<'a>>>,
    pub expr: Spanned<'a, Expr<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Attribute<'a> {
    Enable(Spanned<'a, Vec<Spanned<'a, Extension>>>),
}

impl<'a> Attribute<'a> {
    #[cfg(test)]
    pub fn enables_test(extensions: Vec<Extension>) -> Self {
        Attribute::Enable(Spanned::new_test(
            extensions.into_iter().map(Spanned::new_test).collect(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Extension {
    UnwrapNewtypes,
    ImplicitSome,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Ident<'a>(pub &'a str);

impl<'a> Ident<'a> {
    pub fn from_input(input: Input<'a>) -> Self {
        Ident(input.fragment())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct UnsignedInteger {
    pub number: u64,
}

impl UnsignedInteger {
    #[cfg(test)]
    pub const fn new(number: u64) -> Self {
        UnsignedInteger { number }
    }

    pub fn to_expr(self) -> Expr<'static> {
        Expr::Integer(Integer::Unsigned(self))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct SignedInteger {
    pub sign: Sign,
    pub number: u64,
}

impl SignedInteger {
    #[cfg(test)]
    pub fn new_test(sign: Sign, number: u64) -> Self {
        SignedInteger { sign, number }
    }

    pub fn to_expr(self) -> Expr<'static> {
        Expr::Integer(Integer::Signed(self))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Integer {
    Signed(SignedInteger),
    Unsigned(UnsignedInteger),
}

impl Integer {
    #[cfg(test)]
    pub fn new_test(sign: Option<Sign>, number: u64) -> Self {
        match sign {
            None => Integer::Unsigned(UnsignedInteger::new(number)),
            Some(sign) => Integer::Signed(SignedInteger::new_test(sign, number)),
        }
    }

    #[cfg(test)]
    pub fn to_expr(self) -> Expr<'static> {
        Expr::Integer(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Decimal {
    pub sign: Option<Sign>,
    pub whole: Option<u64>,
    pub fractional: u64,
    pub exponent: Option<(Option<Sign>, u16)>,
}

impl Decimal {
    pub fn new(
        sign: Option<Sign>,
        whole: Option<u64>,
        fractional: u64,
        exponent: Option<(Option<Sign>, u16)>,
    ) -> Self {
        Decimal {
            sign,
            whole,
            fractional,
            exponent,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct KeyValue<'a, K: 'a> {
    pub key: Spanned<'a, K>,
    pub value: Spanned<'a, Expr<'a>>,
}

impl<'a, K: 'a> KeyValue<'a, K> {
    #[cfg(test)]
    pub fn new_test(key: K, value: Expr<'a>) -> Self {
        KeyValue {
            key: Spanned::new_test(key),
            value: Spanned::new_test(value),
        }
    }
}

pub type SpannedKvs<'a, K> = Spanned<'a, Vec<Spanned<'a, KeyValue<'a, K>>>>;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Struct<'a> {
    pub ident: Option<Spanned<'a, Ident<'a>>>,
    pub fields: SpannedKvs<'a, Ident<'a>>,
}

impl<'a> Struct<'a> {
    #[cfg(test)]
    pub fn new(
        ident: Option<Spanned<'a, Ident<'a>>>,
        fields: Spanned<'a, Vec<Spanned<'a, KeyValue<'a, Ident<'a>>>>>,
    ) -> Self {
        Struct { ident, fields }
    }

    #[cfg(test)]
    pub fn new_test(ident: Option<&'a str>, fields: Vec<(&'a str, Expr<'a>)>) -> Self {
        Struct::new(
            ident.map(Ident).map(Spanned::new_test),
            Spanned::new_test(
                fields
                    .into_iter()
                    .map(|field| Spanned::new_test(KeyValue::new_test(Ident(field.0), field.1)))
                    .collect(),
            ),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Map<'a> {
    pub entries: SpannedKvs<'a, Expr<'a>>,
}

impl<'a> Map<'a> {
    #[cfg(test)]
    pub fn new_test(kvs: Vec<(Expr<'a>, Expr<'a>)>) -> Self {
        Map {
            entries: Spanned::new_test(
                kvs.into_iter()
                    .map(|(k, v)| KeyValue {
                        key: Spanned::new_test(k),
                        value: Spanned::new_test(v),
                    })
                    .map(Spanned::new_test)
                    .collect(),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct List<'a> {
    pub elements: Vec<Spanned<'a, Expr<'a>>>,
}

impl<'a> List<'a> {
    #[cfg(test)]
    pub fn new_test(kvs: Vec<Expr<'a>>) -> Self {
        List {
            elements: kvs.into_iter().map(Spanned::new_test).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Expr<'a> {
    Unit,
    Bool(bool),
    Tuple(List<'a>),
    List(List<'a>),
    Map(Map<'a>),
    Struct(Struct<'a>),
    Integer(Integer),
    /// String without escapes (zero-copy)
    Str(&'a str),
    /// Escaped string
    String(String),
    Decimal(Decimal),
}

impl<'a> Expr<'a> {
    /// Replace expr with Unit, returning ownership of the contained expr
    pub fn take(&mut self) -> Self {
        replace(self, Expr::Unit)
    }
}
