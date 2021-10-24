use ron_reboot::{print_error, Error, Value};

fn app() -> Result<(), Error> {
    let value: Value = r#"
MyConfig (
    accurate: Types(are: "awesome"),
    even: ["tuples", "and", "lists"],
    can: {
        "be": ("told", "apart!"),

        "it's": true,
    },
)
    "#
    .parse()?;

    match value {
        Value::Struct(Some(ident), fields) => {
            println!("Config type: {}", ident);

            for (ident, value) in fields {
                println!("{}: {:?}", ident, value)
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}

fn main() {
    if let Err(e) = app() {
        print_error(&e).unwrap();
    }
}
