use std::mem::replace;

#[cfg(feature = "serde1_ast_derives")]
use serde::Serialize;

use crate::location::Location;

/// IMPORTANT: Equality operators do NOT compare the start & end spans!
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
#[cfg_attr(feature = "serde1_ast_derives", serde(transparent))]
pub struct Spanned<T> {
    #[cfg_attr(feature = "serde1_ast_derives", serde(skip))]
    pub start: Location,
    pub value: T,
    #[cfg_attr(feature = "serde1_ast_derives", serde(skip))]
    pub end: Location,
}

/// IMPORTANT: Equality operators do NOT compare the start & end spans!
impl<T> PartialEq for Spanned<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T> Spanned<T> {
    #[cfg(test)]
    pub fn new_test(value: T) -> Self {
        use crate::utf8_parser::test_util::TestMockNew;

        Spanned {
            start: Location::new_mocked(),
            value,
            end: Location::new_mocked(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Ron<'a> {
    pub attributes: Vec<Spanned<Attribute>>,
    pub expr: Spanned<Expr<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Attribute {
    Enable(Spanned<Vec<Spanned<Extension>>>),
}

impl Attribute {
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
    pub fn from_str(input: &'a str) -> Self {
        Ident(input)
    }

    pub fn into_string(self) -> String {
        self.0.to_owned()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    pub fn into_i8(self) -> i8 {
        self.into()
    }
}

impl From<Sign> for i8 {
    fn from(sign: Sign) -> i8 {
        match sign {
            Sign::Positive => 1,
            Sign::Negative => -1,
        }
    }
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

    pub fn into_u64(self) -> u64 {
        self.into()
    }

    pub fn to_expr(self) -> Expr<'static> {
        Expr::Integer(Integer::Unsigned(self))
    }
}

impl From<UnsignedInteger> for u64 {
    fn from(u: UnsignedInteger) -> u64 {
        u.number
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

impl From<SignedInteger> for i64 {
    fn from(s: SignedInteger) -> i64 {
        // TODO: conversion to i64 doesn't work for 2^63 (which, with a negative sign is still in bounds)
        s.sign.into_i8() as i64 * s.number as i64
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

    pub fn into_i64(self) -> i64 {
        match self {
            Integer::Signed(s) => s.into(),
            Integer::Unsigned(u) => u.into_u64() as i64,
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
    pub fractional_digits: u16,
    pub exponent: Option<(Option<Sign>, u16)>,
}

impl Decimal {
    pub fn new(
        sign: Option<Sign>,
        whole: Option<u64>,
        fractional: u64,
        fractional_digits: u16,
        exponent: Option<(Option<Sign>, u16)>,
    ) -> Self {
        Decimal {
            sign,
            whole,
            fractional,
            fractional_digits,
            exponent,
        }
    }
}

impl From<Decimal> for f64 {
    fn from(d: Decimal) -> f64 {
        let sign = d.sign.map(Into::into).unwrap_or(1i8);
        let whole = d.whole.unwrap_or_default();

        let (exp_sign, exp) = d.exponent.unwrap_or((None, 0));

        let exp_sign = exp_sign.map(Into::into).unwrap_or(1i8);
        let exp = exp as i32 * exp_sign as i32;

        let mut f: f64 = sign as f64 * whole as f64;
        f *= (10.0f64).powi(exp);
        f += d.fractional as f64 * (10.0f64).powi(exp - d.fractional_digits as i32) * sign as f64;

        f
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct KeyValue<'a, K: 'a> {
    pub key: Spanned<K>,
    pub value: Spanned<Expr<'a>>,
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

pub type SpannedKvs<'a, K> = Vec<Spanned<KeyValue<'a, K>>>;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Struct<'a> {
    pub fields: SpannedKvs<'a, Ident<'a>>,
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
            entries: kvs
                .into_iter()
                .map(|(k, v)| KeyValue {
                    key: Spanned::new_test(k),
                    value: Spanned::new_test(v),
                })
                .map(Spanned::new_test)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct List<'a> {
    pub elements: Vec<Spanned<Expr<'a>>>,
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
pub struct Tuple<'a> {
    pub elements: Vec<Spanned<Expr<'a>>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Untagged<'a> {
    Unit,
    Struct(Struct<'a>),
    Tuple(Tuple<'a>),
}

impl<'a> Untagged<'a> {
    pub fn take(&mut self) -> Self {
        replace(self, Untagged::Unit)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub struct Tagged<'a> {
    pub ident: Spanned<Ident<'a>>,
    pub untagged: Spanned<Untagged<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde1_ast_derives", derive(Serialize))]
pub enum Expr<'a> {
    Unit,
    Optional(Option<Box<Spanned<Expr<'a>>>>),
    Tagged(Tagged<'a>),
    Bool(bool),
    Tuple(Tuple<'a>),
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
