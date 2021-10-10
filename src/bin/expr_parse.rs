use ron_nom::error_fmt::ErrorTreeFmt;
use ron_nom::parser::ron;
use std::io::{stdin, Read};

fn main() {
    let mut s = String::new();
    stdin().read_to_string(&mut s).unwrap();

    match ron(&s) {
        Ok(_) => println!("ok"),
        Err(e) => {
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
