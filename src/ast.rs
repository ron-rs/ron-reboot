use anyhow::anyhow;

use crate::error::PhantomError;
use crate::parser::Input;

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
pub struct Integer {
    pub sign: Option<Sign>,
    pub number: u64,
}

impl Integer {
    #[cfg(test)]
    pub const fn new_test(sign: Option<Sign>, number: u64) -> Self {
        Integer { sign, number }
    }

    #[cfg(test)]
    pub const fn to_expr(self) -> Expr<'static> {
        Expr::Integer(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Struct<'a> {
    pub ident: Option<Ident<'a>>,
    pub fields: Vec<(Ident<'a>, Expr<'a>)>,
}

impl<'a> Struct<'a> {
    pub fn new(
        ident: Option<Ident<'a>>,
        fields: Vec<(Ident<'a>, Expr<'a>)>,
    ) -> Result<Self, PhantomError> {
        Ok(Struct { ident, fields })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr<'a> {
    Struct(Box<Struct<'a>>),
    Integer(Integer),
}

impl<'a> Expr<'a> {
    pub fn from_struct(s: Struct<'a>) -> Self {
        Expr::Struct(Box::new(s))
    }
}
