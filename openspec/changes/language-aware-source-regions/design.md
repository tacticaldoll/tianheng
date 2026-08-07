## Context

Several repository reactions share `tests/support/region.rs`. Its `executed()` region strips both languages'
line-comment prefixes at once. The helper therefore cannot faithfully represent either language: `#` begins a
shell comment but a Rust attribute, while `//` begins a Rust comment but is ordinary shell text.

## Goals / Non-Goals

**Goals:**

- make the source language explicit at every executed-region call site;
- retain Rust attributes and exclude Rust `//` comments;
- exclude shell `#` comments without inventing Rust semantics for shell;
- keep the existing header, prose, and whole-source regions unchanged.

**Non-Goals:**

- parse either language completely;
- classify inline comments or text inside literals;
- change production scanners or published APIs.

## Decisions

### Expose `rust()` and `shell()` regions

`Source` will expose two named constructors backed by one `Executed` implementation carrying the applicable
line-comment marker. The language choice is visible and greppable at each recognizer, while the filtering
mechanism remains shared.

### Remove `executed()`

Keeping the language-blind escape would let new call sites reintroduce the ambiguity. Every current use has a
known source language, so migration is complete rather than compatibility-preserving; this is internal test
infrastructure and not an adopter break.

## Risks / Trade-offs

This remains a line-oriented region rather than a parser, so inline comment markers and literals stay within the
declared observation bounds of their owning reactions. The change only corrects the full-line language mismatch.

## Verification

A unit guard distinguishes Rust attributes, Rust comments, shell comments, and shell data containing `//`.
Each owning integration test then exercises its migrated recognizers, followed by the complete Definition of Done.
