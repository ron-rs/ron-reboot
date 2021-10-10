#![allow(unused_variables)]

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize, Deserializer};

use crate::ast::Expr::*;
//use crate::error::ErrorKind::{ExpectedBool, ExpectedStrGotEscapes, ExpectedString};
//use crate::error::{ron_err, ErrorKind};
use crate::{ast, parser};

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_str<'a, T>(s: &'a str) -> Result<T, crate::error::Error>
where
    T: Deserialize<'a>,
{
    let mut ron = parser::ron(s)?;

    T::deserialize(RonDeserializer::from_ron(&mut ron))
}

pub struct RonDeserializer<'a, 'de> {
    //ron: ast::Ron<'a>,
    expr: &'a mut ast::Spanned<'de, ast::Expr<'de>>,
}

impl<'a, 'de> RonDeserializer<'a, 'de> {
    /// Create a deserializer from a ron ast
    ///
    /// The ast will be completely replaced with empty exprs,
    /// thus cannot be used anymore.
    pub fn from_ron(ron: &'a mut ast::Ron<'de>) -> Self {
        RonDeserializer {
            expr: &mut ron.expr,
        }
    }

    /*
    fn err<V>(&self, kind: ErrorKind) -> Result<V, crate::error::Error> {
        Err(dbg!(ron_err(kind, self.expr.start, self.expr.end)))
    }
     */
}

impl<'a, 'de> Deserializer<'de> for RonDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.expr.value.take() {
            Unit => visitor.visit_unit(),
            Bool(b) => visitor.visit_bool(b),
            Tuple(mut t) => visitor.visit_seq(SeqDeserializer {
                iter: t.elements.iter_mut(),
            }),
            List(mut l) => visitor.visit_seq(SeqDeserializer {
                iter: l.elements.iter_mut(),
            }),
            Map(_) => todo!(),
            Struct(mut s) => visitor.visit_map(StructDeserializer {
                iter: s.fields.value.iter_mut(),
                value: None,
            }),
            Integer(_) => todo!(),
            Str(s) => visitor.visit_borrowed_str(s),
            String(s) => visitor.visit_string(s),
            Decimal(_) => todo!(),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("identifiers are no expr")
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.expr.value.take();

        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char string str
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum
    }
}

struct SeqDeserializer<'a, 'de> {
    iter: std::slice::IterMut<'a, ast::Spanned<'de, ast::Expr<'de>>>,
}

impl<'a, 'de> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(x) => seed.deserialize(RonDeserializer { expr: x }).map(Some),
            None => Ok(None),
        }
    }
}

struct StructDeserializer<'a, 'de> {
    iter: std::slice::IterMut<'a, ast::Spanned<'de, ast::KeyValue<'de, ast::Ident<'de>>>>,
    value: Option<&'a mut ast::Spanned<'de, ast::Expr<'de>>>,
}

impl<'a, 'de> MapAccess<'de> for StructDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next().map(|s| &mut s.value) {
            Some(x) => {
                self.value = Some(&mut x.value);

                seed.deserialize(IdentDeserializer { ident: &mut x.key })
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(RonDeserializer {
            expr: &mut self
                .value
                .take()
                .expect("called next_value_seed before next_key_seed"),
        })
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        match self.iter.next().map(|s| &mut s.value) {
            Some(x) => {
                let key = kseed.deserialize(IdentDeserializer { ident: &mut x.key })?;
                let value = vseed.deserialize(RonDeserializer { expr: &mut x.value })?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.size_hint().0)
    }
}

struct IdentDeserializer<'a, 'de> {
    ident: &'a mut ast::Spanned<'de, ast::Ident<'de>>,
}

impl<'a, 'de> Deserializer<'de> for IdentDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.ident.value.0)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
