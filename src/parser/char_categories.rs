// From https://github.com/ron-rs/ron/blob/59df2d32422d2334176cdf76fbf40f07b17c2ad9/src/parse.rs

#![allow(non_upper_case_globals, unused)]

// We have the following char categories.
const INT_CHAR: u8 = 1 << 0; // [0-9A-Fa-f_]
const FLOAT_CHAR: u8 = 1 << 1; // [0-9\.Ee+-]
const IDENT_FIRST_CHAR: u8 = 1 << 2; // [A-Za-z_]
const IDENT_OTHER_CHAR: u8 = 1 << 3; // [A-Za-z_0-9]
const IDENT_RAW_CHAR: u8 = 1 << 4; // [A-Za-z_0-9\.+-]
const WHITESPACE_CHAR: u8 = 1 << 5; // [\n\t\r ]

// We encode each char as belonging to some number of these categories.
const DIGIT: u8 = INT_CHAR | FLOAT_CHAR | IDENT_OTHER_CHAR | IDENT_RAW_CHAR; // [0-9]
const ABCDF: u8 = INT_CHAR | IDENT_FIRST_CHAR | IDENT_OTHER_CHAR | IDENT_RAW_CHAR; // [ABCDFabcdf]
const UNDER: u8 = INT_CHAR | IDENT_FIRST_CHAR | IDENT_OTHER_CHAR | IDENT_RAW_CHAR; // [_]
const E____: u8 = INT_CHAR | FLOAT_CHAR | IDENT_FIRST_CHAR | IDENT_OTHER_CHAR | IDENT_RAW_CHAR; // [Ee]
const G2Z__: u8 = IDENT_FIRST_CHAR | IDENT_OTHER_CHAR | IDENT_RAW_CHAR; // [G-Zg-z]
const PUNCT: u8 = FLOAT_CHAR | IDENT_RAW_CHAR; // [\.+-]
const WS___: u8 = WHITESPACE_CHAR; // [\t\n\r ]
const _____: u8 = 0; // everything else

// Table of encodings, for fast predicates. (Non-ASCII and special chars are
// shown with '·' in the comment.)
#[rustfmt::skip]
const ENCODINGS: [u8; 256] = [
    /*                     0      1      2      3      4      5      6      7      8      9    */
    /*   0+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, WS___,
    /*  10+: ·········· */ WS___, _____, _____, WS___, _____, _____, _____, _____, _____, _____,
    /*  20+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /*  30+: ·· !"#$%&' */ _____, _____, WS___, _____, _____, _____, _____, _____, _____, _____,
    /*  40+: ()*+,-./01 */ _____, _____, _____, PUNCT, _____, PUNCT, PUNCT, _____, DIGIT, DIGIT,
    /*  50+: 23456789:; */ DIGIT, DIGIT, DIGIT, DIGIT, DIGIT, DIGIT, DIGIT, DIGIT, _____, _____,
    /*  60+: <=>?@ABCDE */ _____, _____, _____, _____, _____, ABCDF, ABCDF, ABCDF, ABCDF, E____,
    /*  70+: FGHIJKLMNO */ ABCDF, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__,
    /*  80+: PQRSTUVWZY */ G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__,
    /*  90+: Z[\]^_`abc */ G2Z__, _____, _____, _____, _____, UNDER, _____, ABCDF, ABCDF, ABCDF,
    /* 100+: defghijklm */ ABCDF, E____, ABCDF, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__,
    /* 110+: nopqrstuvw */ G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__, G2Z__,
    /* 120+: xyz{|}~··· */ G2Z__, G2Z__, G2Z__, _____, _____, _____, _____, _____, _____, _____,
    /* 130+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 140+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 150+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 160+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 170+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 180+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 190+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 200+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 210+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 220+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 230+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 240+: ·········· */ _____, _____, _____, _____, _____, _____, _____, _____, _____, _____,
    /* 250+: ·········· */ _____, _____, _____, _____, _____, _____
];

const fn is_int_char(c: u8) -> bool {
    ENCODINGS[c as usize] & INT_CHAR != 0
}

const fn is_float_char(c: u8) -> bool {
    ENCODINGS[c as usize] & FLOAT_CHAR != 0
}

pub const fn is_ident_first_char(c: u8) -> bool {
    ENCODINGS[c as usize] & IDENT_FIRST_CHAR != 0
}

pub const fn is_ident_other_char(c: u8) -> bool {
    ENCODINGS[c as usize] & IDENT_OTHER_CHAR != 0
}

const fn is_ident_raw_char(c: u8) -> bool {
    ENCODINGS[c as usize] & IDENT_RAW_CHAR != 0
}

const fn is_whitespace_char(c: u8) -> bool {
    ENCODINGS[c as usize] & WHITESPACE_CHAR != 0
}
