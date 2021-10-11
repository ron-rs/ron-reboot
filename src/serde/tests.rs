use serde::Deserialize;

use crate::{
    error::{Error, ErrorKind::*},
    serde::from_str,
};

#[derive(Debug, Deserialize, PartialEq)]
struct MyStruct {
    x: bool,
    y: String,
}

#[test]
fn structs() {
    assert_eq!(
        from_str(
            r#"
 (x: true, y:"false") "#
        ),
        Ok(MyStruct {
            x: true,
            y: "false".to_string()
        })
    );
    assert_eq!(
        from_str(r#" MyStruct(x:false,y:"true") "#),
        Ok(MyStruct {
            x: false,
            y: "true".to_string()
        })
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct MyNestedStruct {
    foo: MyStruct,
    //bar: bool,
}

#[test]
fn structs_nested() {
    assert_eq!(
        from_str(
            r#"
 (foo: (x: false, y: "abc")) "#
        ),
        Ok(MyNestedStruct {
            foo: MyStruct {
                x: false,
                y: "abc".to_string()
            },
            //bar: false
        })
    );
}

/*
fn struct_fail() {
    let input = r#"Example(xyz: Asdf(
        x: 4, y: !
    ),
)"#;
}

 */

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
    assert_eq!(
        from_str::<&str>(r#" "What a nice, zero-copy string!" "#),
        Ok("What a nice, zero-copy string!")
    );
    assert_eq!(from_str::<&str>(r#" "😀😀" "#), Ok("😀😀"));
    assert_eq!(
        from_str::<&str>(r#"  "Escapes are \\ fun but not available here :|" "#),
        Err(Error { kind: Custom(r#"invalid type: string "Escapes are \\ fun but not available here :|", expected a borrowed string"#.to_owned()), start: None, end: None })
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
