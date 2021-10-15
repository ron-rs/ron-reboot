use crate::utf8_parser::{
    basic::tag,
    combinators::{delimited, map, take_while},
    IResultLookahead, Input,
};

fn inner_str(input: Input) -> IResultLookahead<&str> {
    map(take_while(|c| c != '"' && c != '\\'), |x: Input| {
        x.fragment()
    })(input)
}

pub fn unescaped_str(input: Input) -> IResultLookahead<&str> {
    delimited(tag("\""), inner_str, tag("\""))(input)
}
