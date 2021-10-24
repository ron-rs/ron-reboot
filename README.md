# EXPERIMENT: Rusty Object Notation reboot

Experimental implementation of a new parser for RON using small functional parsers
for individual syntax elements over a stateful parser.

This experiment started off as "RON + [nom](https://github.com/Geal/nom)", but is now using its own
parsers & combinators inspired by `nom`.

## Motivation

The current `ron` parsing suffers from the following problems:

* parsing is done in both `parse.rs` **and** the deserializer itself
    * bad code organization
    * hard to maintain
    * limits reuse
* serde's data model stops us from accurately reflecting struct / map and struct names

## Benefits

Stateless / functional parsers are

* easier to maintain
* easier to reuse
* much easier to test

An abstract syntax tree (AST)...

* makes the deserializer much easier & cleaner to implement
* allows reporting locations of syntax & type errors
* can be reused by multiple deserializer implementations (`serde::Deserializer`, `our_own::Deserializer`, `ron-edit`)

## Error reporting

The old RON deserializer produced errors that were often hard to understand.
`ron-reboot` is meant to change that; this is the output of a deserialization
error as of 2021-10-15:

```
error: invalid type: boolean `true`, expected a string
 --> string:3:9
  |
3 |       y: true,
  |          ^^^^
  |
```

or with a multi-line expression:

```
error: invalid type: map, expected a string
 --> string:3:9
  |
3 |       y: (
  |  ________^
4 | |         this: "is",
5 | |         not: "the right type",
6 | |     ),
  | |______^
  |
```

## Goals / Progress

| Goal                                             | Status                                             |
|--------------------------------------------------|----------------------------------------------------|
| Parser generating AST                            | :heavy_check_mark: done                            |
| Parser generating beautiful errors               | :heavy_check_mark: done                            |
| Accurate Value deserialization from AST          | :heavy_check_mark: done                            |
| Replace nom combinators                          | :heavy_check_mark: done                            |
| Spans in AST (locations for error reporting)     | :heavy_check_mark: done                            |
| Serde Deserializer using AST                     | :heavy_check_mark: done                            |
| Serde Deserializer generating beautiful errors   | :heavy_check_mark: done                            |
| `ron-edit` (format & comments preserving writer) | :x: to be done                                     |

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
shall be licensed according to the license of this project (The Unlicense), without any additional terms or conditions.
