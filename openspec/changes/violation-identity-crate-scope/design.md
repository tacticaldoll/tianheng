## Context

Verified directly against the current tree (not just the audit doc's paraphrase):

- `crates/xuanji/src/violation.rs`: `Violation::id()` returns `ViolationId { target, rule_key, fact }`
  and this is the sole equality/ordering key used for report dedup
  (`crates/guibiao/src/lib.rs`'s inline dedup loop, `crates/hunyi/src/driver.rs::outcome_from`) and
  for baseline matching (`violation-baseline` spec: "identified by its governed target, semantic
  rule key, and structured fact identity").
- guibiao's `target` is always a bare module path (`&governed_module`, e.g. `"crate::app"`,
  `crates/guibiao/src/module_check.rs:265`), and its `ModuleFact` variants
  (`crates/guibiao/src/finding.rs:92-140`) build `StructuredFactIdentity` fields from path-shaped
  strings only (`module`, `path`) — never the crate.
- hunyi's shared emission point `crates/hunyi/src/emit.rs::push_single_module_violations` /
  `push_multi_module_violations` builds `ViolationId::new(context.module, ...)` the same way; every
  semantic capability (`visibility.rs`, `exposure.rs`, `async_exposure.rs`, `dyn_trait.rs`,
  `impl_trait.rs`, …) funnels through it.
- Critically, **every one of those call sites already holds `&boundary.crate_package` in scope**
  (confirmed in `async_exposure.rs`, `impl_trait.rs`, and mirrored across the other capability
  files) — this is a threading gap, not a missing lookup. The package name is available everywhere
  the fix needs it; it is simply never passed the last step into the fact/context builder.
- `structured-violation-identity`'s own requirement text already forbids the wrong fix: "Separately
  observed identity-bearing components SHALL remain fact-specific named fields rather than being
  concatenated into an opaque display string" — so the package name must become its own named
  field, not get appended into `target`'s string.
- Existing convention: `CrateFact::dependency`/`CrateFact::feature` (guibiao's crate-dependency
  fact family, same file) already key a field literally named `"package"` — e.g.
  `[("kind", label), ("package", "serde")]`. The new field should match this vocabulary rather than
  invent a second name (`"crate"`) for the same concept.

## Goals / Non-Goals

**Goals:**
- Every guibiao `ModuleFact` variant and every hunyi `SemanticFact` variant carries a `"package"`
  field equal to `boundary.crate_package`, so two crates declaring the identical module
  path + rule produce distinct `ViolationId`s.
- Each dimension's fact-compatibility catalog test is updated to assert the new field (satisfying
  `structured-violation-identity`'s "adding a fact or finite discriminator requires an explicit
  catalog decision" requirement) rather than only fixing production code.
- A regression test reproduces the exact two-crate collision shape from the audit finding (mirrors
  `crates/tianheng/tests/self_governance.rs`'s own `must_not_call_inline("std::fs")` boundary,
  declared identically on guibiao, hunyi, and louke) and shows two distinct violations survive.

**Non-Goals:**
- louke's runtime-origin-assertion identity (a separately tracked, already-catalogued finding —
  `crates/louke/src/finding.rs:107`) — no observed cross-crate collision there; not touched.
- Any xuanji (`StructuredFactIdentity`/`ViolationId`) schema change — `fields: BTreeMap<String,
  String>` already accepts an arbitrary named field; nothing in the shared model needs to change.
- Any adopter-facing DSL/builder surface change — `ModuleBoundary`/`SemanticBoundary`/`in_crate(...)`
  already carry the package name; this only changes what dimensions do with data they already hold.
- Automatic baseline migration — an outdated baseline is a stale document under the new identity,
  not a format this change teaches the parser to upgrade (see Risks).

## Decisions

- **Field name: `"package"`, not `"crate"`.** Matches the existing `CrateFact` vocabulary in the
  same file family rather than introducing a synonym; also avoids ambiguity with target's own
  `"crate::..."` path spelling.
- **Value: the exact `boundary.crate_package` string**, not a re-resolved crate name from
  `cargo_metadata`. It is what the adopter wrote in `.in_crate(...)`, it is already the value each
  call site holds, and re-deriving it a second way risks a value that silently disagrees with the
  boundary's own declared scope.
- **Placement: a named `fact.fields` entry, not baked into `target`.** Keeps `target` stable as the
  human-anchored module path (unchanged rendering) and keeps the package as a separately-inspectable
  identity component, per `structured-violation-identity`'s explicit ban on concatenating
  identity-bearing components into an opaque string.
- **Single modified capability: `structured-violation-identity`.** guibiao's `module-boundary` and
  each touched hunyi capability spec (`semantic-visibility-boundary`,
  `semantic-async-exposure-boundary`, etc.) describe *behavior* in prose and never enumerate literal
  fact field names (verified: `semantic-visibility-boundary/spec.md` and `module-boundary/spec.md`
  contain no field-literal text) — so none of their requirements text becomes false. The
  cross-dimension identity contract belongs in `structured-violation-identity`, whose own
  compatibility requirement already spans "圭表, 渾儀, and 漏刻 with all features."

## Risks / Trade-offs

- **[Risk] Wide, easy-to-under-cover surface** — every `ModuleFact`/`SemanticFact` variant and every
  `SingleModuleViolationContext`/`MultiModuleViolationContext` call site must gain the field; missing
  even one leaves that fact family's cross-crate collision unfixed. → **Mitigation**: the
  compatibility catalog test asserts every shipped fact family's exact fields (structured-violation-
  identity's own discipline); a missed site fails that test rather than passing silently.
- **[Risk] `MultiModuleViolationContext` may differ in shape from `SingleModuleViolationContext`**
  (not yet read at proposal time) — could need its own threading path. → **Mitigation**: apply-stage
  task explicitly enumerates and verifies both context constructors before claiming the tasks
  covering hunyi are done.
- **[Risk] Breaking change to every existing baseline** for a module or semantic boundary — a
  baseline written before this change stops matching (new required field), so every previously
  accepted violation reappears as new once. → **Mitigation**: this is the correct, intended
  consequence, not a defect to hide — a baseline is a generated snapshot, not policy
  (`violation-baseline` spec's own Purpose), and re-running `--write-baseline` regenerates it. The
  CHANGELOG `[Unreleased]` entry states this explicitly as a migration step so adopters are not
  surprised; **no version bump or release-cut happens as part of this change** (deferred to the
  maintainer's own review of the whole audit-backlog campaign).

## Migration Plan

1. Land the field across guibiao and hunyi fact construction plus their compatibility catalogs.
2. Add the two-crate collision regression test (survives before → fails; passes after — the
   non-vacuous verification this project's adversarial-review discipline requires).
3. CHANGELOG `[Unreleased]` entry under `### Fixed`, marked **BREAKING**, naming the
   `--write-baseline` regeneration step for any adopter with an existing module/semantic baseline.
4. No baseline auto-upgrade, no version bump — out of scope for this change.

## Open Questions

- None outstanding; `MultiModuleViolationContext`'s exact shape is a tasks-stage verification item,
  not an unresolved design fork.
