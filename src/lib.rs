#[cfg(feature = "serde1_serde")]
pub use self::serde::from_str;

mod error;
pub mod error_fmt;
pub mod utf8_parser;
#[cfg(feature = "serde1_serde")]
mod serde;
mod util;

// Integration tests cannot import this without the feature gate
// (not sure why that is...)
#[cfg(any(test, feature = "test"))]
pub mod test_util;
