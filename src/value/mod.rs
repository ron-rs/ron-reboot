//! Value module.

// TODO: do all this with ast instead of serde

use std::{
    cmp::{Eq, Ordering},
    hash::{Hash, Hasher},
};

mod ast;
#[cfg(feature = "value_serde1")]
mod ser_de;

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

///! RON Value type.
///
/// ## Compatibility
///
/// Please note that we cannot accurately deserialize
/// into a `Value` with `serde` because its data model does
/// not account for struct vs map, list vs tuple, etc.
///
/// Results when deserializing from AST (recommended)
/// vs with a serde Deserializer (not recommended)
/// **will** differ!
///
/// Most notably, `serde` will produce a `List` in case of
/// a tuple.
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

#[cfg(all(test, feature = "value_serde1", feature = "utf8_parser_serde1"))]
mod tests {
    use std::{collections::BTreeMap, fmt::Debug};

    use super::*;

    fn assert_same<'de, T>(s: &'de str)
    where
        T: Debug + serde::Deserialize<'de> + PartialEq,
    {
        use crate::from_str;

        let direct: T = from_str(s).unwrap();
        let value_serde: Value = from_str(s).unwrap();
        let value = T::deserialize(value_serde).unwrap();

        assert_eq!(direct, value, "T::deserialize(str) and T::deserialize(Value::deserialize(str)) is not the same for {:?}", s);

        assert_same_pure::<T>(s);
    }

    /// Don't use serde to deserialize Value
    fn assert_same_pure<'de, T>(s: &'de str)
    where
        T: Debug + serde::Deserialize<'de> + PartialEq,
    {
        use crate::{from_str, utf8_parser::ast_from_str};

        let direct: T = from_str(s).unwrap();
        let value: Value = ast_from_str(s).unwrap().into();
        let value = T::deserialize(value).unwrap();

        assert_eq!(direct, value, "T::deserialize(str) and T::deserialize(Value::from(ast_from_str(str))) is not the same for {:?}", s);
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

    fn eval_serde_val(s: &str) -> Value {
        crate::utf8_parser::from_str(s).unwrap()
    }

    #[test]
    fn test_none() {
        assert_same::<Option<i32>>("None");
    }

    #[test]
    fn test_some() {
        assert_same::<Option<()>>("Some(())");
        assert_same::<Option<()>>("Some  (  () )");
    }

    #[test]
    fn test_tuples_basic() {
        assert_same::<(f64, f64, f64)>("(3.0, 4.0, 5.0)");
    }

    #[test]
    #[ignore]
    fn test_floats() {
        assert_eq!(
            eval_serde_val("(inf, -inf, NaN)"),
            Value::Tuple(
                None,
                vec![
                    Value::Number(Number::new(std::f64::INFINITY)),
                    Value::Number(Number::new(std::f64::NEG_INFINITY)),
                    Value::Number(Number::new(std::f64::NAN)),
                ]
            ),
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            eval_serde_val(
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
