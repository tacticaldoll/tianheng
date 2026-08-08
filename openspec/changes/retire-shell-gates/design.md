## Context

Currently, repository self-governance relies on a mix of Rust `cargo test` gates and Bash scripts under `scripts/check_*.sh`. The Bash gates check release coherence, whitespace hygiene, reference integrity, bound register freshness, etc. However, Bash gates suffer from subshell status swallowing, lack of type safety, and require a dedicated Rust meta-harness (`gate-shape-contract`) to enforce structural bash rules.

Furthermore, there has been semantic ambiguity regarding contract projections (derived markdown tables and censuses). Projections are text views generated for human and LLM context, but they are NOT reactions, NOT governance, and NOT product code.

## Goals / Non-Goals

**Goals:**
- Execute a two-stage migration:
  - **Stage 1**: Convert `scripts/check_*.sh` validation gates into Rust `cargo test` gates under `crates/tianheng/tests/`.
  - **Stage 2**: Completely retire `.sh` check scripts, update OpenSpec specs (including `gate-shape-contract` and `projection-register`), update `.github/workflows/ci.yml`, and update governance documentation (`AGENTS.md`, `PROJECT.md`).
- Explicitly define and document the nature of projections/censuses: derived text views that are NOT reactions, NOT governance, and NOT shipped products.

**Non-Goals:**
- Modifying shipped product API signatures in `xuanji`, `guibiao`, `hunyi`, `louke`, or `tianheng`.
- Adding new external crate dependencies.

## Decisions

### Decision 1: Two-Stage Phased Migration
- **Rationale**: Stage 1 converts checking logic to Rust `cargo test` gates so that `cargo test` becomes the sole pre-flight engine. Stage 2 removes legacy `.sh` files, cleans up CI workflow dependencies, and updates OpenSpec delta specs.
- **Alternatives Considered**: Direct single-step deletion of `.sh` scripts. Rejected because incremental verification in Stage 1 ensures test parity before retiring scripts.

### Decision 2: Rust Self-Governance Gate Location
- **Rationale**: Place new test gates in `crates/tianheng/tests/` (e.g., `bound_register.rs`, `reference_integrity.rs`, `whitespace_hygiene.rs`, `release_coherence.rs`, `dod_coherence.rs`, `pin_bites.rs`, `publish_source.rs`). These tests execute during `cargo test -p tianheng` and check workspace compliance.
- **Alternatives Considered**: Creating a new workspace crate. Rejected because `crates/tianheng/tests/` is already the established location for self-governance reactions (`self_governance.rs`, `gate_shape_contract.rs`, `projection_register.rs`).

### Decision 3: Explicit Boundary Definition for Projections
- **Rationale**: Formally document that projections (e.g., `AGENTS.self-law.md`, `docs/observation-bounds.md`) are generated censuses ("A census is produced, never typed"). Projections are text context, not reactions, not governance, and not shipped code.

## Risks / Trade-offs

- **[Risk]**: `check_publish_source.sh` runs at publish time from `publish.sh`.
  → **Mitigation**: `publish.sh` can call `cargo test -p tianheng --test publish_source` or run the Rust publish source verification.
- **[Risk]**: CI job reliance on shell script names.
  → **Mitigation**: Update `.github/workflows/ci.yml` in Stage 2 to invoke `cargo test` directly for all DoD and coherence checks.
