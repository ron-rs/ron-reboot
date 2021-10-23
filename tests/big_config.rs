use std::collections::HashMap;

use ron_reboot::{from_str, utf8_parser::test_util::unwrap_display};
use serde::Deserialize;

const INPUT: &str = include_str!("big_config.ron");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    flags: Vec<Flag>,
    mapping: HashMap<String, Data>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Flag {
    Unit,
    EmptyTuple(),
    NewType(Data),
    Tuple(i32, Option<u64>),
    Struct {
        optional: Option<String>,
        very_optional: Option<Option<Data>>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub string: String,
    #[serde(rename = "int")]
    pub rename_me: i32,
}

#[test]
fn big_config() {
    unwrap_display(from_str::<Config>(INPUT));
}
