use std::{collections::HashMap, hash::Hash, iter::FromIterator};

use serde::Deserialize;

use crate::{error::ErrorKind::*, utf8_parser::serde::from_str as normal_from_str};

// Custom wrapper for tests to make error easier to read
pub fn from_str<'a, T>(s: &'a str) -> Result<T, crate::error::Error>
where
    T: Deserialize<'a>,
{
    normal_from_str(s).map_err(|e| {
        //let _ = crate::print_error(&e);

        e
    })
}

#[derive(Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum MyEnum {
    UnitVariant1,
    UnitVariant2,
    NewtypeVariant(String),
    TupleVariantEmpty(),
    TupleVariant(i32, i32),
    StructVariant { a: bool, b: String },
}

#[test]
fn mixed_enum_unit() {
    assert_eq!(
        from_str(
            r#"
UnitVariant1
"#
        ),
        Ok(MyEnum::UnitVariant1)
    );

    assert_eq!(
        from_str(
            r#"
UnitVariant2
"#
        ),
        Ok(MyEnum::UnitVariant2)
    );
}

#[test]
fn mixed_enum_newtype() {
    assert_eq!(
        from_str(
            r#"
NewtypeVariant("Newtypes are a special case in serde")
"#
        ),
        Ok(MyEnum::NewtypeVariant("Newtypes are a special case in serde".to_owned()))
    );
}

#[test]
fn mixed_enum_tuple() {
    assert_eq!(
        from_str(
            r#"
TupleVariantEmpty()
"#
        ),
        Ok(MyEnum::TupleVariantEmpty())
    );

    assert_eq!(
        from_str(
            r#"
TupleVariant(1, 2)
"#
        ),
        Ok(MyEnum::TupleVariant(1, 2))
    );
}

#[test]
fn mixed_enum_struct() {
    assert_eq!(
        from_str(
            r#"
StructVariant (
    a: true,
    b: "Hello"
)
"#
        ),
        Ok(MyEnum::StructVariant {
            a: true,
            b: "Hello".to_owned()
        })
    );
}

#[derive(Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct MyStruct {
    x: bool,
    y: String,
}

#[test]
fn structs() {
    assert_eq!(
        from_str(
            r#"
 (x: true, y:"false") "#
        ),
        Ok(MyStruct {
            x: true,
            y: "false".to_string()
        })
    );
    assert_eq!(
        from_str(r#" MyStruct(x:false,y:"true") "#),
        Ok(MyStruct {
            x: false,
            y: "true".to_string()
        })
    );
}

#[test]
fn maps() {
    assert_eq!(
        from_str::<HashMap<String, String>>(
            r#"
{
    "string\\ key": "1.2e3",
    "this is": "a test",
}"#
        ),
        Ok(HashMap::<String, String>::from_iter(vec![
            ("string\\ key".to_owned(), "1.2e3".to_owned()),
            ("this is".to_owned(), "a test".to_owned()),
        ]))
    );

    assert_eq!(
        from_str::<HashMap<MyStruct, String>>(
            r#"
{
    (x: true, y: "a"): "a",
    (x: false, y: "b"): "b",
}"#
        ),
        Ok(HashMap::from_iter(vec![
            (
                MyStruct {
                    x: true,
                    y: "a".to_string()
                },
                "a".to_owned()
            ),
            (
                MyStruct {
                    x: false,
                    y: "b".to_string()
                },
                "b".to_owned()
            ),
        ]))
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct MyNestedStruct {
    foo: MyStruct,
    //bar: bool,
}

#[test]
fn structs_nested() {
    assert_eq!(
        from_str(
            r#"
 (foo: (x: false, y: "abc")) "#
        ),
        Ok(MyNestedStruct {
            foo: MyStruct {
                x: false,
                y: "abc".to_string()
            },
            //bar: false
        })
    );
}

/*
fn struct_fail() {
    let input = r#"Example(xyz: Asdf(
        x: 4, y: !
    ),
)"#;
}

 */

#[test]
fn floats() {
    assert_eq!(from_str::<f64>("1.0"), Ok(1.0));
    assert_eq!(from_str::<f64>("-3.14"), Ok(-3.14));
    assert_eq!(from_str::<f64>("1.0e3"), Ok(1.0e3));
    assert_eq!(from_str::<f64>(".001e3"), Ok(0.001e3));
    assert_eq!(from_str::<f64>("+3.14"), Ok(3.14));
}

#[test]
fn ints() {
    assert_eq!(from_str::<i32>("-123"), Ok(-123));
    assert_eq!(from_str::<i32>("+123"), Ok(123));
    assert_eq!(from_str::<i32>("123"), Ok(123));
    assert_eq!(
        from_str::<u64>("18446744073709551615"),
        Ok(18446744073709551615)
    );
    assert_eq!(
        from_str::<i64>("9223372036854775807"),
        Ok(9223372036854775807)
    );
    //assert_eq!(from_str::<i64>("-9223372036854775808"), Ok(-9223372036854775808));
    // TODO fix
}

#[test]
fn bools() {
    assert_eq!(from_str::<bool>("true"), Ok(true));
    assert_eq!(from_str::<bool>("false"), Ok(false));
    assert!(from_str::<bool>("neither").is_err());
}

#[test]
fn strings() {
    assert_eq!(
        from_str::<String>(r#" "Well this is fun" "#),
        Ok("Well this is fun".to_owned())
    );
    assert_eq!(from_str::<String>(r#" "😀😀" "#), Ok("😀😀".to_owned()));
    assert_eq!(
        from_str::<String>(r#"  "Escapes \t are \n fun! \u{1F913}" "#),
        Ok("Escapes \t are \n fun! \u{1F913}".to_owned())
    );
}

#[test]
fn zero_copy_strs() {
    assert_eq!(
        from_str::<&str>(r#" "What a nice, zero-copy string!" "#),
        Ok("What a nice, zero-copy string!")
    );
    assert_eq!(from_str::<&str>(r#" "😀😀" "#), Ok("😀😀"));
    assert_eq!(
        from_str::<&str>(r#"  "Escapes are \\ fun but not available here :|" "#).unwrap_err().kind,
        Custom(r#"invalid type: string "Escapes are \\ fun but not available here :|", expected a borrowed string"#.to_owned()),
    );
}

#[test]
fn lists() {
    assert_eq!(
        from_str::<Vec<bool>>("[true, false]"),
        Ok(vec![true, false])
    );
    assert_eq!(
        from_str::<Vec<bool>>("[false, false, false, ]"),
        Ok(vec![false, false, false])
    );
}
