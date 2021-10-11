use ron_reboot::serde::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MyStruct {
    x: bool,
    y: String,
}

fn main() {
    let s = std::fs::read_to_string(std::env::args().nth(1).unwrap()).expect("file not found");

    match from_str::<MyStruct>(&s) {
        Ok(x) => {

            println!("Debug:");
            println!("{:#?}", x);
        }
        Err(e) => {
            let mut e = &e as &dyn std::error::Error;
            eprintln!("{}", e);

            while let Some(s) = e.source() {
                eprintln!("  caused by: {}", s);
                e = s;
            }
        }
    }
}
