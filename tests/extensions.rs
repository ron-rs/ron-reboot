use ron_reboot::from_str_serde;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct SimpleNewtype(String);

#[derive(Debug, Deserialize, PartialEq)]
struct NestedNewtype(SimpleNewtype);

#[test]
fn test_simple_newtpype() {
    assert_eq!(from_str_serde::<SimpleNewtype>(r#"
#![enable(unwrap_newtypes)]

"Hello, world!"
    "#).unwrap(), SimpleNewtype("Hello, world!".to_owned()));

    assert_eq!(from_str_serde::<SimpleNewtype>(r#"
#![enable(unwrap_newtypes)]

("Hello, world!")
    "#).unwrap(), SimpleNewtype("Hello, world!".to_owned()));

    assert_eq!(from_str_serde::<SimpleNewtype>(r#"
#![enable(unwrap_newtypes)]

SimpleNewtype("Hello, world!")
    "#).unwrap(), SimpleNewtype("Hello, world!".to_owned()));
}


#[test]
fn test_nested_newtpype() {
    assert_eq!(from_str_serde::<NestedNewtype>(r#"
#![enable(unwrap_newtypes)]

"Hello, world!"
    "#).unwrap(), NestedNewtype(SimpleNewtype("Hello, world!".to_owned())));

    assert_eq!(from_str_serde::<NestedNewtype>(r#"
#![enable(unwrap_newtypes)]

("Hello, world!")
    "#).unwrap(), NestedNewtype(SimpleNewtype("Hello, world!".to_owned())));

    // We cannot skip the outer but not the inner newtype
    /*
    assert_eq!(from_str_serde::<NestedNewtype>(r#"
#![enable(unwrap_newtypes)]

SimpleNewtype("Hello, world!")
    "#).unwrap(), NestedNewtype(SimpleNewtype("Hello, world!".to_owned())));

     */

    assert_eq!(from_str_serde::<NestedNewtype>(r#"
#![enable(unwrap_newtypes)]

NestedNewtype("Hello, world!")
    "#).unwrap(), NestedNewtype(SimpleNewtype("Hello, world!".to_owned())));

    assert_eq!(from_str_serde::<NestedNewtype>(r#"
#![enable(unwrap_newtypes)]

NestedNewtype(SimpleNewtype("Hello, world!"))
    "#).unwrap(), NestedNewtype(SimpleNewtype("Hello, world!".to_owned())));
}

#[derive(Debug, Deserialize, PartialEq)]
struct CliOpts {
    source: Option<String>,
    target: SimpleNewtype,
    log: Option<Option<SimpleNewtype>>,
}

#[test]
fn test_implicit_some() {
    assert_eq!(from_str_serde::<CliOpts>(r#"
#![enable(implicit_some)]

(
    source: "a",
    target: ("b"),
    log: ("c"),
)
    "#).unwrap(), CliOpts {
        source: Some("a".to_owned()),
        target: SimpleNewtype("b".to_owned()),
        log: Some(Some(SimpleNewtype("c".to_owned()))),
    });

    assert_eq!(from_str_serde::<CliOpts>(r#"
#![enable(implicit_some)]

(
    source: None,
    target: ("b"),
    log: Some(("c")),
)
    "#).unwrap(), CliOpts {
        source: None,
        target: SimpleNewtype("b".to_owned()),
        log: Some(Some(SimpleNewtype("c".to_owned()))),
    });

    assert_eq!(from_str_serde::<CliOpts>(r#"
#![enable(implicit_some)]

(
    source: "a",
    target: ("b"),
    // if we want to specify `Some(None)`, we have to write it out
    log: None,
)
    "#).unwrap(), CliOpts {
        source: Some("a".to_owned()),
        target: SimpleNewtype("b".to_owned()),
        log: None,
    });
}

#[test]
fn both_implicit_some_unwrap_newtypes() {
    assert_eq!(from_str_serde::<CliOpts>(r#"
#![enable(implicit_some, unwrap_newtypes)]

(
    source: "a",
    target: ("b"),
    log: ("c"),
)
    "#).unwrap(), CliOpts {
        source: Some("a".to_owned()),
        target: SimpleNewtype("b".to_owned()),
        log: Some(Some(SimpleNewtype("c".to_owned()))),
    });

    assert_eq!(from_str_serde::<CliOpts>(r#"
#![enable(implicit_some, unwrap_newtypes)]

(
    source: None,
    target: "b",
    log: Some("c"),
)
    "#).unwrap(), CliOpts {
        source: None,
        target: SimpleNewtype("b".to_owned()),
        log: Some(Some(SimpleNewtype("c".to_owned()))),
    });

    assert_eq!(from_str_serde::<CliOpts>(r#"
#![enable(implicit_some, unwrap_newtypes)]

(
    source: "a",
    target: "b",
    log: None,
)
    "#).unwrap(), CliOpts {
        source: Some("a".to_owned()),
        target: SimpleNewtype("b".to_owned()),
        log: None,
    });
}
