//! An error type, [`ErrorTree`], designed to retain much more useful
//! information about parse failures than the built-in nom error types.

use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter, Write},
};

use indent_write::fmt::IndentWriter;
use nom::{
    error::{ContextError, ErrorKind as NomErrorKind, FromExternalError, ParseError},
    InputLength,
};
use nom_supreme::tag::TagError;

use crate::util::write_pretty_list;

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

    /// An octal digit (`[0-7]`) was expected.
    OctDigit,

    /// An alphanumeric character (`[0-9a-zA-Z]`) was expected.
    AlphaNumeric,

    /// A space or tab was expected.
    Space,

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
            Expectation::Eof => write!(f, "eof"),
            Expectation::CrLf => write!(f, "CRLF"),
            Expectation::Something => write!(f, "not eof"),
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

    /// A nom parser failed.
    Kind(NomErrorKind),

    /// An error outside of nom occurred during parsing; for instance, as a
    /// result of an error during [`map_res`].
    ///
    /// [`map_res`]: crate::parser_ext::ParserExt::map_res
    External(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for BaseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            BaseErrorKind::Expected(expectation) => write!(f, "expected {}", expectation),
            BaseErrorKind::External(ref err) => {
                writeln!(f, "external error:")?;
                let mut f = IndentWriter::new("  ", f);
                write!(f, "{}", err)
            }
            BaseErrorKind::Kind(kind) => write!(f, "error in {:?}", kind),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackContext {
    /// A nom combinator attached an [`ErrorKind`][NomErrorKind] as context
    /// for a subparser error.
    Kind(NomErrorKind),

    /// The [`context`][crate::parser_ext::ParserExt::context] combinator
    /// attached a message as context for a subparser error.
    Context(&'static str),
}

impl Display for StackContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            StackContext::Kind(kind) => write!(f, "while parsing {:?}", kind),
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

        /// The stack of contexts attached to that error
        contexts: Vec<(I, StackContext)>,
    },

    /// A series of parsers were tried at the same location (for instance, via
    /// the [`alt`](nom::branch::alt) combinator) and all of them failed. All
    /// of the errors in this set are "siblings".
    Alt(Vec<Self>),
    // TODO: in a future version of nom-supreme, elaborate on the specific
    // type combinations here. For instance:
    // - Alt can only contain Stack or Base
    // - Stack has a single Base or Alt, followed by a series of contexts
    //   (Context or Kind)
}

impl<I> ErrorTree<I> {
    pub fn expected(location: I, expectation: Expectation) -> Self {
        ErrorTree::Base {
            location,
            kind: BaseErrorKind::Expected(expectation),
        }
    }

    pub fn alt(first: Self, second: Self) -> Self {
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
            ErrorTree::Stack { base, contexts } => ErrorTree::Stack {
                base: Box::new(base.map_locations_ref(convert_location)),
                contexts: contexts
                    .into_iter()
                    .map(|(location, context)| (convert_location(location), context))
                    .collect(),
            },
            ErrorTree::Alt(siblings) => ErrorTree::Alt(
                siblings
                    .into_iter()
                    .map(|err| err.map_locations_ref(convert_location))
                    .collect(),
            ),
        }
    }

    pub fn map_locations<T>(self, mut convert_location: impl FnMut(I) -> T) -> ErrorTree<T> {
        self.map_locations_ref(&mut convert_location)
    }
}

impl<I: Display> Display for ErrorTree<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorTree::Base { location, kind } => write!(f, "{} at {:#}", kind, location),
            ErrorTree::Stack { contexts, base } => {
                contexts.iter().rev().try_for_each(|(location, context)| {
                    writeln!(f, "{} at {:#} because", context, location)
                })?;

                let mut f = IndentWriter::new("  ", f);
                write!(f, "{}", base)
            }
            ErrorTree::Alt(siblings) => {
                writeln!(f, "none of these matched:")?;
                let mut f = IndentWriter::new("  ", f);
                write!(
                    f,
                    "{}",
                    siblings
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" or\n")
                )
            }
        }
    }
}

impl<I: Display + Debug> Error for ErrorTree<I> {}

impl<I: InputLength> ParseError<I> for ErrorTree<I> {
    /// Create a new error at the given position. Interpret `kind` as an
    /// [`Expectation`] if possible, to give a more informative error message.
    fn from_error_kind(location: I, kind: NomErrorKind) -> Self {
        let kind = match kind {
            NomErrorKind::Alpha => BaseErrorKind::Expected(Expectation::Alpha),
            NomErrorKind::Digit => BaseErrorKind::Expected(Expectation::Digit),
            NomErrorKind::HexDigit => BaseErrorKind::Expected(Expectation::HexDigit),
            NomErrorKind::OctDigit => BaseErrorKind::Expected(Expectation::OctDigit),
            NomErrorKind::AlphaNumeric => BaseErrorKind::Expected(Expectation::AlphaNumeric),
            NomErrorKind::Space => BaseErrorKind::Expected(Expectation::Space),
            NomErrorKind::MultiSpace => BaseErrorKind::Expected(Expectation::Multispace),
            NomErrorKind::CrLf => BaseErrorKind::Expected(Expectation::CrLf),

            // Problem: ErrorKind::Eof is used interchangeably by various nom
            // parsers to mean either "expected Eof" or "expected NOT eof". See
            // https://github.com/Geal/nom/issues/1259. For now, we examine the
            // input string to guess what the likely intention is.
            NomErrorKind::Eof => match location.input_len() {
                // The input is at Eof, which means that this refers to an
                // *unexpected* eof.
                0 => BaseErrorKind::Expected(Expectation::Something),

                // The input is *not* at eof, which means that this refers to
                // an *expected* eof.
                _ => BaseErrorKind::Expected(Expectation::Eof),
            },
            kind => BaseErrorKind::Kind(kind),
        };

        ErrorTree::Base { location, kind }
    }

    /// Combine an existing error with a new one. This is how error context is
    /// accumulated when backtracing. "other" is the original error, and the
    /// inputs new error from higher in the call stack.
    ///
    /// If `other` is already an `ErrorTree::Stack`, the context is added to
    /// the stack; otherwise, a new stack is created, with `other` at the root.
    fn append(location: I, kind: NomErrorKind, other: Self) -> Self {
        let context = (location, StackContext::Kind(kind));

        match other {
            // Don't create a stack of [ErrorKind::Alt, ErrorTree::Alt(..)]
            alt @ ErrorTree::Alt(..) if kind == NomErrorKind::Alt => alt,

            // This is already a stack, so push on to it
            ErrorTree::Stack { mut contexts, base } => {
                contexts.push(context);
                ErrorTree::Stack { base, contexts }
            }

            // This isn't a stack; create a new stack
            base => ErrorTree::Stack {
                base: Box::new(base),
                contexts: vec![context],
            },
        }
    }

    /// Create an error indicating an expected character at a given position
    fn from_char(location: I, character: char) -> Self {
        ErrorTree::Base {
            location,
            kind: BaseErrorKind::Expected(Expectation::Char(character)),
        }
    }

    /// Combine two errors from branches of alt. If either or both errors are
    /// already [`ErrorTree::Alt`], the different error sets are merged;
    /// otherwise, a new [`ErrorTree::Alt`] is created, containing both
    /// `self` and `other`.
    fn or(self, other: Self) -> Self {
        // For now we assume that there's no need to try and preserve
        // left-to-right ordering of alternatives.
        let siblings = match (self, other) {
            (ErrorTree::Alt(mut siblings1), ErrorTree::Alt(mut siblings2)) => {
                match siblings1.capacity() >= siblings2.capacity() {
                    true => {
                        siblings1.extend(siblings2);
                        siblings1
                    }
                    false => {
                        siblings2.extend(siblings1);
                        siblings2
                    }
                }
            }
            (ErrorTree::Alt(mut siblings), err) | (err, ErrorTree::Alt(mut siblings)) => {
                siblings.push(err);
                siblings
            }
            (err1, err2) => vec![err1, err2],
        };

        ErrorTree::Alt(siblings)
    }
}

impl<I> ContextError<I> for ErrorTree<I> {
    /// Similar to append: Create a new error with some added context
    fn add_context(location: I, ctx: &'static str, other: Self) -> Self {
        let context = (location, StackContext::Context(ctx));

        match other {
            // This is already a stack, so push on to it
            ErrorTree::Stack { mut contexts, base } => {
                contexts.push(context);
                ErrorTree::Stack { base, contexts }
            }

            // This isn't a stack, create a new stack
            base => ErrorTree::Stack {
                base: Box::new(base),
                contexts: vec![context],
            },
        }
    }
}

impl<I, E: Error + Send + Sync + 'static> FromExternalError<I, E> for ErrorTree<I> {
    /// Create an error from a given external error, such as from FromStr
    fn from_external_error(location: I, _kind: NomErrorKind, e: E) -> Self {
        ErrorTree::Base {
            location,
            kind: BaseErrorKind::External(Box::new(e)),
        }
    }
}

impl<I> TagError<I, &'static str> for ErrorTree<I> {
    fn from_tag(location: I, tag: &'static str) -> Self {
        ErrorTree::Base {
            location,
            kind: BaseErrorKind::Expected(match tag {
                "\r\n" => Expectation::CrLf,
                tag => Expectation::Tag(tag),
            }),
        }
    }
}
