## Context

`PublicSeam` (`crates/hunyi/src/finding.rs`) is the shared seam vocabulary for signature-coupling,
dyn-trait, and impl-trait — three capabilities that tag each exposed position with the public seam
it sits at, so two seams exposing the same forbidden shape never collapse under one `(target,
rule_key, fact)` baseline entry. `key_fields(&self) -> Vec<(&'static str, &str)>` is called from
exactly one site, `SemanticFact::into_finding_with_text`'s `Exposed` arm.

`UnsafeSiteFact::key_fields(&self, module: &str)` (same file) already solves an adjacent problem —
"never forget to key on the enclosing module" — by taking `module` as an **external parameter**,
injected once ahead of the per-variant match, so no variant can omit it. `PublicSeam` cannot copy
that shape verbatim: `PublicSeam::ExternCrate { name }` is crate-scoped (there is no governing module
to inject), and `PublicSeam::TraitImpl { trait_ref, owner, position }` carries no module field today
either (a real, un-named gap, out of scope here — see Non-Goals). Forcing a blanket `module: &str`
parameter onto `key_fields` would need special-casing both, which is worse than the six-variant
per-field pattern the enum already uses.

## Goals / Non-Goals

**Goals:**
- Make `PublicSeam::InherentMethod`/`InherentAssoc` identity-injective across two impl blocks in
  different modules for the same owner, matching the six sibling variants that already carry
  `module`.
- Keep human-rendered text (`Display`) unchanged, so this is identity-only and does not re-word any
  existing finding string.
- Thread the fix through all three consumers (`collect_item_exposures`, `collect_item_dyn_exposures`,
  `collect_item_return_impl_traits`) uniformly, even though only the last is currently reachable
  through a live two-module false negative.

**Non-Goals:**
- `PublicSeam::InherentGenerics`'s per-block distinguisher (separate BACKLOG entry).
- Adding `including_submodules()` to dyn-trait or signature-coupling.
- Touching `ExternCrate` or `TraitImpl`.

## Decisions

### Decision 1: Per-variant `module: String` field, not an external `key_fields(&self, module)` parameter

Add `module: String` directly to `InherentMethod` and `InherentAssoc`, populated at construction
time from the same `module: &str` every call site already threads (the collectors' own scan-loop
parameter). This matches the pattern six sibling variants (`FreeFn`, `TraitMethod`, `Item`, `Member`,
`TraitAssoc`, `Reexport`) already use — each decides its own identity fields — rather than
`UnsafeSiteFact`'s external-injection shape, which does not fit `PublicSeam`'s heterogeneous variant
set (see Context). `key_fields()`'s per-variant match still owns the decision for every variant; only
these two variants change what they decide.

### Decision 2: Identity-only — `Display` ignores the new field

`InherentMethod`/`InherentAssoc`'s existing render (`fn <{owner}>::{name}`, `{kind} <{owner}>::{name}`)
does not change. This mirrors `SemanticFact::AsyncInherentMethod`, already shipped in the same file,
which carries `module` distinct from `owner` and already excludes it from its own `Display` arm
(`Self::AsyncInherentMethod { owner, name, tail, .. } => write!(f, "async fn <{owner}>::{name}{tail}")`).
The BACKLOG entry's own open question — "deciding whether the module belongs in the seam's identity
only or also its rendered label" — is answered by following this existing precedent rather than
inventing a second answer for a near-identical shape in the same file. Consequence: an adopter who
was already seeing `impl crate::Port exposed by fn <crate::common::Conn>::open` sees the identical
string twice now (once per module) rather than a re-worded string once.

### Decision 3: `inherent_method_seam`/`inherent_assoc_seam` gain a leading `module: &str` parameter

```rust
pub(crate) fn inherent_method_seam(module: &str, owner: &str, name: &syn::Ident) -> PublicSeam
pub(crate) fn inherent_assoc_seam(kind: AssocKind, module: &str, owner: &str, name: &syn::Ident) -> PublicSeam
```

All seven call sites (three of `inherent_method_seam`, four of `inherent_assoc_seam`, across
`collect_item_return_impl_traits`, `collect_item_exposures`, `collect_item_dyn_exposures`) already
have `module: &str` in scope as their own function parameter — no new plumbing beyond the call sites
themselves.

### Decision 4: `key_fields()` gains `("seam_module", module)` for both variants

Same field name six siblings already use (`FreeFn`, `TraitMethod`, `Item`, `Member`, `TraitAssoc`,
`Reexport` all key their module under `"seam_module"`), so the structured-fact vocabulary stays
consistent rather than introducing a second name for the same concept.

## Risks / Trade-offs

- **Baseline breakage.** Every existing `InherentMethod`/`InherentAssoc`-seam baseline entry becomes
  stale (the fact gains a required field) — the identical, already-accepted cost the
  `governing_package` fix took (`CHANGELOG.md`, 0.3.0-line **BREAKING** entry). No adopter can
  silently lose coverage: a stale baseline entry means the (now differently-shaped) violation
  reappears as new, never disappears.
- **`InherentAssoc`'s fix is not independently reachable today.** Recorded honestly in the proposal
  rather than claimed as an integration-level closure — the regression coverage for it is a unit-level
  identity-injectivity assertion (`finding.rs`'s existing
  `every_public_seam_shape_is_named_and_identity_injective` test, extended), not a second
  `check_impl_trait`/`check_dyn_trait`/`check` two-module reproduction.

## Migration Plan

Additive field, no DSL/builder/CLI surface change. An adopter regenerates any stale
`InherentMethod`/`InherentAssoc`-seam baseline entry with `--write-baseline`; until then the affected
findings reappear as new (fail-loud, never a silent coverage loss).
