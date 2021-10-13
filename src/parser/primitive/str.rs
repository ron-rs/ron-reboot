use crate::parser::{Input, IResultLookahead};
use crate::parser::basic::tag;
use crate::parser::combinators::{delimited, map, take_while};

fn inner_str(input: Input) -> IResultLookahead<&str> {
    map(take_while(|c| c != '"' && c != '\\'), |x: Input| {
        x.fragment()
    })(input)
}

pub fn unescaped_str(input: Input) -> IResultLookahead<&str> {
    delimited(tag("\""), inner_str, tag("\""))(input)
}
