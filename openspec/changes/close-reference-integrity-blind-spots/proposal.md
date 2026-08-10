## Why

Reference integrity claims to hold repository paths named by documents and comments, but it currently ignores tracked TOML and `.gitignore` comments and skips every line in Rust test sources. Live references to deleted workflow scripts therefore remain green, while its spec also promises exit/stdout behavior a cargo-test reaction cannot provide.

## What Changes

- Extend the inspected tracked corpus to Markdown, Rust, TOML, and `.gitignore` files.
- Remove the test-source-wide skip and retain only shape-specific fixture exclusions.
- Count each inspectable file once and repair stale live references revealed by the widened corpus.
- Rewrite the capability's retired 0/1/2 and stdout vocabulary as observable cargo-test pass/fail behavior while preserving read-only judgment.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reference-integrity`: widen its declared corpus, narrow fixture exclusions, and align its reaction contract with the Rust test that implements it.

## Impact

- Affects `crates/kanhe/tests/reference_integrity.rs`, stale comments it newly observes, and `openspec/specs/reference-integrity/spec.md`.
- Changes unpublished repository governance only; no published crate API, manifest, version, or adopter migration changes.
