## Context

Every directory under `examples/` is its own Cargo workspace. This isolation is load-bearing: the
examples contain deliberate architectural drift so Tianheng can react, and therefore must not join
the governed root workspace. The root Definition of Done consequently does not format, lint, or
document their Rust targets. `scripts/test_examples.sh` already enters each workspace, constructs
the correct local `[patch.crates-io]` arguments, and runs its tests and visible reaction.

The six workspaces contain ordinary compilable Rust even when Tianheng judges their architecture
red. A manual Clippy invocation against the capability catalog found a real warning, proving the
quality-observation gap rather than merely predicting one.

## Goals / Non-Goals

**Goals:**

- Apply format, all-target Clippy, and rustdoc warning gates to every isolated example workspace.
- Reuse the exact dependency resolution already used by each example's tests.
- Keep one examples entry point for local DoD and CI.
- Preserve every deliberate Tianheng violation and existing reaction assertion.

**Non-Goals:**

- Add examples to the root Cargo workspace or Tianheng self-constitution.
- Turn architectural violations into Rust lint suppressions or exempt an example from quality.
- Build a separate GitHub Actions matrix or generic workspace-discovery framework.
- Change committed adopter-facing dependency declarations, public APIs, or package versions.

## Decisions

### Extend the existing per-example execution block

Add one small `quality_gates` shell helper to `scripts/test_examples.sh`. After each workspace has
constructed its existing `PATCH` arguments, call the helper before its tests. The helper runs:

1. `cargo fmt --all --check`;
2. `cargo clippy --all-targets ... -- -D warnings`;
3. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps ...`.

Clippy and rustdoc receive the same local patch arguments as the subsequent test/run commands.
Format needs no dependency resolution. Keeping the calls beside the existing per-example blocks
avoids a second list of workspace-to-family dependency mappings.

### All six example workspaces are in scope

The deliberate faults are architectural facts—an import direction, exposed type, misplaced
unsafe, or runtime crossing—not invalid or intentionally low-quality Rust. Every package target,
binary, and test therefore remains eligible for Clippy and rustdoc. `--all-targets` is required so
the test code that carries structured reaction assertions cannot rot outside the lint gate.

### Preserve adopter-honest manifests

Standalone examples continue to commit crates.io version requirements. Quality commands use Cargo
`--config patch.crates-io.*.path` arguments at execution time, exactly like the existing tests. No
manifest is rewritten to a local path and no example joins the root workspace.

### Repair only findings made observable by the new gate

The first live matrix may expose formatting or lint debt that predates this change. Apply only the
mechanical format and focused lint repairs needed to make the new invariant true; do not polish
example narratives or alter their deliberate boundary faults in the same lifecycle.

### Prove fail-fast ordering with an ephemeral warning fixture

A focused shell test will create a minimal temporary Cargo workspace containing one deterministic
Clippy warning, invoke the same quality helper, and assert both a non-zero result and the absence of
a subsequent reaction sentinel. The fixture is ephemeral so it cannot become a seventh product
example or another manifest list to maintain. This proves that a warning blocks reaction acceptance
rather than relying only on the six positive rows.

## Risks / Trade-offs

- **Examples CI takes longer.** → Reuse Cargo caches and one sequential script; do not duplicate the
  work in a second workflow matrix.
- **A deliberate architectural fault is mistaken for lint debt.** → Rust quality gates do not run
  Tianheng or repair boundary direction; the existing reaction follows afterward and must still be
  red as declared.
- **A new example is added without quality checks.** → The existing examples script is already the
  review and CI insertion point; future work may add discovery only if this manual list actually
  drifts.
- **rustdoc writes generated artifacts.** → Generated target directories remain ignored and
  `--no-deps` bounds work to the example package.
- **The negative test accidentally depends on existing example debt.** → Generate one focused,
  deterministic warning in a temporary workspace and remove it with a trap.
