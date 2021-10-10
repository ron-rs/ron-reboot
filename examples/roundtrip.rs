use ron::{ser::to_string_pretty, Value};
use ron_reboot::serde::from_str;

fn main() {
    let s = std::fs::read_to_string(std::env::args().nth(1).unwrap()).expect("file not found");

    match from_str::<Value>(&s) {
        Ok(x) => {
            println!("{}", to_string_pretty(&x, Default::default()).unwrap());

            println!();
            println!("Debug:");
            println!("{:#?}", x);

            println!();
            println!("AST:");
            println!(
                "{}",
                to_string_pretty(&ron_reboot::parser::ron(&s).unwrap(), Default::default())
                    .unwrap()
            );
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
