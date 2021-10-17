#![allow(clippy::type_complexity)]

use serde::{
    de::{
        DeserializeSeed, EnumAccess, Error as SerdeErrorTrait, MapAccess, SeqAccess, VariantAccess,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize, Deserializer,
};

//use crate::error::ErrorKind::{ExpectedBool, ExpectedStrGotEscapes, ExpectedString};
//use crate::error::{ron_err, ErrorKind};
use crate::{
    ast::Untagged,
    error::Error,
    utf8_parser::{
        ast,
        ast::{Expr::*, Integer},
        ast_from_str,
    },
};

pub fn from_str<'a, T>(s: &'a str) -> Result<T, crate::error::Error>
where
    T: Deserialize<'a>,
{
    let mut ron = ast_from_str(s)
        .map_err(Error::from)
        .map_err(|e| e.context_file_content(s.to_owned()))?;

    T::deserialize(RonDeserializer::from_ron(&mut ron))
        .map_err(|e| e.context_file_content(s.to_owned()))
}

pub struct RonDeserializer<'a, 'de> {
    //ron: ast::Ron<'a>,
    expr: &'a mut ast::Spanned<ast::Expr<'de>>,
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
        let res = match self.expr.value.take() {
            Unit => visitor.visit_unit(),
            Bool(b) => visitor.visit_bool(b),
            Tuple(mut t) => visitor.visit_seq(SeqDeserializer {
                iter: t.elements.iter_mut(),
            }),
            List(mut l) => visitor.visit_seq(SeqDeserializer {
                iter: l.elements.iter_mut(),
            }),
            Map(mut m) => visitor.visit_map(MapDeserializer {
                iter: m.entries.iter_mut(),
                value: None,
            }),
            Struct(mut s) => visitor.visit_map(StructDeserializer {
                iter: s.fields.iter_mut(),
                value: None,
            }),
            Integer(i) => match i {
                Integer::Signed(s) => visitor.visit_i64(s.into()),
                Integer::Unsigned(u) => visitor.visit_u64(u.into()),
            },
            Str(s) => visitor.visit_borrowed_str(s),
            String(s) => visitor.visit_string(s),
            Decimal(d) => visitor.visit_f64(d.into()),
            Tagged(t) => match t.untagged.value {
                Untagged::Struct(mut s) => visitor.visit_map(StructDeserializer {
                    iter: s.fields.iter_mut(),
                    value: None,
                }),
                Untagged::Tuple(mut t) => visitor.visit_seq(SeqDeserializer {
                    iter: t.elements.iter_mut(),
                }),
                Untagged::Unit => visitor.visit_unit(),
            },
        };

        res.map_err(|e| e.context_loc(self.expr.start.into(), self.expr.end.into()))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let start_loc = self.expr.start;
        let end_loc = self.expr.end;
        let res = match self.expr.value.take() {
            ast::Expr::Tagged(ast::Tagged { ident, .. }) if ident.value.0 != name => {
                Err(Error::custom(format!(
                    "invalid struct type: `{}`, expected `{}`",
                    ident.value.0, name
                )).context_loc(ident.start, ident.end))
            }
            ast::Expr::Tagged(ast::Tagged {
                untagged:
                    ast::Spanned {
                        value: Untagged::Struct(mut s),
                        ..
                    },
                ..
            })
            | ast::Expr::Struct(mut s) => visitor.visit_map(StructDeserializer {
                iter: s.fields.iter_mut(),
                value: None,
            }),
            _ => self.deserialize_any(visitor),
        };

        res.map_err(|e| e.context_loc(start_loc, end_loc))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let start_loc = self.expr.start;
        let end_loc = self.expr.end;
        let res = match self.expr.value.take() {
            Tagged(mut t) => visitor.visit_enum(EnumDeserializer { tagged: &mut t }),
            // probably no enum and will error
            _ => self.deserialize_any(visitor),
        };

        res.map_err(|e| e.context_loc(start_loc, end_loc))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
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
        tuple_struct map
    }
}

struct SeqDeserializer<'a, 'de> {
    iter: std::slice::IterMut<'a, ast::Spanned<ast::Expr<'de>>>,
}

impl<'a, 'de> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(x) => seed
                .deserialize(RonDeserializer { expr: x })
                .map(Some)
                .map_err(|e| e.context_loc(x.start.into(), x.end.into())),
            None => Ok(None),
        }
    }
}

struct StructDeserializer<'a, 'de> {
    iter: std::slice::IterMut<'a, ast::Spanned<ast::KeyValue<'de, ast::Ident<'de>>>>,
    value: Option<&'a mut ast::Spanned<ast::Expr<'de>>>,
}

impl<'a, 'de> MapAccess<'de> for StructDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(x) => {
                let start_loc = x.start;
                let end_loc = x.end;
                self.value = Some(&mut x.value.value);

                seed.deserialize(IdentDeserializer {
                    ident: &mut x.value.key,
                })
                .map(Some)
                .map_err(|e| e.context_loc(start_loc, end_loc))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let x = self
            .value
            .take()
            .expect("called next_value_seed before next_key_seed");
        seed.deserialize(RonDeserializer { expr: x })
            .map_err(|e| e.context_loc(x.start.into(), x.end.into()))
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
        match self.iter.next() {
            Some(x) => {
                let key = kseed
                    .deserialize(IdentDeserializer {
                        ident: &mut x.value.key,
                    })
                    .map_err(|e| e.context_loc(x.start.into(), x.end.into()))?;
                let value = vseed.deserialize(RonDeserializer {
                    expr: &mut x.value.value,
                })?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.size_hint().0)
    }
}

struct MapDeserializer<'a, 'de> {
    iter: std::slice::IterMut<'a, ast::Spanned<ast::KeyValue<'de, ast::Expr<'de>>>>,
    value: Option<&'a mut ast::Spanned<ast::Expr<'de>>>,
}

impl<'a, 'de> MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(x) => {
                let start_loc = x.start;
                let end_loc = x.end;

                self.value = Some(&mut x.value.value);

                seed.deserialize(RonDeserializer {
                    expr: &mut x.value.key,
                })
                .map(Some)
                .map_err(|e| e.context_loc(start_loc, end_loc))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let x = self
            .value
            .take()
            .expect("called next_value_seed before next_key_seed");
        seed.deserialize(RonDeserializer { expr: x })
            .map_err(|e| e.context_loc(x.start.into(), x.end.into()))
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
        match self.iter.next() {
            Some(x) => {
                let key = kseed
                    .deserialize(RonDeserializer {
                        expr: &mut x.value.key,
                    })
                    .map_err(|e| e.context_loc(x.start.into(), x.end.into()))?;
                let value = vseed
                    .deserialize(RonDeserializer {
                        expr: &mut x.value.value,
                    })
                    .map_err(|e| e.context_loc(x.start.into(), x.end.into()))?;

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
    ident: &'a mut ast::Spanned<ast::Ident<'de>>,
}

impl<'a, 'de> Deserializer<'de> for IdentDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let start_loc = self.ident.start;
        let end_loc = self.ident.end;

        visitor
            .visit_borrowed_str(self.ident.value.0)
            .map_err(|e: Error| e.context_loc(start_loc, end_loc))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct EnumDeserializer<'a, 'de> {
    tagged: &'a mut ast::Tagged<'de>,
}

impl<'a, 'de> EnumAccess<'de> for EnumDeserializer<'a, 'de> {
    type Error = crate::error::Error;
    type Variant = UntaggedDeserializer<'a, 'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant_ident = seed.deserialize(IdentDeserializer {
            ident: &mut self.tagged.ident,
        })?;

        Ok((
            variant_ident,
            UntaggedDeserializer {
                untagged: &mut self.tagged.untagged,
            },
        ))
    }
}

struct UntaggedDeserializer<'a, 'de> {
    untagged: &'a mut ast::Spanned<ast::Untagged<'de>>,
}

impl<'a, 'de> VariantAccess<'de> for UntaggedDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.untagged.value.take() {
            Untagged::Struct(_) => todo!(),
            Untagged::Tuple(_) => todo!(),
            Untagged::Unit => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.untagged.value.take() {
            Untagged::Struct(_) => todo!(),
            Untagged::Tuple(mut t) => seed.deserialize(RonDeserializer {
                expr: t.elements.iter_mut().next().ok_or_else(|| Error::custom("invalid enum variant, got zero tuple elements, but expected one (newtype variant)"))?
            }),
            Untagged::Unit => todo!(),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.untagged.value.take() {
            Untagged::Struct(_) => todo!(),
            Untagged::Tuple(mut t) => visitor.visit_seq(SeqDeserializer {
                iter: t.elements.iter_mut(),
            }),
            Untagged::Unit => todo!(),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.untagged.value.take() {
            Untagged::Struct(mut s) => visitor.visit_map(StructDeserializer {
                iter: s.fields.iter_mut(),
                value: None,
            }),
            Untagged::Tuple(_) => todo!(),
            Untagged::Unit => todo!(),
        }
    }
}
