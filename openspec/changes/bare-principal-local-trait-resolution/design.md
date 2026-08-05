## Context

`resolve_principal` (`crates/hunyi/src/crate_scope.rs`) is the shared resolver for the operand-scoped
`dyn` and `impl Trait` boundaries, reached from exactly one production caller,
`shape_scan::matches_forbidden_principal`. Its ladder is: `crate`/`self`/`super`-relative → the
module's `use` map (`BareFallback::Ignore`) → the external-crate verbatim oracle → **the bare fallback
added in this window**, which unconditionally pushes `{module}::{name}` when nothing else resolved.

`BareFallback::Ignore` is chosen at that call site deliberately; its own doc states why —
"resolving it risks a same-module false positive" — while `BareFallback::CurrentModule` exists for the
impl-locality and unsafe-site callers that *want* the module-relative reading. The new fallback
reproduces `CurrentModule`'s body by hand at the one call site that opted out of it, and drops two
properties every other resolution site holds: it never checks that the module declares the name, and it
never canonicalizes a raw identifier.

`file_extern_scope` already computes the answer to the first question. It derives `externs_type` as
`res.externs.difference(&local_type_namespace_names(file_items))` and then discards the local set.

## Goals / Non-Goals

**Goals:**

- A bare principal resolves when — and only when — the governed file's own branch declares that name.
- A raw identifier canonicalizes here as everywhere else, so `crate::m::type` is the one spelling.
- Both operand capabilities' bound scenarios state what the resolver now does, and their pinning tests
  observe the drop rather than a spelling mismatch.

**Non-Goals:**

- Widening resolution to a parent module, a glob import, or the prelude. Rust requires a bare name to be
  declared or imported in its own module, so the file's own items are the whole admissible source; a
  glob-imported trait stays a stated bound.
- Switching the call site to `BareFallback::CurrentModule`. That mode resolves a bare name without
  proving it exists, which is the very over-reach this change removes — the two are not equivalent, and
  the parameter keeps its documented meaning.
- Touching signature-coupling, forbidden-marker, trait-impl locality, or unsafe confinement. They call
  `resolve_path_all` directly and are unaffected.

## Decisions

- **The observation source is the file's own type namespace, carried on `FileExternScope`.** That struct
  is already the per-`#[cfg]`-branch resolution context and is already built from the same `file_items`,
  so the local set rides along instead of being recomputed or threaded separately. This also keeps the
  "`file_scope` MUST be the branch that OWNS `path`" discipline the struct already documents: the branch
  that owns the exposure is the branch whose declarations admit its bare names.
- **The type namespace, not a trait-only filter.** `local_type_namespace_names` collects structs, enums,
  unions, traits, type aliases, and child modules. A principal position accepts only a trait, so a
  compiling crate cannot present a `struct` name there; filtering to `syn::Item::Trait` would add a
  second, narrower enumerator for no observable difference. Reuse the one that exists.
- **`strip_raw` before the candidate is built**, matching `resolve_path_all`, `local_type_namespace_names`,
  and `principal_trait_paths`. The set being compared against is already stripped, so the comparison and
  the emitted candidate agree.
- **The redundant `path.leading_colon.is_none()` conjunct goes.** The fallback sits inside the
  `leading_colon.is_some()` else-branch, so the condition cannot be false; a guard that cannot fail
  misleads a reader about which states are possible.
- **The existing pinning tests are re-pointed, not supplemented.** They forbid the bare spelling
  `["Frobnicate"]`, so they pass whatever the fallback does. Forbidding `["crate::m::Frobnicate"]` is what
  makes them observe the drop — and is what makes them fail against the unfixed resolver.

## Risks / Trade-offs

- **A trait declared in a macro body stays unresolved.** `local_type_namespace_names` reads `syn::Item`s,
  so a `macro_rules!`-generated trait is not in the set and a bare use of it is dropped. That is the
  universal 渾儀 macro-expansion bound, already declared, and the drop direction is the safe one.
- **An adopter's recorded baseline may change in both directions** — a fabricated `{module}::{name}`
  finding disappears, and a raw-identifier trait that never matched now does. This is `**BREAKING**` per
  the CHANGELOG marking rule and earns a minor, however small the diff.
- **The auto-trait scenario's mechanism sentence is corrected in passing.** It claimed a bare `dyn Send`
  is dropped by the resolver under `BareFallback::Ignore`; `Send` is in fact removed by
  `principal_trait_paths`' auto-trait filter before any resolution runs. The outcome was right, the
  mechanism was not, and left standing it invites a test that asserts the wrong thing (a local
  `trait Send` case can never react, so such a test would pass vacuously in one direction and fail
  misleadingly in the other).
