#![cfg(test)]

use std::{collections::HashMap, iter::FromIterator};

use ron_reboot::{from_str_serde, utf8_parser::test_util::unwrap_display};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct MyStruct {
    foo: bool,
    bar: String,
    #[serde(flatten)]
    everything_else: HashMap<String, bool>,
}

#[test]
fn flattened_struct_support() {
    assert_eq!(
        unwrap_display(from_str_serde::<MyStruct>(
            r#"
(
    foo: false,
    bar: "bar",

    extension_baz: true,
)
        "#
        )),
        MyStruct {
            foo: false,
            bar: "bar".to_string(),
            everything_else: HashMap::from_iter(
                vec![("extension_baz".to_owned(), true)].into_iter()
            )
        }
    );
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum MyEnum {
    Bool(bool),
    MyStruct(MyStruct),
}

#[test]
fn untagged_enum_support() {
    assert_eq!(
        unwrap_display(from_str_serde::<MyEnum>(
            r#"
MyStruct(
    foo: false,
    bar: "bar",

    extension_baz: true,
)
        "#
        )),
        MyEnum::MyStruct(MyStruct {
            foo: false,
            bar: "bar".to_string(),
            everything_else: HashMap::from_iter(
                vec![("extension_baz".to_owned(), true)].into_iter()
            )
        })
    );

    assert_eq!(
        unwrap_display(from_str_serde::<MyEnum>(
            r#"
false
        "#
        )),
        MyEnum::Bool(false)
    );
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "t", content = "c")]
enum TypeTagged {
    Bool(bool),
    MyStruct(MyStruct),
}

#[test]
fn adjacently_tagged_enum_support() {
    assert_eq!(
        unwrap_display(from_str_serde::<TypeTagged>(
            r#"
(
    t: MyStruct,
    c: MyStruct(
        foo: false,
        bar: "bar",

        extension_baz: true,
    )
)
        "#
        )),
        TypeTagged::MyStruct(MyStruct {
            foo: false,
            bar: "bar".to_string(),
            everything_else: HashMap::from_iter(
                vec![("extension_baz".to_owned(), true)].into_iter()
            )
        })
    );

    assert_eq!(
        unwrap_display(from_str_serde::<TypeTagged>(
            r#"
(
    t: Bool,
    c: false
)
        "#
        )),
        TypeTagged::Bool(false)
    );
}
