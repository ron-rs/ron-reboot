use std::{fs::File, io::Read, path::Path};

use serde::de::DeserializeOwned;

pub use self::de::from_str;
use crate::Error;

mod de;
#[cfg(test)]
mod tests;

pub fn from_reader<R: Read, T: DeserializeOwned>(mut reader: R) -> Result<T, Error> {
    let mut buf = String::new();

    reader.read_to_string(&mut buf).map_err(Error::from)?;

    from_str(&buf)
}

pub fn from_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, Error> {
    let path = path.as_ref();

    File::open(path)
        .map_err(Error::from)
        .and_then(from_reader)
        .map_err(|e| e.context_file_name(path.display().to_string()))
}
