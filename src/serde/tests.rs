use crate::error::ErrorKind::ExpectedBool;
use crate::error::{ron_err, ron_err_custom};
use crate::serde::from_str;

#[test]
fn bools() {
    assert_eq!(from_str::<bool>("true"), Ok(true));
    assert_eq!(from_str::<bool>("false"), Ok(false));
    assert!(from_str::<bool>("neither").is_err());
}
