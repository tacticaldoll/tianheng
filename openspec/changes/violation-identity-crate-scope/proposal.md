## Why

`ViolationId { target, rule_key, fact }` never carries which crate produced it. `target` is a bare
module path (`"crate::app"`) and every fact family's fields (guibiao's `ModuleFact`, hunyi's
`SemanticFact`) name only path-shaped values, never the governing crate — even though every module
and semantic boundary is declared `in_crate(...)` and the same module path/rule shape is routinely
declared once per crate across a workspace (the identical shape Tianheng's own self-law uses:
`must_not_call_inline("std::fs")` on `crate` in guibiao, hunyi, and louke each). When two workspace
members carry the same governed module path under the same rule, their two, real, independent
violations collapse into one identity: guibiao's and hunyi's own report dedup
(`crates/guibiao/src/lib.rs`, `crates/hunyi/src/driver.rs::outcome_from`) and the shell's baseline
gate (`crates/tianheng/src/runner.rs`) all key on `Violation::id()`, so the second crate's violation
is silently dropped from the report — or worse, baseline-suppressed by an entry that was only ever
accepted for the first crate. This is the exact false-negative PROJECT.md's Core Contract forbids
(an enforce violation reaching exit 0), and it defeats `structured-violation-identity`'s own stated
requirement that "two observations differ in any identity-bearing observed value" only when their
identity actually captures that value — crate was never one of them.

## What Changes

- guibiao's module-boundary fact construction (`ModuleFact` in `crates/guibiao/src/finding.rs`,
  and the `target` passed into `push_module_violation` in `crates/guibiao/src/module_check.rs`)
  gains the governing crate as an identity-bearing field.
- hunyi's shared semantic-violation emission (`SemanticFact` in `crates/hunyi/src/finding.rs`, and
  `crates/hunyi/src/emit.rs`'s `push_single_module_violations` / `push_multi_module_violations`,
  which every module-path-scoped semantic capability funnels through) gains the same — **except**
  `unsafe_confinement`, whose `MultiModuleViolationContext` already sets `target: &boundary.
  crate_package` directly (confirmed: `crates/hunyi/src/unsafe_confinement.rs:55`), so it is
  already crate-scoped and adding a redundant field there would duplicate identity information
  rather than fix a gap.
- The new field is named `governing_package`, not `package` — guibiao's existing `CrateFact`
  family already uses `"package"` to mean the *observed dependency's* name (the object of a
  `must_depend_on`/confinement rule), a different referent from "the crate that declared this
  boundary." Reusing the same field name for two different meanings inside one identity model
  would be its own hazard.
- Each dimension's own published-fact compatibility catalog (`structured-violation-identity`'s
  "every shipped fact family cataloged" tests) is updated to the new field so the compatibility
  reaction actually proves the new discriminator, not just documents it.
- **BREAKING**: every existing baseline entry for a module or semantic boundary changes identity
  (a new required field), so a baseline written before this change no longer matches after it —
  every accepted violation reappears as new once, and a fresh `--write-baseline` is required. No
  DSL, builder, or CLI surface changes; only the identity `fact` payload gains a field.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `structured-violation-identity`: extends the existing "Distinct facts carry distinct identities"
  requirement with scenarios covering the cross-crate case, rather than adding a new requirement
  that would dictate specific dimensions' fact-field content — that capability's own Purpose
  already disclaims owning crate-/module-/semantic-specific fact vocabulary, so the delta stays at
  the level of "an identity-bearing observed value" (crate is one, when it varies) instead of
  naming a literal field name in the spec text.

## Impact

- Affected code: `crates/guibiao/src/finding.rs`, `crates/guibiao/src/module_check.rs`,
  `crates/hunyi/src/finding.rs`, `crates/hunyi/src/emit.rs`, each dimension's fact-compatibility
  test suite, and the cross-dimension conformance fixtures under
  `crates/tianheng/tests/` that pin this exact shape (self-governance's own
  `must_not_call_inline("std::fs")` boundary, declared identically on guibiao, hunyi, and louke).
- Affected data: any existing `--write-baseline` output for a module or semantic boundary — this is
  a `format: tianheng.baseline/structured-facts` document, not a public Rust type, but its contents
  become stale across this change (see BREAKING above). CHANGELOG's `[Unreleased]` entry states
  this migration explicitly.
- No public API/DSL/builder surface changes; `ViolationId`/`Violation`/`Baseline` types and their
  accessors are unchanged in shape, only in the values dimensions populate.
- Out of scope: louke's runtime-origin-assertion identity (a different, already-catalogued finding
  — `crates/louke/src/finding.rs:107`, absolute-path-in-identity — tracked as its own change) is not
  touched here; no cross-crate collision has been observed or reported for louke's fact family, and
  extending scope to it without an observed defect would be speculative.
