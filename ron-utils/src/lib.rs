use std::{fs::read_to_string, path::Path};

use ron_reboot::utf8_parser::ast_from_str;

pub use ron_reboot::{print_error, Error};

pub fn validate_str(s: &str) -> Result<(), ron_reboot::Error> {
    ast_from_str(s).map(|_| ())
}

pub fn validate_file(p: impl AsRef<Path>) -> Result<(), ron_reboot::Error> {
    ast_from_str(&read_fs_string(p)?).map(|_| ())
}

#[cfg(feature = "serde1")]
pub fn validate_typed_str<'a, T: serde::Deserialize<'a>>(s: &'a str) -> Result<(), ron_reboot::Error> {
    ron_reboot::utf8_parser::from_str(s)
}

#[cfg(feature = "serde1")]
pub fn validate_typed_file<T: serde::de::DeserializeOwned>(
    p: impl AsRef<Path>,
) -> Result<(), ron_reboot::Error> {
    ron_reboot::utf8_parser::from_str(&read_fs_string(p)?)
}

fn read_fs_string(path: impl AsRef<Path>) -> Result<String, ron_reboot::Error> {
    let path = path.as_ref();
    read_to_string(path)
        .map_err(ron_reboot::Error::from)
        .map_err(|e: ron_reboot::Error| e.context_file_name(path.display().to_string()))
}
