## Why

Tianheng's six examples are intentionally isolated from the root workspace so architectural faults
remain scan targets rather than self-governance violations. That isolation also hides their Rust
code from the root fmt, Clippy, and rustdoc gates; a manual catalog-local Clippy run already found
a warning the normal Definition of Done could not observe.

## What Changes

- Run format, all-target Clippy, and warning-denied rustdoc checks for every isolated example
  workspace through the existing examples gate.
- Reuse each example's existing local patch resolution so quality checks exercise the in-development
  Tianheng crates while committed manifests remain adopter-honest.
- Keep architectural violations intact: the matrix enforces Rust/code quality, then the existing
  tests prove each declared Tianheng reaction.
- Repair the pre-existing isolated-example formatting and lint findings exposed when the gate first
  becomes live.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `governance-dogfood`: Require every isolated example workspace to pass repository-owned Rust
  quality gates in addition to its existing reaction contract.

## Impact

The change affects the examples gate, isolated example source formatting/lints, CI runtime, the
governance-dogfood specification, and repository hygiene documentation. It changes no public API,
evaluator, violation identity, baseline/report wire, dependency declaration, or package version.
