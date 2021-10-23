//! Value module.

// TODO: do all this with ast instead of serde

use serde::{de::{
    DeserializeOwned, DeserializeSeed, Deserializer, Error as SerdeError, MapAccess, SeqAccess,
    Visitor,
}, Deserialize, forward_to_deserialize_any};
use std::{cmp::{Eq, Ordering}, fmt, hash::{Hash, Hasher}};
use serde::de::{EnumAccess, VariantAccess};
use crate::Error;

/// A wrapper for a number, which can be either `f64` or `i64`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Number {
    Integer(i64),
    Float(Float),
}

/// A wrapper for `f64`, which guarantees that the inner value
/// is finite and thus implements `Eq`, `Hash` and `Ord`.
#[derive(Copy, Clone, Debug)]
pub struct Float(f64);

impl Float {
    /// Construct a new `Float`.
    pub fn new(v: f64) -> Self {
        Float(v)
    }

    /// Returns the wrapped float.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl Number {
    /// Construct a new number.
    pub fn new(v: impl Into<Number>) -> Self {
        v.into()
    }

    /// Returns the `f64` representation of the number regardless of whether the number is stored
    /// as a float or integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert_eq!(i.into_f64(), 5.0);
    /// assert_eq!(f.into_f64(), 2.0);
    /// ```
    pub fn into_f64(self) -> f64 {
        self.map_to(|i| i as f64, |f| f)
    }

    /// If the `Number` is a float, return it. Otherwise return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert_eq!(i.as_f64(), None);
    /// assert_eq!(f.as_f64(), Some(2.0));
    /// ```
    pub fn as_f64(self) -> Option<f64> {
        self.map_to(|_| None, Some)
    }

    /// If the `Number` is an integer, return it. Otherwise return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert_eq!(i.as_i64(), Some(5));
    /// assert_eq!(f.as_i64(), None);
    /// ```
    pub fn as_i64(self) -> Option<i64> {
        self.map_to(Some, |_| None)
    }

    /// Map this number to a single type using the appropriate closure.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert!(i.map_to(|i| i > 3, |f| f > 3.0));
    /// assert!(!f.map_to(|i| i > 3, |f| f > 3.0));
    /// ```
    pub fn map_to<T>(
        self,
        integer_fn: impl FnOnce(i64) -> T,
        float_fn: impl FnOnce(f64) -> T,
    ) -> T {
        match self {
            Number::Integer(i) => integer_fn(i),
            Number::Float(Float(f)) => float_fn(f),
        }
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Number {
        Number::Float(Float(f))
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Number {
        Number::Integer(i)
    }
}

impl From<i32> for Number {
    fn from(i: i32) -> Number {
        Number::Integer(i64::from(i))
    }
}

// The following number conversion checks if the integer fits losslessly into an i64, before
// constructing a Number::Integer variant. If not, the conversion defaults to float.

impl From<u64> for Number {
    fn from(i: u64) -> Number {
        if i <= std::i64::MAX as u64 {
            Number::Integer(i as i64)
        } else {
            Number::new(i as f64)
        }
    }
}

/// Partial equality comparison
/// In order to be able to use `Number` as a mapping key, NaN floating values
/// wrapped in `Float` are equals to each other. It is not the case for
/// underlying `f64` values itself.
impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_nan() && other.0.is_nan() || self.0 == other.0
    }
}

/// Equality comparison
/// In order to be able to use `Float` as a mapping key, NaN floating values
/// wrapped in `Float` are equals to each other. It is not the case for
/// underlying `f64` values itself.
impl Eq for Float {}

impl Hash for Float {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64);
    }
}

/// Partial ordering comparison
/// In order to be able to use `Number` as a mapping key, NaN floating values
/// wrapped in `Number` are equals to each other and are less then any other
/// floating value. It is not the case for the underlying `f64` values themselves.
/// ```
/// use ron::value::Number;
/// assert!(Number::new(std::f64::NAN) < Number::new(std::f64::NEG_INFINITY));
/// assert_eq!(Number::new(std::f64::NAN), Number::new(std::f64::NAN));
/// ```
impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            _ => self.0.partial_cmp(&other.0),
        }
    }
}

/// Ordering comparison
/// In order to be able to use `Float` as a mapping key, NaN floating values
/// wrapped in `Float` are equals to each other and are less then any other
/// floating value. It is not the case for underlying `f64` values itself. See
/// the `PartialEq` implementation.
impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Bug: Contract violation")
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Char(char),
    Map(Vec<(Value, Value)>),
    Number(Number),
    Option(Option<Box<Value>>),
    String(String),
    List(Vec<Value>),
    Unit(Option<String>),
    Tuple(Option<String>, Vec<Value>),
    Struct(Option<String>, Vec<(String, Value)>),
}

impl Value {
    /// Tries to deserialize this `Value` into `T`.
    pub fn into_rust<T>(self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        T::deserialize(self)
    }
}

/// Deserializer implementation for RON `Value`.
impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
    {
        match self {
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Char(c) => visitor.visit_char(c),
            Value::Map(m) => visitor.visit_map(MapAccessor {
                keys: m.iter().rev().map(|kv| kv.0.clone()).collect(),
                values: m.iter().rev().map(|kv| kv.1.clone()).collect()
            }),
            Value::Number(Number::Float(ref f)) => visitor.visit_f64(f.get()),
            Value::Number(Number::Integer(i)) => visitor.visit_i64(i),
            Value::Option(Some(o)) => visitor.visit_some(*o),
            Value::Option(None) => visitor.visit_none(),
            Value::String(s) => visitor.visit_string(s),
            Value::Unit(Some(tag)) => visitor.visit_enum(EnumDeserializer {
                ident: tag,
                value: Value::Unit(None),
            }),
            Value::Unit(None) => visitor.visit_unit(),
            Value::List(l) => visitor.visit_seq(Seq { seq: l.into_iter().rev().collect() }),
            Value::Tuple(Some(tag), untagged) => {
                visitor.visit_enum(EnumDeserializer {
                    ident: tag,
                    value: Value::Tuple(None, untagged),
                })
            }
            Value::Tuple(None, seq) => visitor.visit_seq(Seq { seq }),
            Value::Struct(Some(tag), untagged) => {
                visitor.visit_enum(EnumDeserializer {
                    ident: tag,
                    value: Value::Struct(None, untagged),
                })
            }
            Value::Struct(None, m) => visitor.visit_map(MapAccessor {
                keys: m.iter().rev().map(|kv| kv.0.clone()).map(Value::String).collect(),
                values: m.iter().rev().map(|kv| kv.1.clone()).collect()
            }),
        }
    }
}


struct IdentDeserializer {
    ident: String,
}

impl<'de> Deserializer<'de> for IdentDeserializer {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        visitor
            .visit_string(self.ident)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct EnumDeserializer {
    ident: String,
    value: Value,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = crate::error::Error;
    type Variant = Value;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where
            V: DeserializeSeed<'de>,
    {
        let variant_ident = seed.deserialize(IdentDeserializer {
            ident: self.ident,
        })?;

        Ok((
            variant_ident,
            self.value
        ))
    }
}

impl<'de> VariantAccess<'de> for Value {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self {
            Value::Unit(None) => Ok(()),
            _ => Err(Error::custom(format!("expected unit, got {:?}", self)))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error> where T: DeserializeSeed<'de> {
        match self {
            Value::Tuple(None, mut values) if values.len() == 1 => seed.deserialize(values.remove(0)),
            _ => Err(Error::custom(format!("expected newtype, got {:?}", self)))
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        match self {
            Value::Tuple(None, values) if values.len() == len => Value::Tuple(None, values).deserialize_tuple(len, visitor),
            _ => Err(Error::custom(format!("expected tuple, got {:?}", self)))
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        match self {
            this @ Value::Struct(None, _) => this.deserialize_any(visitor),
            this => Err(Error::custom(format!("expected struct, got {:?}", this)))
        }
    }
}

struct MapAccessor {
    keys: Vec<Value>,
    values: Vec<Value>,
}

impl<'de> MapAccess<'de> for MapAccessor {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where
            K: DeserializeSeed<'de>,
    {
        // The `Vec` is reversed, so we can pop to get the originally first element
        self.keys
            .pop()
            .map_or(Ok(None), |v| seed.deserialize(v).map(Some))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where
            V: DeserializeSeed<'de>,
    {
        // The `Vec` is reversed, so we can pop to get the originally first element
        self.values
            .pop()
            .map(|v| seed.deserialize(v))
            .expect("Contract violation")
    }
}

struct Seq {
    seq: Vec<Value>,
}

impl<'de> SeqAccess<'de> for Seq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where
            T: DeserializeSeed<'de>,
    {
        // The `Vec` is reversed, so we can pop to get the originally first element
        self.seq
            .pop()
            .map_or(Ok(None), |v| seed.deserialize(v).map(Some))
    }
}

#[cfg(feature = "utf8_parser_serde1")]
impl std::str::FromStr for Value {
    type Err = Error;

    /// Creates a value from a string reference.
    fn from_str(s: &str) -> Result<Self, Error> {
        crate::utf8_parser::from_str(s)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a RON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Number(Number::new(v)))
    }

    #[cfg(integer128)]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Number(Number::new(v)))
    }

    #[cfg(integer128)]
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Number(Number::new(v)))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Char(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        self.visit_string(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        self.visit_byte_buf(v.to_vec())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        self.visit_string(String::from_utf8(v).map_err(|e| serde::de::Error::custom(format!("{}", e)))?)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
    {
        Ok(Value::Option(Some(Box::new(
            deserializer.deserialize_any(ValueVisitor)?,
        ))))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Value::Unit(None))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        if let Some(cap) = seq.size_hint() {
            vec.reserve_exact(cap);
        }

        while let Some(x) = seq.next_element()? {
            vec.push(x);
        }

        Ok(Value::List(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
    {
        let mut res: Vec<(Value, Value)> = Vec::new();

        while let Some(entry) = map.next_entry()? {
            res.push((entry.0, entry.1));
        }

        Ok(Value::Map(res))
    }
}


#[cfg(all(test, feature = "utf8_parser_serde1"))]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::{collections::BTreeMap, fmt::Debug};

    fn assert_same<'de, T>(s: &'de str)
        where
            T: Debug + Deserialize<'de> + PartialEq,
    {
        use crate::from_str;

        let direct: T = from_str(s).unwrap();
        let value: Value = from_str(s).unwrap();
        let value = T::deserialize(value).unwrap();

        assert_eq!(direct, value, "Deserialization for {:?} is not the same", s);
    }

    #[test]
    fn boolean() {
        assert_same::<bool>("true");
        assert_same::<bool>("false");
    }

    #[test]
    fn float() {
        assert_same::<f64>("0.123");
        assert_same::<f64>("-4.19");
    }

    #[test]
    fn int() {
        assert_same::<u32>("626");
        assert_same::<i32>("-50");
    }

    #[test]
    #[ignore]
    fn char() {
        assert_same::<char>("'4'");
        assert_same::<char>("'c'");
    }

    #[test]
    fn map() {
        assert_same::<BTreeMap<String, String>>(
            "{
\"a\": \"Hello\",
\"b\": \"Bye\",
        }",
        );
    }

    #[test]
    fn option() {
        assert_same::<Option<bool>>("Some(true)");
        assert_same::<Option<char>>("None");
    }

    #[test]
    fn seq() {
        assert_same::<Vec<f64>>("[1.0, 2.0, 3.0, 4.0]");
    }

    #[test]
    fn unit() {
        assert_same::<()>("()");
    }

    fn eval(s: &str) -> Value {
        s.parse().expect("Failed to parse")
    }

    #[test]
    fn test_none() {
        assert_eq!(eval("None"), Value::Option(None));
    }

    #[test]
    fn test_some() {
        assert_eq!(eval("Some(())"), Value::Option(Some(Box::new(Value::Unit(None)))));
        assert_eq!(
            eval("Some  (  () )"),
            Value::Option(Some(Box::new(Value::Unit(None))))
        );
    }

    #[test]
    fn test_tuples_basic() {
        assert_eq!(
            eval("(3, 4.0, 5.0)"),
            Value::Tuple(None, vec![
                Value::Number(Number::new(3)),
                Value::Number(Number::new(4.0)),
                Value::Number(Number::new(5.0)),
            ],),
        );
    }

    #[test]
    fn test_tuples_ident() {
        assert_eq!(
            eval("(true, 3, 4, 5.0)"),
            Value::Tuple(None, vec![
                Value::Bool(true),
                Value::Number(Number::new(3)),
                Value::Number(Number::new(4)),
                Value::Number(Number::new(5.0)),
            ]),
        );
    }

    #[test]
    #[ignore]
    fn test_floats() {
        assert_eq!(
            eval("(inf, -inf, NaN)"),
            Value::Tuple(None, vec![
                Value::Number(Number::new(std::f64::INFINITY)),
                Value::Number(Number::new(std::f64::NEG_INFINITY)),
                Value::Number(Number::new(std::f64::NAN)),
            ]),
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            eval(
                "Some([
    Room ( width: 20, height: 5, name: \"The Room\" ),

    (
        width: 10.0,
        height: 10.0,
        name: \"Another room\",
        enemy_levels: {
            \"Enemy1\": 3,
            \"Enemy2\": 5,
            \"Enemy3\": 7,
        },
    ),
])"
            ),
            Value::Option(Some(Box::new(Value::List(vec![
                Value::Map(
                    vec![
                        (
                            Value::String("width".to_owned()),
                            Value::Number(Number::new(20)),
                        ),
                        (
                            Value::String("height".to_owned()),
                            Value::Number(Number::new(5)),
                        ),
                        (
                            Value::String("name".to_owned()),
                            Value::String("The Room".to_owned()),
                        ),
                    ]
                        .into_iter()
                        .collect(),
                ),
                Value::Map(
                    vec![
                        (
                            Value::String("width".to_owned()),
                            Value::Number(Number::new(10.0)),
                        ),
                        (
                            Value::String("height".to_owned()),
                            Value::Number(Number::new(10.0)),
                        ),
                        (
                            Value::String("name".to_owned()),
                            Value::String("Another room".to_owned()),
                        ),
                        (
                            Value::String("enemy_levels".to_owned()),
                            Value::Map(
                                vec![
                                    (
                                        Value::String("Enemy1".to_owned()),
                                        Value::Number(Number::new(3)),
                                    ),
                                    (
                                        Value::String("Enemy2".to_owned()),
                                        Value::Number(Number::new(5)),
                                    ),
                                    (
                                        Value::String("Enemy3".to_owned()),
                                        Value::Number(Number::new(7)),
                                    ),
                                ]
                                    .into_iter()
                                    .collect(),
                            ),
                        ),
                    ]
                        .into_iter()
                        .collect(),
                ),
            ]))))
        );
    }
}
