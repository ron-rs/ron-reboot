use std::{
    error::Error,
    fmt,
    fmt::{Debug, Display, Formatter},
};

use crate::utf8_parser::ErrorTree;

#[derive(Debug)]
pub struct ErrorTreeFmt(ErrorTree<String>);

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
