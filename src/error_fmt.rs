use std::{
    error::Error,
    fmt,
    fmt::{Debug, Display, Formatter},
};

use crate::parser::{ErrorTree, Input};

#[derive(Debug)]
pub struct ErrorTreeFmt(ErrorTree<String>);

impl ErrorTreeFmt {
    pub fn new(e: ErrorTree<Input<'_>>) -> Self {
        ErrorTreeFmt(e.map_locations(|input| {
            format!("{}:{}", input.location_line(), input.get_utf8_column())
        }))
    }
}

impl Display for ErrorTreeFmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ErrorTreeFmt {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}
