# proposal: Hunyi Dual-Backed Module Anchor Reaction

## Why

A plain `mod child;` backed by BOTH conventional forms at once (`child.rs` AND `child/mod.rs`) is a
genuine rustc compile error (E0761), independent of any `#[cfg]`. 圭表 and 漏刻 each react to it with
a constitution error; 渾儀's `locate_module_file` (`crates/hunyi/src/module_resolve.rs`) instead
returns the **first** form it probes — `child.rs` if present, otherwise `child/mod.rs` — and scans
only that one, silently.

This is not merely a missing constitution error. It is an exposure **false negative**, the one bug
the core contract forbids. Measured with a control/treatment probe on the same boundary
(`SemanticBoundary::in_crate(p).module("crate::child").must_not_expose("crate::forbidden::Thing")`):

| fixture | 渾儀 | 圭表 |
| --- | --- | --- |
| single `child.rs`, leak in it (control) | exit 1 | exit 1 |
| single `child.rs`, no leak (control) | exit 0 | — |
| dual-backed, leak only in `child/mod.rs` | **exit 0** | exit 2 |
| dual-backed, leak only in `child.rs` | exit 1 | exit 2 |

The controls establish that the same boundary does react when the leak sits in the form 渾儀 reads,
so `exit 0` on the third row is unambiguous: moving a forbidden exposure from `child.rs` into
`child/mod.rs` turns governance off. Whether a dual-backed module is governed at all currently
depends on which of the two files the author happened to write the violation in.

The composed `tianheng check` masks this, because 圭表 reaches exit 2 first on the same workspace.
The false negative is therefore reachable only by a **standalone 渾儀 consumer** — the consumer class
`examples/hunyi-standalone` dogfoods and that `BACKLOG.md`'s `xuanji`-sink entry names (Pacta, Modou).

`locate_module_file` backs both `resolve_module_items_with_files` (the anchored descent) and
`scan::resolve_child_modules` (the crate-wide and subtree walks), so the gap is shared by nearly
every semantic capability — signature-coupling's own crate-wide alias and extern scan included,
alongside visibility, dyn/impl-trait, async-exposure, trait-impl locality, forbidden marker, and
unsafe confinement — not signature-coupling's anchor path alone.

The false negative is also reachable on source that **compiles**: a `#[cfg]`-gated-off `mod x;` with
both files present is stripped by rustc before module resolution, so no E0761 fires, yet 圭表 and 漏刻
both already refuse to judge it (each places its ambiguity check ahead of its own absent-file
cfg-tolerance). The "nobody ships uncompilable source, record it as accepted debt" objection
therefore does not hold.

## What Changes

- Make `locate_module_file` in `crates/hunyi/src/module_resolve.rs` distinguish the ambiguous state
  from the two single-form states: when `child.rs` and `child/mod.rs` are both present, produce a
  constitution error instead of returning the flat form.
- Add the dual-backed error builder to `crates/hunyi/src/errors.rs`, naming both resolved paths and
  the exactly-one-file rule. No parity of wording is claimed: 圭表's and 漏刻's existing messages
  already differ from each other in three ways (single-quoted full module path vs backticked bare
  name, and a trailing rule clause present in one and absent in the other), so "parallel twin"
  language here would be choosing a side while sounding like agreement.
- Extend `crates/tianheng/tests/dual_backed_module_conformance.rs` from two dimensions to three, so
  the cross-dimension agreement is pinned rather than claimed. The four-state input space
  (`neither` / `flat` / `nested` / `both`) is exhaustively coverable, so the ledger — not code
  sharing — is the drift reaction for this convention.
- Add a `CHANGELOG.md` `[Unreleased]` entry (`### Fixed`), as release coherence requires during
  development.

## Capabilities

### Modified Capabilities

- `semantic-signature-coupling`: add a requirement that a resolvable-but-**ambiguous** module anchor
  (both conventional forms present) is a constitution error, never a silent first-form pick — the
  shared anchor-resolution property of every single-module-anchored semantic capability.

## Impact

- `crates/hunyi/src/module_resolve.rs`: `locate_module_file` return shape and its callers
  (`descend`, `crate::scan::resolve_child_modules`).
- `crates/hunyi/src/errors.rs`: one new error builder.
- `crates/hunyi/src/tests.rs`: unit coverage for all four states.
- `crates/tianheng/tests/dual_backed_module_conformance.rs`: 渾儀 joins the ledger.
- `CHANGELOG.md`: `[Unreleased]` entry.
- Non-breaking: no public API, DSL, or wire-format change. The adopter-facing effect is a new exit 2
  — reachable on source rustc refuses to compile (a live E0761 declaration) **and** on source it
  compiles cleanly (a `#[cfg]`-gated-off declaration with both files present, which rustc strips
  before module resolution). See `design.md` for why the gated case still reacts, and note that the
  static and runtime dimensions already refuse to judge it today.
