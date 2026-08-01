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
- hunyi's shared emission points build `ViolationId::new(context.<anchor>, ...)` the same way, but
  the two contexts differ in shape: `push_single_module_violations` keys on `context.module`
  (`SingleModuleViolationContext`), while `push_multi_module_violations` keys on `context.target`
  (`MultiModuleViolationContext` has no `module` field at all —
  `crates/hunyi/src/emit.rs:52-64` vs `31-35`). Both are module-path strings for every caller
  **except** `unsafe_confinement.rs:55`, which sets `target: &boundary.crate_package` directly —
  already crate-scoped, not part of this bug. Confirmed by grepping every `target:`/`module:` value
  across `visibility.rs`, `exposure.rs`, `dyn_trait.rs`, `async_exposure.rs`, `impl_trait.rs`,
  `forbidden_marker.rs`, and `trait_impl.rs`: all of them pass `&boundary.module` (or an
  equivalently module-shaped local) — genuinely collision-prone. (An earlier draft of this design
  claimed both emit functions keyed on `context.module` uniformly; an independent adversarial
  review of the propose stage caught this as imprecise — corrected here.)
- Critically, **every one of those call sites already holds `&boundary.crate_package` in scope**
  (confirmed in `async_exposure.rs`, `impl_trait.rs`, and mirrored across the other capability
  files) — this is a threading gap, not a missing lookup. The package name is available everywhere
  the fix needs it; it is simply never passed the last step into the fact/context builder.
- `structured-violation-identity`'s own requirement text already forbids the wrong fix: "Separately
  observed identity-bearing components SHALL remain fact-specific named fields rather than being
  concatenated into an opaque display string" — so the package name must become its own named
  field, not get appended into `target`'s string.
- Existing convention check (this is where the propose-stage adversarial review found a real
  problem): `CrateFact::dependency`/`CrateFact::feature` (guibiao's crate-dependency fact family,
  same file, `crates/guibiao/src/finding.rs:52,68,82`) already key a field literally named
  `"package"` — but there it names the *observed dependency's* package (e.g. `"serde"`, the object
  of a `must_depend_on`/confinement rule), confirmed against `crates/guibiao/src/crate_check.rs:
  13-25` where the governing crate itself already lives in `target`, not in a `"package"` fact
  field. Reusing `"package"` for a *different* referent (the crate that declared the boundary,
  for `ModuleFact`/`SemanticFact`) would silently overload one field name with two meanings across
  the same identity model — worse than inventing a new name. **Field name is `governing_package`.**

## Goals / Non-Goals

**Goals:**
- Every guibiao `ModuleFact` variant, and every hunyi `SemanticFact` variant reached through a
  module-path-scoped boundary (i.e. every capability except `unsafe_confinement`), carries a
  `governing_package` field equal to `boundary.crate_package`, so two crates declaring the
  identical module path + rule produce distinct `ViolationId`s.
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
- `unsafe_confinement`'s `SemanticFact::UnsafeSite` (or equivalent variant) does **not** gain
  `governing_package` — its `target` already carries `boundary.crate_package` directly, so adding
  the same value again as a fact field would be a redundant reaction (the drift law's own
  minimalism clause: no reaction without something new to react to).

## Decisions

- **Field name: `governing_package`, not `"package"` or `"crate"`.** guibiao's existing `CrateFact`
  family already uses `"package"` for the *observed dependency's* name (confirmed:
  `crates/guibiao/src/finding.rs:52,68,82` pairs with `crate_check.rs:13-25`, where the governing
  crate itself is `target`, and `"package"` in fact fields names the dependency being reasoned
  about — a different referent). Reusing that literal for "the crate that declared this boundary"
  would overload one field name with two meanings in one identity model — an independent
  adversarial review of this proposal caught the collision before implementation. `"crate"` was
  also considered and rejected: `target` already renders `"crate::..."` path spellings, so a field
  named bare `"crate"` invites confusion with that unrelated string.
- **Value: the exact `boundary.crate_package` string**, not a re-resolved crate name from
  `cargo_metadata`. It is what the adopter wrote in `.in_crate(...)`, it is already the value each
  call site holds, and re-deriving it a second way risks a value that silently disagrees with the
  boundary's own declared scope.
- **Placement: a named `fact.fields` entry, not baked into `target`.** Keeps `target` stable as the
  human-anchored module path (unchanged rendering) and keeps the package as a separately-inspectable
  identity component, per `structured-violation-identity`'s explicit ban on concatenating
  identity-bearing components into an opaque string.
- **`unsafe_confinement` is explicitly excluded**, not silently covered by a generic rule. Its
  `MultiModuleViolationContext` call already sets `target: &boundary.crate_package`
  (`crates/hunyi/src/unsafe_confinement.rs:55`) — it has no cross-crate collision to begin with,
  since the governing crate already IS the identity's `target`. Both context structs still gain
  `crate_package: &'a str` uniformly (every caller already holds the value, and a uniform struct
  shape is simpler than a conditional field), but `unsafe_confinement`'s own fact-variant
  conversion is the one place that deliberately does not consume it into `fact.fields`.
- **Spec placement: extend the existing `structured-violation-identity` requirement, not add a new
  one, and do not touch per-dimension capability specs.** The first propose draft added a *new*
  requirement to `structured-violation-identity` prescribing that dimensions add a specific named
  field — an independent adversarial review flagged this as self-contradicting: that capability's
  own existing "Observation dimensions own fact meaning and rendering" requirement states the
  shared envelope "SHALL NOT contain crate-, module-, semantic-, or runtime-specific fact
  vocabulary," and dictating dimension-specific field content from within the same spec violates
  that just as much as if xuanji itself hardcoded a `"crate"` field. The corrected placement instead
  adds scenarios to the *already-existing* "Distinct facts carry distinct identities" requirement —
  which already states the general principle abstractly ("two observations differ in any
  identity-bearing observed value") without naming any field — so the delta only sharpens an
  existing cross-dimension discipline rather than injecting new dimension-owned vocabulary. This
  also avoids the alternative of touching ~8 per-capability semantic-* specs for one conceptual
  fix, which would cut against minimalism for no behavioral-text gain (none of those specs
  enumerate literal fields today, confirmed by the same review).

## Risks / Trade-offs

- **[Risk] Wide, easy-to-under-cover surface** — every `ModuleFact`/`SemanticFact` variant and every
  `SingleModuleViolationContext`/`MultiModuleViolationContext` call site must gain the field; missing
  even one leaves that fact family's cross-crate collision unfixed. → **Mitigation**: the
  compatibility catalog test asserts every shipped fact family's exact fields (structured-violation-
  identity's own discipline); a missed site fails that test rather than passing silently.
- **[Risk] An apply-stage implementer follows the requirement's abstract wording literally and adds
  `governing_package` to `unsafe_confinement`'s fact anyway**, since it IS a boundary kind
  declarable against more than one crate — just not one with a collision, because its `target`
  already encodes crate. → **Mitigation**: tasks.md calls this out as its own explicit item with the
  file:line citation, not left to be inferred from the general rule.
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
