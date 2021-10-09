use anyhow::anyhow;

use crate::error::PhantomError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ident<'a>(pub &'a str);

impl<'a> Ident<'a> {
    pub fn new(ident: &'a str) -> Result<Self, PhantomError> {
        Ok(Ident(ident))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Integer {
    pub sign: Option<Sign>,
    pub number: u64,
}

impl Integer {
    #[cfg(test)]
    pub fn new_test(sign: Option<Sign>, number: u64) -> Self {
        Integer { sign, number }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Struct<'a> {
    pub ident: Option<Ident<'a>>,
    pub fields: Vec<(Ident<'a>, Integer)>,
}

impl<'a> Struct<'a> {
    pub fn new(
        ident: Option<Ident<'a>>,
        fields: Vec<(Ident<'a>, Integer)>,
    ) -> Result<Self, PhantomError> {
        Ok(Struct { ident, fields })
    }
}
