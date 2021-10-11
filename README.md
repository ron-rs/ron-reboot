# EXPERIMENT: Rusty Object Notation reboot

Experimental implementation of a new parser for RON using small functional parsers
for individual syntax elements over a stateful parser.

This experiment started off as "RON + [nom](https://github.com/Geal/nom)", and is still using `nom`
but is slowly replacing nom with its own combinators with the ultimate goal to get rid of it.

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

## Goals / Progress

| Goal                                             | Status                                         |
|--------------------------------------------------|------------------------------------------------|
| Parser generating AST                            | :hourglass_flowing_sand: working, 85% complete |
| Replace nom combinators                          | :heavy_check_mark: done                        |
| Spans in AST (locations for error reporting)     | :heavy_check_mark: implemented                 |
| Serde Deserializer using AST                     | :hourglass_flowing_sand: working, 70% complete |
| `ron-edit` (format & comments preserving writer) | :x: to be done                                 |
