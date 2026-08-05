## Why

The bare-principal resolution fallback added in this release window resolves **every** unresolved
single-segment principal to `{module}::{name}`, without proving that the module declares that name and
without canonicalizing a raw identifier. Both directions are wrong, and one of them is the class the
Core Contract forbids outright:

- **A false negative.** Every other resolution site in 渾儀 canonicalizes an identifier through
  `strip_raw`, so a forbidden operand is spelled `crate::m::type`. The fallback does not, so a real local
  `pub trait r#type` resolves as `crate::m::r#type` and never matches. A declared boundary silently
  passes over the trait it forbids.
- **A fabricated resolution.** A bare non-auto name that the module does not declare — a prelude trait
  (`dyn Iterator`, `dyn Fn()`, `dyn Future`), a glob-imported trait, a name the file never mentions — is
  resolved to `{module}::{name}` anyway. A boundary forbidding that path then reacts over a trait the
  module never declared, which contradicts both operand capabilities' own `genuinely unresolvable bare
  principal` bound: the resolver-coverage bound states the principal does **not** resolve, and the
  oracle "does not over-reach a single bare segment".

The two existing pinning tests do not catch either direction. They forbid the bare spelling
`["Frobnicate"]` rather than the qualified `["crate::m::Frobnicate"]`, so both still pass — for a spelling
mismatch rather than for the drop they claim to pin. A bound whose test passes for an unrelated reason
reads as permission: it tells an auditor a real escape is governed policy.

Now, because the fallback is unreleased. It ships in no published version, so the bound's wording and
the resolver can be brought back into agreement without an adopter ever having seen the disagreement.

## What Changes

The fallback stays — a bare trait genuinely needs no `use` in its own module, which is why it was added —
but it is admitted only where the observation supports it:

- The **branch's own local type namespace** is carried on `FileExternScope`, which already computes it
  (it derives `externs_type` from exactly that set), and the fallback fires only when the canonical name
  is in it. Nothing new is computed and no second scope is introduced.
- The segment is **canonicalized with `strip_raw`** before the candidate is built, so `r#type` and
  `type` are one name here as everywhere else.
- Both operand capabilities' bound scenarios say what now holds: a bare principal resolves against a
  **locally declared** name, and stays dropped otherwise.
- Both pinning tests are re-pointed at the qualified forbidden spelling, so they observe the drop rather
  than a spelling mismatch, and each capability gains a discriminating pair: a locally declared bare
  trait reacts; a bare name the module does not declare does not.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `semantic-dyn-trait-operand-boundary`: the `genuinely unresolvable bare principal` bound is restated —
  a bare principal resolves when the governed module's own branch declares that name, and is dropped
  otherwise. The resolution requirement gains the raw-identifier canonicalization it always implied.
- `semantic-impl-trait-operand-boundary`: the same restatement over the `impl Trait` operand, which
  shares `resolve_principal`. 三儀 ⊥ 三儀 requires each capability to declare its own.

## Impact

- `crates/hunyi/src/crate_scope.rs` — `FileExternScope` gains `local_types`; `resolve_principal`'s
  fallback is gated and canonicalized.
- `crates/hunyi/src/tests/resolver_fidelity.rs`, `crates/hunyi/src/tests/impl_trait.rs` — the two bound
  tests are re-pointed and the discriminating cases added.
- `openspec/specs/semantic-dyn-trait-operand-boundary/spec.md`,
  `openspec/specs/semantic-impl-trait-operand-boundary/spec.md` — bound wording.
- `docs/observation-bounds.md` — regenerated projection (bound statements change).
- `CHANGELOG.md` — a `**BREAKING**` entry: the reaction changes in both directions, so a recorded
  baseline no longer describes the adopter's tree.
- No public API, no wire format, no identity shape moves. `resolve_principal` has exactly one production
  caller (`shape_scan::matches_forbidden_principal`), so signature-coupling, forbidden-marker,
  trait-impl locality, and unsafe confinement are untouched.
