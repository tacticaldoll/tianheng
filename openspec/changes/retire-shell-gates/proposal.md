## Why

`scripts/check_*.sh` shell gates (such as `check_bound_register.sh`, `check_reference_integrity.sh`, `check_whitespace_hygiene.sh`, `check_release_coherence.sh`, `check_dod_coherence.sh`, `check_pin_bites.sh`, `check_publish_source.sh`) create semantic confusion between bash scripts and true Rust self-governance reactions. They introduce fragile subshell status handling, require a separate Rust meta-harness (`gate-shape-contract`) to govern bash gate shapes, and obscure the single source of truth.

This change migrates all repository check gates to Rust-native `self-governance` tests (`crates/tianheng/tests/*.rs`) and formally classifies contract projections/censuses as derived text views (NOT reactions, NOT governance, and NOT shipped product code). Phase 1 converts check gates to Rust self-governance; Phase 2 fully retires `.sh` scripts, simplifies specs, and updates CI workflow.

## What Changes

- **Phase 1: Rust Self-Governance Migration**:
  - Convert `scripts/check_*.sh` validation logic into Rust `cargo test` gates under `crates/tianheng/tests/`.
  - Maintain projection staleness checks directly via Rust (`BLESS=1 cargo test`).
  - Explicitly document projections (derived markdown tables/censuses) as non-reaction, non-governance text views that do not ship in product crates.
- **Phase 2: Shell Gate Retirement & Spec Sync**:
  - Retire `scripts/check_*.sh` and helper bash scripts.
  - Simplify/retire `gate-shape-contract` requirements for bash script structural properties.
  - Update `.github/workflows/ci.yml` so CI pre-flight runs directly via `cargo test`.
  - Update `AGENTS.md` and `PROJECT.md` governance docs.

## Capabilities

### New Capabilities
- `rust-self-governance-gates`: Defines the Rust-native self-governance test gates (`crates/tianheng/tests/*.rs`) replacing shell check scripts.

### Modified Capabilities
- `gate-shape-contract`: Update/retire bash script shape requirements as check gates transition to Rust self-governance.
- `projection-register`: Reflect Rust test handlers for all generated documents.

## Impact

- **Code**: `crates/tianheng/tests/`, `scripts/`, `.github/workflows/ci.yml`.
- **Docs/Governance**: `AGENTS.md`, `PROJECT.md`, `docs/projection-register.md`, `docs/gate-shape-contract.md`.
- **Dependencies**: No external crate dependency additions.
