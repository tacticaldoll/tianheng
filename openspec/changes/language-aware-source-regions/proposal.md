## Why

The shared test-region helper currently defines "executed text" by dropping both `//` and `#` lines for every
source language. That hides Rust attributes such as `#[cfg(...)]` and treats shell text with Rust's comment rule,
so a reaction can silently exclude executable source before its recognizer runs.

## What Changes

- Replace the language-blind executed region with explicit Rust and shell regions.
- Migrate each governance reaction to select the region matching the file it reads.
- Add a guard proving Rust attributes remain executable while shell comments remain excluded.

## Capabilities

### Modified Capabilities

- `gate-shape-contract`: shell gate properties are read through shell comment semantics.
- `observer-protocol`: Rust observer bodies are read through Rust comment semantics.
- `projection-register`: Rust and shell holders use their respective executed regions.

## Impact

This changes only Tianheng's repository test infrastructure and governance reactions. It does not alter a
published crate API, manifest, package version, or adopter output.
