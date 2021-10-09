use nom::Err;
use ron_nom::error_fmt::ErrorTreeFmt;
use ron_nom::parser::{expr, Input};
use std::io::{stdin, Read};

fn main() {
    let mut s = String::new();
    stdin().read_to_string(&mut s).unwrap();

    match expr(Input::new(&s)) {
        Ok(_) => println!("ok"),
        Err(e) => {
            let e = match e {
                Err::Incomplete(i) => panic!("Incomplete: {:?}", i),
                Err::Error(e) => e,
                Err::Failure(e) => e,
            };
            let e = ErrorTreeFmt::new(e);
            let mut e = &e as &dyn std::error::Error;
            eprintln!("error: {}", e);

            while let Some(s) = e.source() {
                eprintln!("  caused by: {}", s);
                e = s;
            }
        }
    }
}
