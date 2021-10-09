use ron_nom::parser::{expr, Input};
use std::io::{stdin, Read};
use nom::Err;
use nom_supreme::error::ErrorTree;

fn main() {
    let mut s = String::new();
    stdin().read_to_string(&mut s).unwrap();

    match expr(Input::new(&s)) {
        Ok(_) => println!("ok"),
        Err(e) => {
            let e = match e {
                Err::Incomplete(i) => panic!("Incomplete: {:?}", i),
                Err::Error(e) => panic!("Error (not failure?): {:?}", e),
                Err::Failure(e) => e,
            };
            let e = e.map_locations(|input| format!("{}:{}", input.location_line(), input.get_utf8_column()));
            let mut e = &e as &dyn std::error::Error;
            eprintln!("error: {}", e);

            while let Some(s) = e.source() {
                eprintln!("  caused by: {}", s);
                e = s;
            }
        }
    }
}
