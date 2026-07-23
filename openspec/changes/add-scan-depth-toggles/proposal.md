# Proposal: Explicit Scan Depth Toggles (`ScanDepth`)

## Context

Tianheng's observation dimensions (`guibiao` static, `hunyi` semantic, `louke` runtime) observe crate and module boundaries to prevent architectural drift. Previously, boundary observation depth was binary or implicit: a boundary was either scanned at default depth or required specialized boolean modifiers. In complex codebases, deeper scanning on certain boundaries (such as recursive submodule traversal or private implementation inspection) could introduce false-positive noise in edge cases. Without an explicit mechanism to declare depth intent, some valuable architectural checks were historically `DECLINED` to avoid false positives in default scans.

## Intent

Introduce a first-class, structured `ScanDepth` enum (`Shallow`, `Subtree`, `Audit`) to the shared `xuanji` reaction model, and expose non-breaking `.depth(ScanDepth)` DSL methods on boundary builders across the crate family.

This change:
1. **Manifests Architectural Decisions**: Makes the decision of *how deep* to observe a first-class, explicit declaration in Rust code alongside `.because(...)`, which is projected into `AGENTS.self-law.md` and `list --format markdown`.
2. **Reclaims Previously Declined Capabilities**: Allows deeper or stricter observation checks to be offered as opt-in modes (`ScanDepth::Subtree` / `ScanDepth::Audit`), where the adopter explicitly endorses the depth selection.
3. **Preserves 100% Backward Compatibility**: Defaults to `ScanDepth::Shallow` (matching existing behavior), ensuring zero breaking changes for existing adopters, tests, or baseline snapshots in `0.3.x`.

## Capabilities Touched

- `rule-model-surface`: Add `ScanDepth` enum to `xuanji` and expose `.depth(ScanDepth)` builder APIs on boundary types while retaining ergonomic wrappers like `.including_submodules()`.

## Adversarial Review Stance

1. **Drift Law Compliance**: `xuanji` remains measure-only (`serde_json` dependency only, zero IO/observation engines). `ScanDepth` is a pure value type.
2. **Dimension Independence (三儀 ⊥ 三儀)**: Static engines (`guibiao` & `hunyi`) evaluate `ScanDepth` over static AST/metadata bounds. They do NOT execute runtime probes or pretend to perform runtime verification; runtime seam coverage remains `louke`'s domain (`audit_probe_coverage`).
3. **Zero Wire/Baseline Breakage**: `scan_depth` serialization uses `#[serde(default, skip_serializing_if = "is_shallow")]` so un-annotated baseline JSON snapshots and SARIF outputs parse without error or schema migration.

## Non-Goals

- No global magic sensitivity numbers (e.g. `sensitivity = 5`).
- No breaking changes to existing `0.3.0` public API or wire formats.
- No heavy compiler/proc-macro expansion passes inside `guibiao` (maintains the functional core `serde_json`-only invariant).
