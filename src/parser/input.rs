use std::{
    fmt::{Display, Formatter},
    slice::SliceIndex,
};

use crate::parser::{IResult, Input};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Offset {
    Absolute(usize),
    Relative(usize),
}

impl Offset {
    pub fn add(self, offset: usize) -> Self {
        match self {
            Offset::Absolute(x) => Offset::Absolute(x + offset),
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    pub line: u32,
    /// UTF-8 column
    pub column: u32,
}

impl<'a> From<Input<'a>> for Location {
    fn from(i: Input<'a>) -> Self {
        match i.offset {
            Offset::Absolute(offset) => {
                assert!(
                    i.input.is_char_boundary(offset),
                    "offset not at char boundary"
                );

                let line = i
                    .input
                    .bytes()
                    .take(offset)
                    .filter(|&b| b == b'\n')
                    .count()
                    + 1;

                let (byte_ind, char_ind, _c) = get_char_at_offset(i.input, offset);

                if byte_ind != offset {
                    println!("Input {:?}", i);
                    assert_eq!(byte_ind, offset, "offset not at char boundary");
                }

                let line_start = i
                    .input
                    .char_indices()
                    .take(char_ind)
                    .filter(|(_, c)| *c == '\n')
                    .map(|(i, _c)| i)
                    .last()
                    .unwrap_or(0);

                Location {
                    line: line as u32,
                    column: (char_ind - line_start + 1) as u32,
                }
            }
            Offset::Relative(_) => todo!(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LocatedSpan<'a> {
    offset: Offset,

    /// the complete input
    ///
    /// if `offset` is absolute
    input: &'a str,
    fragment: &'a str,
}

impl<'a> LocatedSpan<'a> {
    pub fn new(input: &'a str) -> Self {
        LocatedSpan {
            offset: Offset::Absolute(0),
            input,
            fragment: input,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fragment.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fragment.len()
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn offset_to(&self, other: &Self) -> usize {
        str_offset(self.fragment, other.fragment)
    }

    pub fn fragment(&self) -> &'a str {
        self.fragment
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + 'a {
        self.fragment.chars()
    }

    pub fn char_indices(&self) -> impl Iterator<Item = (usize, char)> + 'a {
        self.fragment.char_indices()
    }

    pub fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }

    pub fn slice(&self, range: impl SliceIndex<str, Output = str>) -> Self {
        let next_fragment = &self.fragment[range];
        let consumed_len = str_offset(self.fragment, next_fragment);
        if consumed_len == 0 {
            return LocatedSpan {
                offset: self.offset,
                input: self.input,
                fragment: next_fragment,
            };
        }
        let next_offset = self.offset.add(consumed_len);

        LocatedSpan {
            offset: next_offset,
            input: self.input,
            fragment: next_fragment,
        }
    }
}

impl<'a> Display for LocatedSpan<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "at {} (`{}`)",
            Location::from(*self),
            self.fragment.get(..1).unwrap_or("eof"),
        )
    }
}

pub fn position(input: Input) -> IResult<Input> {
    Ok(input.take_split(0))
}

/// returns (byte index, char index, char)
#[inline]
fn get_char_at_offset(input: &str, offset: usize) -> (usize, usize, char) {
    input
        .char_indices()
        .enumerate()
        .map(|(c_ind, (b_ind, c))| (b_ind, c_ind, c))
        // we now have an iterator of (byte index, char index, char)
        .find(|(i, _, _)| *i >= offset)
        .unwrap_or((input.len(), input.chars().count(), '�'))
}

/// Byte offset between string slices
fn str_offset(first: &str, second: &str) -> usize {
    let first = first.as_ptr();
    let second = second.as_ptr();

    second as usize - first as usize
}

#[cfg(test)]
mod tests {
    use crate::parser::input::get_char_at_offset;

    #[test]
    fn test_char_offset_basic() {
        assert_eq!(get_char_at_offset("123", 1), (1, 1, '2'));
    }

    #[test]
    fn test_char_offset_start() {
        assert_eq!(get_char_at_offset("123", 0), (0, 0, '1'));
    }

    #[test]
    fn test_char_offset_end() {
        assert_eq!(get_char_at_offset("123", 2), (2, 2, '3'));
    }

    #[test]
    fn test_char_offset_eof() {
        assert_eq!(get_char_at_offset("123", 3), (3, 3, '�'));
    }
}
