use std::process::exit;

use ron_utils::{print_error, validate_file};
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(Debug)]
    pub enum PrintOpt {
        PrettyErrors,
        ErrorStatus,
        OkStatus,
        Status,
        StatusAndPrettyError,
    }
}

impl PrintOpt {
    pub fn print_ok(&self, file_name: &str) {
        use PrintOpt::*;

        match self {
            OkStatus | StatusAndPrettyError | Status => {
                println!("{} ok", file_name)
            }
            _ => {}
        }
    }

    pub fn print_err(&self, file_name: &str) {
        use PrintOpt::*;

        match self {
            ErrorStatus | StatusAndPrettyError | Status => {
                println!("{} err", file_name)
            }
            _ => {}
        }
    }

    pub fn print_pretty_error(&self, error: &ron_utils::Error) {
        use PrintOpt::*;

        match self {
            PrettyErrors | StatusAndPrettyError => {
                let _ = print_error(error);
            }
            _ => {}
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "ron-utils")]
/// Rusty Object Notation (RON) utilities
enum Opt {
    /// Validate .ron file(s)
    Validate {
        #[structopt(long)]
        /// Fail on first error encountered
        fail_fast: bool,
        #[structopt(long, required = false, default_value = "StatusAndPrettyError", possible_values = &PrintOpt::variants(), case_insensitive = true)]
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
