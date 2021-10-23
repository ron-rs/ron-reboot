//! Parse tree

use std::mem::replace;

pub use crate::ast::Extension;
use crate::{ast, utf8_parser::input::Input};

/// IMPORTANT: Equality operators do NOT compare the start & end spans!
#[derive(Clone, Debug)]
pub struct Spanned<'a, T> {
    pub start: Input<'a>,
    pub value: T,
    pub end: Input<'a>,
}

/// IMPORTANT: Equality operators do NOT compare the start & end spans!
impl<T> PartialEq for Spanned<'_, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<'a, T> Spanned<'a, T> {
    #[cfg(test)]
    pub fn new_test(value: T) -> Self {
        use crate::utf8_parser::test_util::TestMockNew;

        Spanned {
            start: Input::new_mocked(),
            value,
            end: Input::new_mocked(),
        }
    }

    pub fn map<T2>(self, f: impl FnOnce(T) -> T2) -> Spanned<'a, T2> {
        Spanned {
            start: self.start,
            end: self.end,
            value: f(self.value),
        }
    }
}

impl<'a, T, T2> From<Spanned<'a, T>> for ast::Spanned<T2>
where
    T: Into<T2>,
{
    fn from(s: Spanned<'a, T>) -> Self {
        ast::Spanned {
            start: s.start.into(),
            value: s.value.into(),
            end: s.end.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ron<'a> {
    pub attributes: Vec<Spanned<'a, Attribute<'a>>>,
    pub expr: Spanned<'a, Expr<'a>>,
}

impl<'a> From<Ron<'a>> for ast::Ron<'a> {
    fn from(r: Ron<'a>) -> Self {
        ast::Ron {
            attributes: r.attributes.into_iter().map(Into::into).collect(),
            expr: r.expr.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute<'a> {
    Enable(Spanned<'a, Vec<Spanned<'a, Extension>>>),
}

impl Attribute<'_> {
    #[cfg(test)]
    pub fn enables_test(extensions: Vec<Extension>) -> Self {
        Attribute::Enable(Spanned::new_test(
            extensions.into_iter().map(Spanned::new_test).collect(),
        ))
    }
}

impl<'a> From<Attribute<'a>> for ast::Attribute {
    fn from(a: Attribute<'a>) -> Self {
        match a {
            Attribute::Enable(e) => ast::Attribute::Enable(
                e.map(|v| v.into_iter().map(Into::into).collect::<Vec<_>>())
                    .into(),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ident<'a>(pub &'a str);

impl<'a> Ident<'a> {
    pub fn from_str(input: &'a str) -> Self {
        Ident(input)
    }
}

impl<'a> From<Ident<'a>> for ast::Ident<'a> {
    fn from(i: Ident<'a>) -> Self {
        ast::Ident(i.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl From<Sign> for ast::Sign {
    fn from(s: Sign) -> Self {
        match s {
            Sign::Positive => ast::Sign::Positive,
            Sign::Negative => ast::Sign::Negative,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl From<UnsignedInteger> for ast::UnsignedInteger {
    fn from(s: UnsignedInteger) -> Self {
        ast::UnsignedInteger { number: s.number }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl From<SignedInteger> for ast::SignedInteger {
    fn from(s: SignedInteger) -> Self {
        ast::SignedInteger {
            sign: s.sign.into(),
            number: s.number,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl From<Integer> for ast::Integer {
    fn from(i: Integer) -> Self {
        match i {
            Integer::Signed(s) => ast::Integer::Signed(s.into()),
            Integer::Unsigned(u) => ast::Integer::Unsigned(u.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl From<Decimal> for ast::Decimal {
    fn from(d: Decimal) -> ast::Decimal {
        ast::Decimal {
            sign: d.sign.map(Into::into),
            whole: d.whole,
            fractional: d.fractional,
            fractional_digits: d.fractional_digits,
            exponent: d.exponent.map(|(s, e)| (s.map(Into::into), e)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl<'a, K, K2> From<KeyValue<'a, K>> for ast::KeyValue<'a, K2>
where
    K: Into<K2>,
{
    fn from(m: KeyValue<'a, K>) -> Self {
        ast::KeyValue {
            key: m.key.into(),
            value: m.value.into(),
        }
    }
}

pub type SpannedKvs<'a, K> = Vec<Spanned<'a, KeyValue<'a, K>>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Struct<'a> {
    pub fields: SpannedKvs<'a, Ident<'a>>,
}

impl<'a> Struct<'a> {
    #[cfg(test)]
    pub fn new_test(fields: Vec<(&'a str, Expr<'a>)>) -> Self {
        Struct {
            fields: fields
                .into_iter()
                .map(|field| Spanned::new_test(KeyValue::new_test(Ident(field.0), field.1)))
                .collect(),
        }
    }

    #[cfg(test)]
    pub fn new_tagged(ident: &'a str, fields: Vec<(&'a str, Expr<'a>)>) -> Tagged<'a> {
        Tagged {
            ident: Spanned::new_test(Ident(ident)),
            untagged: Spanned::new_test(Untagged::Struct(Struct::new_test(fields))),
        }
    }
}

impl<'a> From<Struct<'a>> for ast::Struct<'a> {
    fn from(m: Struct<'a>) -> Self {
        ast::Struct {
            fields: m.fields.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl<'a> From<Map<'a>> for ast::Map<'a> {
    fn from(m: Map<'a>) -> Self {
        ast::Map {
            entries: m.entries.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl<'a> From<List<'a>> for ast::List<'a> {
    fn from(l: List<'a>) -> Self {
        ast::List {
            elements: l.elements.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tuple<'a> {
    pub elements: Vec<Spanned<'a, Expr<'a>>>,
}

impl<'a> Tuple<'a> {
    #[cfg(test)]
    pub fn new_test(kvs: Vec<Expr<'a>>) -> Self {
        Tuple {
            elements: kvs.into_iter().map(Spanned::new_test).collect(),
        }
    }
}

impl<'a> From<Tuple<'a>> for ast::Tuple<'a> {
    fn from(l: Tuple<'a>) -> Self {
        ast::Tuple {
            elements: l.elements.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Untagged<'a> {
    Unit,
    Struct(Struct<'a>),
    Tuple(Tuple<'a>),
}

impl<'a> From<Untagged<'a>> for ast::Untagged<'a> {
    fn from(u: Untagged<'a>) -> Self {
        match u {
            Untagged::Unit => ast::Untagged::Unit,
            Untagged::Struct(s) => ast::Untagged::Struct(s.into()),
            Untagged::Tuple(t) => ast::Untagged::Tuple(t.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tagged<'a> {
    pub ident: Spanned<'a, Ident<'a>>,
    pub untagged: Spanned<'a, Untagged<'a>>,
}

impl<'a> Tagged<'a> {
    pub fn is_optional(&self) -> bool {
        match (self.ident.value.0, &self.untagged.value) {
            ("Some", Untagged::Tuple(Tuple { elements })) if elements.len() == 1 => {
                true
            }
            ("None", Untagged::Unit) => true,
            _ => false,
        }
    }

    pub fn into_optional(self) -> Option<Spanned<'a, Expr<'a>>> {
        match (self.ident.value.0, self.untagged.value) {
            ("Some", Untagged::Tuple(Tuple { mut elements })) if elements.len() == 1 => {
                Some(elements.remove(0))
            }
            ("None", Untagged::Unit) => None,
            _ => unimplemented!(),
        }
    }
}

impl<'a> From<Tagged<'a>> for ast::Tagged<'a> {
    fn from(t: Tagged<'a>) -> Self {
        ast::Tagged {
            ident: t.ident.into(),
            untagged: t.untagged.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'a> {
    Unit,
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

impl<'a> From<Expr<'a>> for ast::Expr<'a> {
    fn from(e: Expr<'a>) -> Self {
        match e {
            Expr::Unit => ast::Expr::Unit,
            Expr::Tagged(t) if t.is_optional() => ast::Expr::Optional(t.into_optional().map(|e| Box::new(e.into()))),
            Expr::Tagged(t) => ast::Expr::Tagged(t.into()),
            Expr::Bool(x) => ast::Expr::Bool(x),
            Expr::Tuple(x) => ast::Expr::Tuple(x.into()),
            Expr::List(x) => ast::Expr::List(x.into()),
            Expr::Map(x) => ast::Expr::Map(x.into()),
            Expr::Struct(x) => ast::Expr::Struct(x.into()),
            Expr::Integer(x) => ast::Expr::Integer(x.into()),
            Expr::Str(x) => ast::Expr::Str(x.into()),
            Expr::String(x) => ast::Expr::String(x.into()),
            Expr::Decimal(x) => ast::Expr::Decimal(x.into()),
        }
    }
}

trait FromExt<T> {
    fn from_ext(t: T) -> T;
}

/*
impl<T, T2> FromExt<T> for T2
where
    T: Into<T2>
{
    fn from_ext(t: T) -> T {
        t.into()
    }
}
 */

impl<'a, T, T2> FromExt<Vec<T>> for Vec<T2>
where
    T2: FromExt<T>,
{
    fn from_ext(v: Vec<T>) -> Vec<T> {
        v.into_iter().map(T2::from_ext).collect()
    }
}
