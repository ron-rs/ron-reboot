use structopt::StructOpt;
use ron_utils::{print_error, validate_file};

#[derive(Debug, StructOpt)]
#[structopt(name = "ron-utils")]
/// Rusty Object Notation (RON) utilities
enum Opt {
    /// Validate .ron file(s)
    Validate {
        #[structopt(required = true)]
        files: Vec<String>,
    }
}

fn main() {
    let opt: Opt = Opt::from_args();

    match opt {
        Opt::Validate { files } => {
            for file in &files {
                match validate_file(file) {
                    Ok(_) => {
                        println!("{}: ok", file);
                    }
                    Err(e) => {
                        println!("{}: ok", file);
                        let _ = print_error(&e);
                    }
                }
            }
        }
    }
}
