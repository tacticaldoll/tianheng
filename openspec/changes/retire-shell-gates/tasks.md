## 1. Phase 1: Rust Self-Governance Migration

- [x] 1.1 Convert `scripts/check_bound_register.sh` logic to `crates/tianheng/tests/bound_register.rs`
- [x] 1.2 Convert `scripts/check_reference_integrity.sh` logic to `crates/tianheng/tests/reference_integrity.rs`
- [x] 1.3 Convert `scripts/check_whitespace_hygiene.sh` logic to `crates/tianheng/tests/whitespace_hygiene.rs`
- [x] 1.4 Convert `scripts/check_release_coherence.sh` logic to `crates/tianheng/tests/release_coherence.rs`
- [x] 1.5 Convert `scripts/check_dod_coherence.sh` logic to `crates/tianheng/tests/dod_coherence.rs`
- [x] 1.6 Convert `scripts/check_pin_bites.sh` logic to `crates/tianheng/tests/pin_bites.rs`
- [x] 1.7 Convert `scripts/check_publish_source.sh` logic to `crates/tianheng/tests/publish_source.rs`
- [x] 1.8 Verify `cargo test -p tianheng` passes and all projection staleness checks hold via Rust

## 2. Phase 2: Shell Gate Retirement & Spec Sync

- [x] 2.1 Remove `scripts/check_*.sh` and helper bash scripts
- [x] 2.2 Update `gate-shape-contract` and `projection-register` specs and projection documents
- [x] 2.3 Update `.github/workflows/ci.yml` so CI jobs invoke `cargo test` directly
- [x] 2.4 Update `AGENTS.md` and `PROJECT.md` documentation to document Rust self-governance and non-product status of projections
