#![cfg(test)]

use std::{collections::HashMap, iter::FromIterator};

use ron_reboot::serde::from_str;
use serde::Deserialize;
use ron_reboot::test_util::unwrap_display;

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
        unwrap_display(from_str::<MyStruct>(
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
