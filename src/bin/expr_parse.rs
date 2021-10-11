#![cfg(never)]

use std::io::{stdin, Read};

use ron_reboot::{error_fmt::ErrorTreeFmt, parser::ron};

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
