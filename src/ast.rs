use anyhow::anyhow;

use crate::error::PhantomError;
use crate::parser::Input;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spanned<'a, T>
where
    T: 'a,
{
    pub start: Input<'a>,
    pub value: T,
    pub end: Input<'a>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ident<'a>(pub &'a str);

impl<'a> Ident<'a> {
    pub fn from_input(input: Input<'a>) -> Result<Self, PhantomError> {
        Ok(Ident(input.fragment()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    pub fn from_char(sign: char) -> anyhow::Result<Self> {
        match sign {
            '+' => Ok(Sign::Positive),
            '-' => Ok(Sign::Negative),
            _ => Err(anyhow!("Expected '+' or '-', got {}", sign)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnsignedInteger {
    pub number: u64,
}

impl UnsignedInteger {
    #[cfg(test)]
    pub const fn new_test(sign: Option<Sign>, number: u64) -> Self {
        UnsignedInteger { number }
    }

    #[cfg(test)]
    pub const fn to_expr(self) -> Expr<'static> {
        Expr::Integer(Integer::new_test(None, self))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Integer<'a> {
    pub sign: Option<Spanned<'a, Sign>>,
    pub number: Spanned<'a, UnsignedInteger>,
}

impl Integer<'_> {
    #[cfg(test)]
    pub fn new_test(sign: Option<Sign>, number: UnsignedInteger) -> Self {
        Integer {
            sign: sign.map(Spanned::new_test),
            number: Spanned::new_test(number),
        }
    }

    #[cfg(test)]
    pub fn to_expr(self) -> Expr<'static> {
        Expr::Integer(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyValue<'a, K: 'a> {
    pub key: Spanned<'a, K>,
    pub value: Spanned<'a, Expr<'a>>,
}

pub type SpannedKvs<'a, K> = Spanned<'a, Vec<Spanned<'a, KeyValue<'a, K>>>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Struct<'a> {
    pub ident: Option<Spanned<'a, Ident<'a>>>,
    pub fields: SpannedKvs<'a, Ident<'a>>,
}

impl<'a> Struct<'a> {
    pub fn new(
        ident: Option<Spanned<'a, Ident<'a>>>,
        fields: Spanned<'a, Vec<Spanned<'a, KeyValue<'a, Ident<'a>>>>>,
    ) -> Result<Self, PhantomError> {
        Ok(Struct { ident, fields })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr<'a> {
    Struct(Box<Struct<'a>>),
    Integer(Integer<'a>),
}

impl<'a> Expr<'a> {
    pub fn from_struct(s: Struct<'a>) -> Self {
        Expr::Struct(Box::new(s))
    }
}
