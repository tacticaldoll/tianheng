## Why

The lexical guard for trait-object exposure scans only top-level files in `crates/tianheng/src`. Its justification
assumes a private nested module cannot expose an item, but a crate root can publicly re-export an item from that
module. Nesting therefore removes source from the reaction even when the item remains adopter-reachable.

## What Changes

- Scan every Rust source file recursively below the Tianheng crate's `src` directory.
- Remove the private-module premise check that attempted to justify a smaller corpus.
- Add a nested-source fixture proving a trait-object exposure cannot leave the corpus by moving below `src/`.

## Capabilities

### Modified Capabilities

- `observer-protocol`: the lexical no-trait-object reaction observes the complete recursive Rust source corpus.

## Impact

Repository governance becomes stricter over Tianheng's own source tree. Published APIs, manifests, package
versions, and adopter-facing reaction behavior are unchanged.
