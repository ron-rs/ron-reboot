use ron_reboot::{from_str, print_error};
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
            print_error(&e).unwrap();
        }
    }
}
