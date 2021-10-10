use std::collections::HashMap;
use std::iter::FromIterator;
use serde::{Deserialize};
use ron_nom::serde::from_str;

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
        from_str::<MyStruct>(r#"
(
    foo: false,
    bar: "bar",

    extension_baz: true,
)
        "#).unwrap(), MyStruct {
            foo: false,
            bar: "bar".to_string(),
            everything_else: HashMap::from_iter(vec![
                ("extension_baz".to_owned(), true)
            ].into_iter())
        });
}
