use ron_nom::parser::{expr, Input};
use std::io::{stdin, Read};

fn main() {
    let mut s = String::new();
    stdin().read_to_string(&mut s).unwrap();

    match expr(Input::new(&s)) {
        Ok(_) => println!("ok"),
        Err(e) => {
            let mut e: &dyn std::error::Error = &e;
            eprintln!("Error: {:?}", e);

            while let Some(s) = e.source() {
                eprintln!("  caused by: {:?}", s);
                e = s;
            }
        }
    }
}
