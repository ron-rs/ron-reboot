use crate::serde::from_str;

#[test]
fn bools() {
    assert_eq!(from_str::<bool>("true"), Ok(true));
    assert_eq!(from_str::<bool>("false"), Ok(false));
    assert!(from_str::<bool>("neither").is_err());
}

#[test]
fn strings() {
    assert_eq!(from_str::<String>(r#" "Well this is fun" "#), Ok("Well this is fun".to_owned()));
    assert_eq!(from_str::<String>(r#" "😀😀" "#), Ok("😀😀".to_owned()));
    assert_eq!(from_str::<String>(r#"  "Escapes \t are \n fun! \u{1F913}" "#), Ok("Escapes \t are \n fun! \u{1F913}".to_owned()));
}

#[test]
fn lists() {
    assert_eq!(from_str::<Vec<bool>>("[true, false]"), Ok(vec![true, false]));
    assert_eq!(from_str::<Vec<bool>>("[false, false, false, ]"), Ok(vec![false, false, false]));
}
