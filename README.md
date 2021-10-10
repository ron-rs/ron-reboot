# EXPERIMENT: Rusty Object Notation implementation with `nom`

[`nom`](https://github.com/Geal/nom) is a Rust parsing library. The intention of
this project is to evaluate whether RON could benefit from using this parser
over the
[rather error-prone, manual logic it is using right now](https://github.com/ron-rs/ron/blob/master/src/parse.rs).

## Motivation

The current `ron` parsing suffers from the following problems:

* parsing is done in both `parse.rs` **and** the deserializer itself
    * bad code organization
    * hard to maintain
    * limits reuse
* serde's data model stops us from accurately reflecting struct / map and struct names

## Goals

| Goal | Status |
|---|---|
| Parser generating AST | :hourglass_flowing_sand: working, 80% complete |
| Spans in AST (locations for error reporting) | :heavy_check_mark: implemented |
| Serde Deserializer using AST | :hourglass_flowing_sand: working, 20% complete |
| `ron-edit` (format & comments preserving writer) | :x: to be done |
