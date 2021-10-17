use std::process::exit;

use ron_utils::validate_file;
use structopt::StructOpt;

use crate::print_opt::PrintOpt;

mod print_opt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ron-utils")]
/// Rusty Object Notation (RON) utilities
enum Opt {
    /// Validate .ron file(s)
    Validate {
        #[structopt(long)]
        /// Fail on first error encountered
        fail_fast: bool,
        #[structopt(long, required = false, default_value = "status-and-pretty-errors", possible_values = &PrintOpt::variants())]
        /// What to print
        print: PrintOpt,
        #[structopt(required = true)]
        /// The .ron files to validate
        files: Vec<String>,
    },
}

fn main() {
    let opt: Opt = Opt::from_args();

    match opt {
        Opt::Validate {
            files,
            print,
            fail_fast,
        } => {
            let mut error = false;

            for file in &files {
                match validate_file(file) {
                    Ok(_) => {
                        print.print_ok(file);
                    }
                    Err(e) => {
                        print.print_err(file);
                        print.print_pretty_error(&e);
                        if fail_fast {
                            exit(1);
                        } else {
                            error = true;
                        }
                    }
                }
            }

            if error {
                exit(1);
            }
        }
    }
}
