use crate::utf8_parser::{input::Input, IResultLookahead, InputParseError, OutputResult};

#[derive(Debug)]
pub struct IOk<'a, O> {
    pub remaining: Input<'a>,
    pub parsed: O,
    // TODO: is there a better way to do this?
    /// A resolvable error that got discarded because the parsed
    /// expression was optional (`opt`, `many0`, etc.).
    ///
    /// The parser says "I could've done more, but this is the problem I ran into."
    /// We forgive him by calling `forget_err` ;)
    pub discarded_error: Option<InputParseError<'a>>,
}

impl<'a, O> IOk<'a, O> {
    pub fn and_then<P, Q>(
        self,
        mut parser: impl FnMut(Input<'a>) -> IResultLookahead<'a, P>,
        map: impl FnOnce(O, P) -> Q,
    ) -> IResultLookahead<'a, Q> {
        let IOk {
            remaining,
            parsed,
            discarded_error,
        } = self;
        let res = parser(remaining);
        let IOk {
            remaining,
            parsed: parsed2,
            discarded_error,
        } = res.map(move |ok| ok.prepend_err(discarded_error))?;

        Ok(IOk {
            remaining,
            parsed: map(parsed, parsed2),
            discarded_error,
        })
    }

    /// Run the `parser` on the remaining input,
    /// calling `map` with the already parsed input and the result of `parser`.
    pub fn then_res<P, Q>(
        self,
        mut parser: impl FnMut(Input<'a>) -> IResultLookahead<'a, P>,
        map: impl FnOnce(O, IResultLookahead<'a, P>) -> IResultLookahead<'a, Q>,
    ) -> IResultLookahead<'a, Q> {
        let IOk {
            remaining,
            parsed,
            discarded_error,
        } = self;
        let res = parser(remaining);
        let res = res.map(move |ok| ok.prepend_err(discarded_error));

        map(parsed, res)
    }

    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> IOk<'a, P> {
        IOk {
            remaining: self.remaining,
            parsed: f(self.parsed),
            discarded_error: self.discarded_error,
        }
    }

    pub fn map_res<P>(self, f: impl FnOnce(O) -> OutputResult<'a, P>) -> IResultLookahead<'a, P> {
        Ok(IOk {
            remaining: self.remaining,
            parsed: f(self.parsed)?,
            discarded_error: self.discarded_error,
        })
    }

    pub fn replace<P>(self, parsed: P) -> IOk<'a, P> {
        IOk {
            remaining: self.remaining,
            parsed,
            discarded_error: self.discarded_error,
        }
    }

    pub fn forget_err(self) -> Self {
        IOk {
            remaining: self.remaining,
            parsed: self.parsed,
            discarded_error: None,
        }
    }

    pub fn prepend_err(self, discarded_error: Option<InputParseError<'a>>) -> Self {
        IOk {
            remaining: self.remaining,
            parsed: self.parsed,
            discarded_error: discarded_error.or(self.discarded_error),
        }
    }
}

impl<'a, O> From<(Input<'a>, O)> for IOk<'a, O> {
    fn from((remaining, parsed): (Input<'a>, O)) -> Self {
        IOk {
            remaining,
            parsed,
            discarded_error: None,
        }
    }
}
