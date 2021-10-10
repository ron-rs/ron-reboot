use crate::error::{Error, ErrorKind::*, Location};
use crate::serde::from_str;

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
    assert_eq!(from_str::<&str>(r#" "What a nice, zero-copy string!" "#), Ok("What a nice, zero-copy string!"));
    assert_eq!(from_str::<&str>(r#" "😀😀" "#), Ok("😀😀"));
    assert_eq!(
        from_str::<&str>(r#"  "Escapes are \\ fun but not available here :|" "#),
        Err(Error { kind: ExpectedStrGotEscapes, start: Some(Location { line: 1, column: 3 }), end: Some(Location { line: 1, column: 49 }) })
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
