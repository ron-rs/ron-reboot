//! An error type, [`ErrorTree`], designed to retain much more useful
//! information about parse failures than the built-in nom error types.

use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use crate::{location::Location, utf8_parser::Input, util::write_pretty_list};

pub type InputParseError<'a> = ErrorTree<Input<'a>>;

#[derive(Debug)]
pub struct _PrivateConstructor {
    private: (),
}

#[derive(Debug)]
pub enum InputParseErr<'a> {
    /// The utf8_parser had an error (recoverable)
    Recoverable(InputParseError<'a>),
    /// The utf8_parser had an unrecoverable error: we got to the right
    /// branch and we know other branches won't work, so backtrack
    /// as fast as possible
    Fatal(InputParseError<'a>),
}

impl<'a> InputParseErr<'a> {
    pub fn recoverable(e: InputParseError<'a>) -> Self {
        InputParseErr::Recoverable(e)
    }

    pub fn fatal(e: InputParseError<'a>) -> Self {
        InputParseErr::Fatal(e)
    }

    pub fn is_recoverable(&self) -> bool {
        match self {
            InputParseErr::Recoverable(_) => true,
            InputParseErr::Fatal(_) => false,
        }
    }
}

impl<'a> Display for InputParseErr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InputParseErr::Recoverable(e) => write!(f, "{}", e),
            InputParseErr::Fatal(e) => write!(f, "{}", e),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Expectation {
    /// A string tag was expected.
    Tag(&'static str),

    /// A specific character was expected.
    Char(char),

    /// One of the chars in the str
    OneOfChars(&'static str),

    /// One of the chars in the str
    OneOfTags(&'static [&'static str]),

    /// One of the classes in the array
    OneOfExpectations(&'static [Self]),

    /// An ASCII letter (`[a-zA-Z]`) was expected.
    Alpha,

    /// A decimal digit (`[0-9]`) was expected.
    Digit,

    /// A decimal digit (`[1-9]`) was expected.
    DigitFirst,

    /// A hexadecimal digit (`[0-9a-fA-F]`) was expected.
    HexDigit,

    /// A hexadecimal digit (`[0-9a-fA-F]`) was expected.
    UnicodeHexSequence { got: u32 },

    /// An octal digit (`[0-7]`) was expected.
    OctDigit,

    /// An alphanumeric character (`[0-9a-zA-Z]`) was expected.
    AlphaNumeric,

    /// A space or tab was expected.
    Space,

    /// The ned of a raw string was expected.
    RawStringEnd,

    /// The closing */ of a block comment.
    BlockCommentEnd,

    /// A space, tab, newline, or carriage return was expected.
    Multispace,

    /// `"\r\n"` was expected.
    CrLf,

    /// Eof was expected.
    Eof,

    /// Expected something; ie, not Eof.
    Something,
}

impl Display for Expectation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Expectation::Tag(tag) => write!(f, "{:?}", tag),
            Expectation::Char(c) => write!(f, "{:?}", c),
            Expectation::OneOfChars(one_of) => {
                write_pretty_list(f, one_of.chars(), |f, c| write!(f, "{:?}", c))
            }
            Expectation::OneOfTags(one_of) => {
                write_pretty_list(f, one_of.iter(), |f, c| write!(f, "{:?}", c))
            }
            Expectation::OneOfExpectations(one_of) => {
                write_pretty_list(f, one_of.iter(), |f, c| write!(f, "{}", c))
            }
            Expectation::Alpha => write!(f, "an ascii letter"),
            Expectation::Digit => write!(f, "an ascii digit"),
            Expectation::DigitFirst => write!(f, "a non-zero ascii digit [1-9]"),
            Expectation::HexDigit => write!(f, "a hexadecimal digit"),
            Expectation::OctDigit => write!(f, "an octal digit"),
            Expectation::AlphaNumeric => write!(f, "an ascii alphanumeric character"),
            Expectation::Space => write!(f, "a space or tab"),
            Expectation::Multispace => write!(f, "whitespace"),
            Expectation::BlockCommentEnd => write!(f, "end of block comment (`*/`)"),
            Expectation::Eof => write!(f, "eof"),
            Expectation::CrLf => write!(f, "CRLF"),
            Expectation::Something => write!(f, "not eof"),
            Expectation::UnicodeHexSequence { got } => {
                write!(f, "a valid unicode hex sequence (got 0x{:X})", got)
            }
            Expectation::RawStringEnd => write!(f, "closing raw string sequence"),
        }
    }
}

/// These are the different specific things that can go wrong at a particular
/// location during a nom parse. Many of these are collected into an
/// [`ErrorTree`].
#[derive(Debug)]
pub enum BaseErrorKind {
    /// Something specific was expected, such as a specific
    /// [character][Expectation::Char] or any [digit](Expectation::Digit).
    /// See [`Expectation`] for details.
    Expected(Expectation),

    External(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for BaseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            BaseErrorKind::Expected(expectation) => write!(f, "expected {}", expectation),
            BaseErrorKind::External(ref err) => {
                writeln!(f, "external error:")?;
                write!(f, "{}", indent(err))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackContext {
    /// The [`context`][crate::parser_ext::ParserExt::context] combinator
    /// attached a message as context for a subparser error.
    Context(&'static str),
}

impl Display for StackContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            StackContext::Context(ctx) => write!(f, "could not match {:?}", ctx),
        }
    }
}

#[derive(Debug)]
pub enum ErrorTree<I> {
    /// A specific error event at a specific location. Often this will indicate
    /// that something like a tag or character was expected at that location.
    Base {
        /// The location of this error in the input
        location: I,

        /// The specific error that occurred
        kind: BaseErrorKind,
    },

    /// A stack indicates a chain of error contexts was provided. The stack
    /// should be read "backwards"; that is, errors *earlier* in the `Vec`
    /// occurred "sooner" (deeper in the call stack).
    Stack {
        /// The original error
        base: Box<Self>,

        /// Whether it was indicated that the "final" useful context has been pushed onto the stack
        finalized: bool,

        /// The stack of contexts attached to that error
        contexts: Vec<(I, StackContext)>,
    },

    /// A series of parsers were tried at the same location (for instance, via
    /// the `alt2` combinator) and all of them failed.
    Alt(Vec<Self>),
    // TODO: in a future version of nom-supreme, elaborate on the specific
    // type combinations here. For instance:
    // - Alt can only contain Stack or Base
    // - Stack has a single Base or Alt, followed by a series of contexts
    //   (Context or Kind)
}

impl<I> ErrorTree<I> {
    pub(crate) fn max_location(&self) -> &I
    where
        I: Ord,
    {
        match self {
            ErrorTree::Base { location, .. } => location,
            ErrorTree::Stack { base, .. } => base.max_location(),
            ErrorTree::Alt(v) => v.iter().map(ErrorTree::max_location).max().unwrap(),
        }
    }

    pub(crate) fn expected(location: I, expectation: Expectation) -> Self {
        ErrorTree::Base {
            location,
            kind: BaseErrorKind::Expected(expectation),
        }
    }

    pub(crate) fn alt(first: Self, second: Self) -> Self {
        match (first, second) {
            (ErrorTree::Alt(mut alt), ErrorTree::Alt(alt2)) => {
                alt.extend(alt2);
                ErrorTree::Alt(alt)
            }
            (ErrorTree::Alt(mut alt), x) | (x, ErrorTree::Alt(mut alt)) => {
                // TODO: should we preserve order?
                alt.push(x);
                ErrorTree::Alt(alt)
            }
            (first, second) => ErrorTree::Alt(vec![first, second]),
        }
    }

    fn map_locations_ref<T>(self, convert_location: &mut impl FnMut(I) -> T) -> ErrorTree<T> {
        match self {
            ErrorTree::Base { location, kind } => ErrorTree::Base {
                location: convert_location(location),
                kind,
            },
            ErrorTree::Stack {
                base,
                contexts,
                finalized,
            } => ErrorTree::Stack {
                base: Box::new(base.map_locations_ref(convert_location)),
                contexts: contexts
                    .into_iter()
                    .map(|(location, context)| (convert_location(location), context))
                    .collect(),
                finalized,
            },
            ErrorTree::Alt(siblings) => ErrorTree::Alt(
                siblings
                    .into_iter()
                    .map(|err| err.map_locations_ref(convert_location))
                    .collect(),
            ),
        }
    }

    pub(crate) fn map_locations<T>(self, mut convert_location: impl FnMut(I) -> T) -> ErrorTree<T> {
        self.map_locations_ref(&mut convert_location)
    }

    pub(crate) fn calc_locations(self) -> ErrorTree<Location>
    where
        I: Into<Location>,
    {
        self.map_locations(|i| i.into())
    }
}

impl<I: Display> Display for ErrorTree<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorTree::Base { location, kind } => write!(f, "{} at {:#}", kind, location),
            ErrorTree::Stack {
                contexts,
                base,
                finalized: _,
            } => {
                contexts.iter().rev().try_for_each(|(location, context)| {
                    writeln!(f, "{} at {:#} because", context, location)
                })?;

                write!(f, "{}", indent(base))
            }
            ErrorTree::Alt(siblings) => {
                writeln!(f, "none of these matched:")?;
                write!(
                    f,
                    "{}",
                    indent(
                        siblings
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" or\n")
                    )
                )
            }
        }
    }
}

impl<I: Display + Debug> Error for ErrorTree<I> {}

impl<I> ErrorTree<I> {
    /// Similar to append: Create a new error with some added context
    pub fn add_context(location: I, ctx: &'static str, final_context: bool, other: Self) -> Self {
        let context = (location, StackContext::Context(ctx));

        match other {
            // This is already a stack, so push on to it
            ErrorTree::Stack {
                mut contexts,
                base,
                finalized: false,
            } => {
                contexts.push(context);
                ErrorTree::Stack {
                    base,
                    contexts,
                    finalized: final_context,
                }
            }

            ErrorTree::Stack {
                finalized: true, ..
            } => other,

            // This isn't a stack, create a new stack
            base => ErrorTree::Stack {
                base: Box::new(base),
                contexts: vec![context],
                finalized: final_context,
            },
        }
    }
}

impl From<ErrorTree<Location>> for crate::error::Error {
    fn from(e: ErrorTree<Location>) -> Self {
        let max_location = *e.max_location();
        let max_location: Location = max_location.into();

        Self {
            kind: crate::error::ErrorKind::ParseError(e.to_string()),
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

impl From<InputParseError<'_>> for crate::error::Error {
    fn from(e: InputParseError) -> Self {
        e.calc_locations().into()
    }
}

pub struct Indented(String);

pub fn indent(display: impl Display) -> Indented {
    Indented(display.to_string())
}

impl Display for Indented {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = self.0.as_str();
        let mut need_indent = true;
        loop {
            match need_indent {
                // We don't need an indent. Scan for the end of the line
                false => match s.as_bytes().iter().position(|&b| b == b'\n') {
                    // No end of line in the input; write the entire string
                    None => break f.write_str(s),

                    // We can see the end of the line. Write up to and including
                    // that newline, then request an indent
                    Some(len) => {
                        let (head, tail) = s.split_at(len + 1);
                        f.write_str(head)?;
                        need_indent = true;
                        s = tail;
                    }
                },
                // We need an indent. Scan for the beginning of the next
                // non-empty line.
                true => match s.as_bytes().iter().position(|&b| b != b'\n') {
                    // No non-empty lines in input, write the entire string
                    None => break f.write_str(s),

                    // We can see the next non-empty line. Write up to the
                    // beginning of that line, then insert an indent, then
                    // continue.
                    Some(len) => {
                        let (head, tail) = s.split_at(len);
                        f.write_str(head)?;
                        f.write_str("    ")?;
                        need_indent = false;
                        s = tail;
                    }
                },
            }
        }
    }
}
