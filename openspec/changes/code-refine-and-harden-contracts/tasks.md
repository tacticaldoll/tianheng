# Tasks: Code Refine and Harden Contracts

## Tasks

- [ ] 1. **Sweep Crate Rustdocs**: Audit and purify module `//!` and item `///` docstrings across `xuanji`, `xingbiao`, `guibiao`, `hunyi`, `louke`, and `tianheng`.
- [ ] 2. **Audit & Fortify Test Reactions**: Verify that all capability invariants have backing test reactions in unit/conformance suites.
- [ ] 3. **Update Release Coherence**: Add adopter-facing maintenance entry under `[Unreleased]` in `CHANGELOG.md`.
- [ ] 4. **Run Pre-flight DoD Gates**: Execute workspace build, 3 clippy passes, rustfmt, workspace tests (`TIANHENG_WORKSPACE_TESTS=1`), rustdoc check, cargo deny, release coherence scripts, and example suite script.
