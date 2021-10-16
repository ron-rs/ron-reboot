use std::{
    fmt::{Display, Formatter},
    io::stdout,
};

use crate::{
    error_fmt::ErrorTreeFmt,
    utf8_parser::{InputParseError, Location},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ErrorContext {
    pub start_end: Option<(Location, Location)>,
    pub file_name: Option<String>,
    pub file_content: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub context: Option<Box<ErrorContext>>,
}

impl Error {
    /// Set locations for this error, if they are `None`.
    /// Keeps already set locations.
    pub fn context_loc(self, start: Location, end: Location) -> Self {
        let mut context = self.context.unwrap_or_default();
        context.start_end.get_or_insert((start, end));

        Error {
            kind: self.kind,
            context: Some(context),
        }
    }

    /// Set file name for this error, if they are `None`.
    /// Keeps already set file name.
    pub fn context_file_name(self, file_name: String) -> Self {
        let mut context = self.context.unwrap_or_default();
        context.file_name.get_or_insert(file_name);

        Error {
            kind: self.kind,
            context: Some(context),
        }
    }

    /// Set file content for this error, if they are `None`.
    /// Keeps already set file contents.
    pub fn context_file_content(self, file_content: String) -> Self {
        let mut context = self.context.unwrap_or_default();
        context.file_content.get_or_insert(file_content);

        Error {
            kind: self.kind,
            context: Some(context),
        }
    }

    /// Set locations for this error, if they are `None`.
    /// Keeps already set locations.
    pub fn start(&self) -> Option<Location> {
        self.context
            .as_ref()
            .and_then(|c| c.start_end)
            .map(|se| se.0)
    }

    /// Set locations for this error, if they are `None`.
    /// Keeps already set locations.
    pub fn end(&self) -> Option<Location> {
        self.context
            .as_ref()
            .and_then(|c| c.start_end)
            .map(|se| se.1)
    }
}

impl From<InputParseError<'_>> for Error {
    fn from(e: InputParseError) -> Self {
        let max_location = *e.max_location();
        let max_location: Location = max_location.into();

        Error {
            kind: ErrorKind::ParseError(ErrorTreeFmt::new(e).to_string()),
            context: None,
        }
        .context_loc(
            max_location,
            Location {
                line: max_location.line,
                column: max_location.column + 1,
            },
        )
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error {
            kind: ErrorKind::Custom(msg.to_string()),
            context: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: any way to do this more elegantly?
        match self.context.as_ref() {
            Some(context) => match (
                context.start_end.as_ref(),
                context.file_name.as_ref(),
                context.file_content.as_ref(),
            ) {
                (Some((start, _)), _, _) => {
                    write!(f, "error at {}: {}", start, self.kind)
                }
                (_, _, _) => {
                    write!(f, "error: {}", self.kind)
                }
            },
            None => write!(f, "error: {}", self.kind),
        }
    }
}

impl std::error::Error for Error {}

pub fn print_error(e: &Error) -> std::io::Result<()> {
    use std::io::Write;

    let f = stdout();
    let mut f = f.lock();
    match e.context.as_ref() {
        Some(context) => match (
            context.start_end.as_ref(),
            context.file_name.as_ref(),
            context.file_content.as_ref(),
        ) {
            (Some((start, end)), file_name, Some(file_content)) => {
                let max_line_col_width = start.line.max(end.line).to_string().len();
                let col_ws_rep = " ".repeat(max_line_col_width);
                writeln!(f, "error: {}", e.kind)?;
                writeln!(
                    f,
                    "{}--> {}:{}:{}",
                    col_ws_rep,
                    file_name.map(AsRef::as_ref).unwrap_or("string"),
                    start.line,
                    start.column
                )?;

                writeln!(f, "{} |", col_ws_rep)?;
                let mut lines = file_content.lines().skip(start.line as usize - 1);
                let start_line_string = start.line.to_string();
                let start_line_padding = " ".repeat(max_line_col_width - start_line_string.len());

                if start.line == end.line {
                    // The first line
                    writeln!(
                        f,
                        "{}{} | {}",
                        start_line_padding,
                        start.line,
                        lines.next().unwrap_or_default()
                    )?;
                    // it's just one line, mark the whole span with ^
                    writeln!(
                        f,
                        "{} | {}{}",
                        col_ws_rep,
                        " ".repeat(start.column as usize - 1),
                        "^".repeat((end.column - start.column) as usize)
                    )?;
                } else {
                    // The first line
                    writeln!(
                        f,
                        "{}{} |   {}",
                        start_line_padding,
                        start.line,
                        lines.next().unwrap_or_default()
                    )?;
                    writeln!(
                        f,
                        "{} |  {}^",
                        col_ws_rep,
                        "_".repeat((start.column - 1) as usize),
                    )?;
                    for line_number in start.line + 1..=end.line {
                        let line_nr_string = line_number.to_string();
                        let line_padding = " ".repeat(max_line_col_width - line_nr_string.len());
                        writeln!(
                            f,
                            "{}{} | | {}",
                            line_padding,
                            line_nr_string,
                            lines.next().unwrap_or_default()
                        )?;
                    }

                    writeln!(
                        f,
                        "{} | |{}^",
                        col_ws_rep,
                        "_".repeat((end.column - 1) as usize)
                    )?;
                }

                writeln!(f, "{} |", col_ws_rep)
            }
            _ => writeln!(f, "{}", e),
        },
        _ => writeln!(f, "{}", e),
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    ExpectedBool,
    ExpectedString,
    ExpectedStrGotEscapes,
    ExpectedList,

    ParseError(String),

    Custom(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ExpectedBool => write!(f, "expected bool"),
            ErrorKind::ExpectedStrGotEscapes => {
                write!(f, "expected zero-copy string which doesn't support escapes")
            }
            ErrorKind::ExpectedString => write!(f, "expected string"),
            ErrorKind::ExpectedList => write!(f, "expected list"),
            ErrorKind::ParseError(e) => write!(f, "parsing error: {}", e),
            ErrorKind::Custom(s) => write!(f, "{}", s),
        }
    }
}
