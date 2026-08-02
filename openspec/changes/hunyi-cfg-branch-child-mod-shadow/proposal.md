## Why

`docs/audit/0.3.1-adversarial-sweep.md`'s "渾儀 cfg-branch merging" section records a false negative
at `crates/hunyi/src/exposure.rs:157` ([2/3], one dissenting lens): a governed module declaring
`#[cfg(unix)] mod serde;` beside `#[cfg(not(unix))] pub use serde::Value;` (or the identical shape
via `cfg_if! { if #[cfg(unix)] { mod serde; } else { pub use serde::Value; } }`) produces zero
findings under `must_not_expose("serde")`, even though under `cfg(not(unix))` the `pub use`
genuinely republishes the real extern crate `serde`'s type. Deleting the sibling `#[cfg(unix)] mod
serde;` makes the identical module react correctly — so the sibling declaration, which never
compiles alongside the `pub use` in any single build, is still shadowing it.

The dissenting lens argued this is a stated, spec'd bound: `semantic-reexport-exposure/spec.md`
resolves a bare re-export head against the external-crate set "with the governed module's own
child modules excluded," an explicit SHALL with a rustc-shadow rationale. Adjudicating that dissent
was this change's first job. Reading the whole spec file (not just the cited lines) confirms it
contains **zero mentions of `cfg`** anywhere — the child-module-exclusion rule is written, and its
every scenario constructed, for a `mod` and a `pub use` that coexist unconditionally in the same
build. The rule's own rustc rationale (a local child module wins name resolution over the extern
prelude) only holds when both declarations are actually compiled together; under mutually exclusive
`#[cfg]` arms, the `pub use`'s own real build never has the local `mod` present to shadow it. The
prior change that hardened this same resolver for cfg-branches, `change/hunyi-cfg-branch-use-reexport-merging`
(PR #149, `b353264`), explicitly named this exact line as **out of scope**, calling it "a different
mechanism (cfg-aware child-module-name partitioning, not a multi-valued map)" and "its own follow-up
change" — the project's own prior verdict already treated this as a genuine gap, not an accepted
bound. The dissent is therefore rejected; this is a real false negative and is fixed here.

Reproducing the trigger also surfaced an unnamed sibling: the identical cfg-blind child-module
computation is duplicated in the crate-wide re-export closure (`scan.rs`'s `collect_reexports`,
which a facade chain reaching the same shape through another module relies on) — the spec's own
"External-crate re-exports are observed by default" requirement already states the exclusion
"SHALL be applied both to the direct re-export head resolution and inside the crate-wide re-export
closure," so leaving the closure side cfg-blind while fixing only the direct head would satisfy
neither the letter nor the point of that existing sentence. Reproduced directly (a facade in
`crate::domain` re-exporting `crate::a::Value`, where `crate::a` carries the identical
mutually-exclusive `mod serde` / `pub use serde::Value` pair) and fixed alongside the direct case.

## What Changes

- `hunyi`'s child-module re-export shadow (`externs_reexport` in `module_findings`, and its
  crate-wide-closure mirror in `collect_reexports`) becomes **cfg-aware**: a same-named child `mod`
  no longer shadows a `pub use`'s bare extern head when the two are **provably mutually exclusive**
  — two arms of the identical `cfg_if!` invocation, or two bare `#[cfg(...)]` items whose predicates
  are syntactic negations of one another (`#[cfg(P)]` / `#[cfg(not(P))]`). Anything less
  syntactically direct (unrelated predicates, arms of two different `cfg_if!` invocations, more than
  one bare `#[cfg]` stacked on either side) stays the existing cfg-blind "may coexist" default — a
  stated, conservative residual bound, not a guess.
- No DSL change, no new boundary kind; this is a resolution-correctness fix behind the existing
  `must_not_expose` reaction, on the patch line per this project's SemVer-honesty rule (a
  behavior-changing bugfix that only makes the boundary react on a genuine leak it previously
  missed).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `semantic-reexport-exposure`: the "External-crate re-exports are observed by default" requirement's
  child-module-exclusion rule gains the cfg-mutual-exclusion carve-out described above, at both the
  direct head and the crate-wide closure, with new scenarios pinning the bare-`#[cfg]`-negation form,
  the `cfg_if!` form, and the closure/facade form.
- `semantic-signature-coupling`: the "Anchor resolution" requirement's existing cfg-branch scenarios
  (which already illustrate the branch-level child-module-shadow fix from `change/hunyi-cfg-branch-use-reexport-merging`)
  gain one new scenario for the item-level sibling case this change closes, so the requirement's own
  scenario set does not read as though the branch-level fix already covered it.

## Impact

- `crates/hunyi/src/syn_util.rs`: `FlatItem` gains an `cfg_if!` arm/invocation identity; new
  `provably_mutually_exclusive`, `reexport_externs_for`, and `child_module_decls` helpers.
- `crates/hunyi/src/module_resolve.rs`: new `resolve_module_items_with_cfg_tags`, alongside the
  existing `resolve_module_items_with_files` (unchanged, still used by every other capability).
- `crates/hunyi/src/exposure.rs`: `module_findings`'s `externs_reexport` becomes per-exposure-item
  rather than per-branch.
- `crates/hunyi/src/resolve/mod.rs`: `collect_reexports` becomes cfg-aware per re-export item.
- `crates/hunyi/src/scan.rs`: `walk_module` feeds `collect_reexports` the item-level cfg tags it
  already computes (`flat`) instead of the flattened, cfg-blind plain item list.
- No public API change; no manifest/package-version change beyond the normal patch bump.
