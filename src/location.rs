use std::fmt::{Display, Formatter};

#[cfg(test)]
use crate::utf8_parser::test_util::TestMockNew;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Location {
    pub line: u32,
    /// UTF-8 column
    pub column: u32,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
impl TestMockNew for Location {
    fn new_mocked() -> Self {
        Location { line: 1, column: 1 }
    }
}
