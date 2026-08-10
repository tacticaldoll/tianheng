## Why

The Definition-of-Done coherence reaction silently exempts `cargo deny check`, so removing CI's supply-chain job leaves AGENTS.md claiming a strict subset while the reaction still passes. The CI action already declares the effective command; the reaction needs to observe it instead of skipping the contract.

## What Changes

- Treat `EmbarkStudios/cargo-deny-action` with `command: check` as the effective CI command `cargo deny check`.
- Remove the hardcoded DoD exemption.
- Add a failure direction proving a missing or misconfigured supply-chain action is visible.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rust-repository-reactions`: require DoD-to-CI coherence to compare effective commands supplied by a supported action rather than silently skipping them.

## Impact

- Affects only the unpublished repository reaction in `crates/kanhe/tests/dod_coherence.rs` and its capability spec.
- Does not change CI configuration, published crates, manifests, versions, or adopter behavior.
