# proposal: Hunyi Rejects a Malformed `::`-Path Operand

## Why

An adversarial-sweep finding (`docs/audit/0.3.1-adversarial-sweep.md`) claimed
`must_not_expose("::serde")` — a leading-`::` forbidden operand — silently disables
signature-coupling instead of reacting. Verified directly against `crates/hunyi/src/exposure.rs`
with a throwaway regression probe (module `crate::api` declaring `pub fn ext() -> ::serde::Value`,
`serde` a real dependency), then reverted before this change existed:

| `must_not_expose(...)` operand | reacts? |
| --- | --- |
| `"serde"` (bare, control) | yes — `serde::Value exposed by fn crate::api::ext` |
| `"::serde"` (leading `::`) | **no — silent pass** |
| `"serde::"` (trailing `::`) | **no — silent pass** |
| `"::serde::"` (doubled) | **no — silent pass** |

Root cause is a spelling mismatch, not a resolution bug: `extern_verbatim_renamed`
(`crates/hunyi/src/resolve/mod.rs`) builds a resolved canonical path purely from `syn::Path`
*segments* — it never consults `leading_colon` — so `-> ::serde::Value` resolves to `"serde::Value"`,
never `"::serde::Value"`. `containment::path_within` then compares that resolved string against the
forbidden entry **verbatim**: `path_within("serde::Value", "::serde")` is `false` (no equality, and
`"serde::Value"` does not start with `"::serde::"`). No guard, no error — a pure string-containment
miss that silently falls through to "boundary satisfied."

The audit's framing — that the spec's own advice made this operand spelling look
*recommended* — does not hold up: `semantic-signature-coupling/spec.md`'s "leading `::` is an
unambiguous extern" language (originally cited near line 150, now the "Requirement: Name resolution
scope and no false negative" section) is about how to write the **scanned source** (`-> ::serde::Value`)
to force extern resolution over a local shadow — not about how to spell the `must_not_expose(...)`
**operand**. The spec's own worked scenario for this exact rule uses the bare form,
`must_not_expose("serde")`, against that same `::serde::Value` source, and every existing test in
`crates/hunyi/src/tests.rs` that exercises this path does the same. A whole-repo grep for
`must_not_expose("::` (and the analogous forms on sibling boundaries) finds zero occurrences outside
the audit document itself. So this is not "the spec's recommended escape hatch defeats itself" — it
is an unvalidated, undocumented DSL-operand footgun that a user could plausibly reach by (mis)applying
the *source-spelling* advice to the *operand* spelling, since nothing distinguishes the two contexts
for a reader skimming the spec.

Checking for siblings with the identical shape surfaced two more affected call sites and one narrower
but related one:

- `dyn_operand_module_findings` / `impl_trait_operand_module_findings` (via
  `shape_scan::operand_module_findings`) and `impl_trait_operand_subtree_findings` resolve a `dyn`/
  `impl Trait`'s principal via `crate_scope::resolve_principal`, which calls the *same*
  `extern_verbatim_renamed` — so `must_not_expose_dyn_of(["::serde::Serialize"])` and
  `must_not_expose_impl_trait_of(["::serde::Serialize"])` have the identical leading/trailing-`::`
  silent-pass gap.
- `forbidden_marker_findings` (`must_not_acquire`/`and_not_acquire`) matches by **leaf identifier**
  (`containment::leaf_of`), not full-path containment, so a *leading* `::` is harmless there
  (`leaf_of("::serde::Serialize")` still yields `"Serialize"`). A *trailing* `::` is not: `leaf_of`
  computes `rsplit("::").next()`, which yields `""` for `"serde::"` — an empty leaf no real
  identifier can ever equal, the same silent-pass class through a different mechanism.

Not in scope, and not exhibiting the silent-pass class this finding is about: `unsafe_confinement`'s
and `trait_impl`'s `allowed_locations` (matched via `matches_allowed`/`path_within` against a
crate-relative module path, never extern-resolved) — a malformed entry there makes *every* site look
disallowed, producing spurious violations (fails loud, if noisily) rather than a silent pass; and
`TraitImplBoundary::trait_(...)`'s anchor, which already fails loud via `unknown_trait_error` because
a malformed spelling cannot match any real local trait definition — the anchor-resolution backstop
every boundary already has. See `design.md`'s Non-Goals for the full reasoning.

## What Changes

- Add one shared predicate recognizing a malformed `::`-path operand — any leading, trailing, or
  doubled `::`, or the empty string, i.e. any empty segment of `operand.split("::")` — and one shared
  error constructor, following `unsafe_confinement`'s own precedent (`unsafe_empty_allowed_error`,
  `unsafe_crate_root_allowed_error`) for "this operand shape could never react": a check-time
  `Result<_, String>` returned from the pure heart, surfaced by `run_boundaries` as a **constitution
  error (exit 2)** — never a DSL-construction-time panic, since no existing builder method in this
  DSL validates its string arguments eagerly (not even `because`'s "non-empty reason"), and every
  other malformed-input class in this boundary type (an unresolvable module/trait anchor, an
  empty/crate-root `unsafe` allowlist) already reports through this exact mechanism.
- Wire the check into every call site that canonicalizes a forbidden-operand-shaped list before
  matching: `exposure::module_findings`, `shape_scan::operand_module_findings`,
  `impl_trait::impl_trait_operand_subtree_findings`, and `forbidden_marker::forbidden_marker_findings`.
- Regression coverage for all four call sites: leading `::`, trailing `::`, doubled `::`, and the
  bare-string control continuing to react — reusing the exact reproduction shape verified above.
- State the requirement in the specs it touches, keeping the *source-spelling* advice (leading `::`
  disambiguates a scanned extern path from a local shadow) and the *operand-spelling* restriction
  (the DSL string itself must carry no empty `::`-segment) visibly distinct, so a future reader does
  not re-collapse them into the same claim.

## Capabilities

### Modified Capabilities

- `semantic-signature-coupling`: the forbidden-type-matching requirement gains a stated bound
  rejecting a malformed operand as a constitution error, distinguished from the existing
  source-spelling requirement it sits beside.
- `semantic-dyn-trait-operand-boundary`, `semantic-impl-trait-operand-boundary`: both already state
  they match "exactly as signature-coupling matches a forbidden type... through the same resolver
  ladder" — each gains a one-line cross-reference that the new validation is inherited, not a
  separately-derived rule.
- `semantic-forbidden-marker`: the leaf-identifier matching requirement gains its own stated bound
  for the narrower (trailing-`::`-only) case, since leaf matching is a different mechanism than
  path-prefix containment and needs its own scenario, not a copy of signature-coupling's.

## Impact

- `crates/hunyi/src/resolve/mod.rs`: the shared malformed-operand predicate, beside
  `canonical_path_str`.
- `crates/hunyi/src/errors.rs`: the shared error constructor, beside `unsafe_empty_allowed_error`.
- `crates/hunyi/src/exposure.rs`, `crates/hunyi/src/shape_scan.rs`, `crates/hunyi/src/impl_trait.rs`,
  `crates/hunyi/src/forbidden_marker.rs`: the four call sites.
- `crates/hunyi/src/tests.rs`: regression coverage for all four.
- `CHANGELOG.md`: `[Unreleased]` → `### Fixed`.
- Non-breaking: no public API/signature change (builder methods stay infallible, matching every
  other DSL method in this family); a project using a malformed operand today gets a new constitution
  error where it previously got a silent, permanent non-reaction — a whole-repo grep found no such
  usage anywhere in this codebase, so no adopter-visible regression is expected, and the change is a
  strict tightening of an already-broken (never-reacting) configuration.
