use crate::utf8_parser::test_util::TestMockNew;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    pub line: u32,
    /// UTF-8 column
    pub column: u32,
}

impl TestMockNew for Location {
    fn new_mocked() -> Self {
        Location { line: 1, column: 1 }
    }
}
