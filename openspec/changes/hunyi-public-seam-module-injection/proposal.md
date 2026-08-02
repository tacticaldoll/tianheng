# proposal: Hunyi Public-Seam Module Injection

## Why

`PublicSeam::InherentMethod { owner, name }` and its sibling `InherentAssoc { kind, owner, name }`
(`crates/hunyi/src/finding.rs`) key an inherent method/associated-item seam on the **self type's**
canonical owner and the item's own name only. Six sibling variants (`FreeFn`, `TraitMethod`, `Item`,
`Member`, `TraitAssoc`, `Reexport`) already carry a `module` field — the module the *item itself* is
declared in — precisely because two distinct impl sites can otherwise render the same identity.
`InherentMethod`/`InherentAssoc` never got one.

`owner` is the self type's **canonical path**, resolved through `canonical_self_owner` against the
enclosing scope's `use`s — it names *what the impl is for*, not *where the impl block is written*.
Rust's coherence rules let an inherent `impl` for a type declared in module `common` be written in
ANY module of the same crate — a real, common idiom for platform-conditional code
(`impl Conn { … }` once in `plat_unix`, once in `plat_win`, both importing the same `common::Conn`).
Both impls resolve `owner` to the identical `crate::common::Conn`. If both also declare a same-named
public method (`open`, `connect`, …), their `PublicSeam::InherentMethod{owner, name}` facts are
byte-identical — even though they are two distinct source sites.

This is **verified real**, not hypothetical: a control fixture with `common::Conn`, and
`plat_unix`/`plat_win` submodules each writing `impl Conn { pub fn open(&self) -> impl crate::Port { … } }`,
evaluated through the real `hunyi::check_impl_trait` (`including_submodules()` subtree scan, the one
capability that already walks more than one module per boundary evaluation) produces exactly **one**
violation, attributed to whichever module sorts first (`crate::plat_unix`) — `crate::plat_win`'s
identical, real violation is silently dropped by `sort_attributed_facts`'s fact-only
`dedup_by`, not merely deduplicated against an equivalent finding. This is the false negative
PROJECT.md's Core Contract forbids outright.

`dyn-trait` (`collect_item_dyn_exposures`) and `signature-coupling` (`collect_item_exposures`) build
the identical `PublicSeam::InherentMethod`/`InherentAssoc` seams through the same `inherent_method_seam`/
`inherent_assoc_seam` constructors, sharing the identical structural gap — but neither capability's
boundary can currently observe more than one module per evaluation (no `including_submodules()`),
and `ViolationId`'s `target` is the boundary's own anchored module (`push_single_module_violations`),
which already differs across two separately-declared boundaries. Confirmed by code reading (both
collectors' call sites, `exposure.rs`'s and `dyn_trait.rs`'s single-module-only evaluation shape,
`emit.rs`'s `target`-keyed `ViolationId`): the collision is **not currently reachable** through
either sibling. It would become reachable the moment either gained a subtree scope — matching
`impl-trait`'s and `async-exposure`'s own precedent — so the shared vocabulary is fixed once here
rather than left to bite silently on the next capability that grows one.

`InherentAssoc` is fixed for the identical structural reason and for consistency with its sibling,
though today no collector wires it to any subtree-capable capability (`collect_item_return_impl_traits`
does not observe associated `const`/`type` items at all, and `signature-coupling`/`dyn-trait` are the
only consumers, both single-module). Its own reproduction is therefore an identity-shape unit
assertion, not a live integration false negative — recorded honestly as such, not overstated.

## What changes

Both variants gain a `module: String` field — the module the impl **block** is written in, always
already in scope at every call site (`collect_item_return_impl_traits`, `collect_item_exposures`,
`collect_item_dyn_exposures` each already take `module: &str`). `key_fields()` gains a `("seam_module",
module)` entry for both variants, following the exact field-name convention the six existing
module-carrying variants already use. The field is **identity-only**: `Display` ignores it (`..`),
mirroring `SemanticFact::AsyncInherentMethod`'s own already-shipped precedent in the same file, which
carries `module` distinct from `owner` for the identical reason and already excludes it from
rendering — so this presentation choice is not new invention, it matches the sibling shape that
already solved the identical "identity vs. label" question the BACKLOG entry left open.

This is a **behavior change**: a previously-collapsed two-module false negative is now two real,
distinct violations. It also **breaks existing baseline entries** for any `InherentMethod`/
`InherentAssoc`-seam finding (`signature-exposure`, `dyn-trait-exposure`, `impl-trait-exposure` fact
types, `public-seam` shape) — the structured fact gains a required field, so a `--write-baseline`
snapshot recorded before this change no longer matches after it, exactly the `governing_package`
precedent already recorded in `CHANGELOG.md`. Per this project's own OpenSpec-only-when-behavior-changes
rule (`bffcf2a`), this goes through the full explore → propose → apply → sync lifecycle, unlike the
pure-refactor sibling on this same branch history that was corrected away from it.

## Non-goals

- `PublicSeam::InherentGenerics { owner }` (a *third* variant with the identical missing-module gap,
  plus a second, independent gap — no per-impl-**block** distinguisher at all) is a separate,
  already-recorded BACKLOG entry (per-block distinguisher needs real design work on what makes a
  block identity-stable) and is **not** touched here.
- `PublicSeam::ExternCrate` (crate-scoped, no module concept) and `PublicSeam::TraitImpl` (its own
  owner/position shape, no module field, not named in any BACKLOG entry) are untouched.
- Adding `including_submodules()` to `dyn-trait` or `signature-coupling` is out of scope; this change
  only makes the shared vocabulary correct so that whenever either gains one, it does not reproduce
  this false negative on day one.
