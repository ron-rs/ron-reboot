use std::{fmt, str::FromStr};

use ron_utils::print_error;

pub enum PrintOpt {
    PrettyErrors,
    ErrorStatus,
    OkStatus,
    Status,
    StatusAndPrettyError,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl fmt::Debug for PrintOpt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (&*self,) {
            (&PrintOpt::PrettyErrors,) => fmt::Formatter::write_str(f, "PrettyErrors"),
            (&PrintOpt::ErrorStatus,) => fmt::Formatter::write_str(f, "ErrorStatus"),
            (&PrintOpt::OkStatus,) => fmt::Formatter::write_str(f, "OkStatus"),
            (&PrintOpt::Status,) => fmt::Formatter::write_str(f, "Status"),
            (&PrintOpt::StatusAndPrettyError,) => {
                fmt::Formatter::write_str(f, "StatusAndPrettyError")
            }
        }
    }
}
impl FromStr for PrintOpt {
    type Err = String;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        #[allow(deprecated, unused_imports)]
        use ::std::ascii::AsciiExt;
        match s {
            "pretty-errors" => Ok(PrintOpt::PrettyErrors),
            "err-status" => Ok(PrintOpt::ErrorStatus),
            "ok-status" => Ok(PrintOpt::OkStatus),
            "status" => Ok(PrintOpt::Status),
            "status-and-pretty-errors" => Ok(PrintOpt::StatusAndPrettyError),
            _ => Err(format!(
                "valid values: {}",
                Self::variants().to_vec().join(", ")
            )),
        }
    }
}

impl PrintOpt {
    pub fn variants() -> [&'static str; 1 + (1 + (1 + (1 + 1)))] {
        [
            "pretty-errors",
            "err-status",
            "ok-status",
            "status",
            "status-and-pretty-errors",
        ]
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
