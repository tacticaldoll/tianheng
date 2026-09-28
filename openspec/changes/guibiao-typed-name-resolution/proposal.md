# Proposal: resolve inline paths from one typed scope table

## Status

Design and measurement only. No product code is changed by this proposal. The proposal branch is based on `release/0.8.0` at `8dae20662188316c55fd104953632961ad4966c3`.

## Problem

The inline-path scanner currently answers related questions through separate heuristics: a module-local use map, a crate-wide alias/re-export closure, glob-hazard walks, prefix spelling checks, and a token-position test for leading `::`. The required Cargo probes show these answers diverge at scope inheritance, spelling identity, local-versus-external shadowing, edition keywords, and qualified associated paths. In particular, the current release misses a forbidden `SystemTime::now()` inherited through `use super::*` (R1), while it reports `<W>::md5x()` as an external-root path (R6).

The proposed change asks one question for a path head: **which typed binding does this first identifier resolve to from the occurrence's lexical scope, edition, and root form?** The resolver returns a canonical binding or an explicit cannot-judge result. Prefix matching and default/strict observation policy consume that answer; they do not repeat Rust-name heuristics.

## Proposed outcome

- Build one per-package scope graph for module declarations, named imports and re-exports, glob edges, item aliases, dependency roots, sysroot roots, `extern crate` aliases, and editions.
- Resolve direct bindings and glob imports to a fixed point. Preserve competing candidates instead of choosing an arbitrary one.
- Normalize raw identifiers and equivalent extern-root spellings before constructing `RuleKey`; distinguish an actual local shadow from an external-prelude binding.
- Keep `.strict_external()` as the existing opt-in observation policy for fully-qualified dependency paths. In each mode, `foo::f()` and `::foo::f()` receive the same answer. The measured R3 target is default clean for both spellings and strict violation for both, matching the documented default bound and the older release behavior.
- Keep guibiao independent of `syn` and `hunyi::resolve`. Consolidate duplicate use-tree walks so every consumer reads the same collected scope facts.

The complete per-scenario measurements and raw commands/output are in [measurements.md](measurements.md) and [measurements.raw.log](measurements.raw.log). The requirement delta is a draft in [spec.md](spec.md); it is not yet the active capability spec.

## Compatibility direction

R1 closes a measured false negative; R2 changes baseline identity so equivalent spellings share one key. Both require a **BREAKING** release under this repository's versioning rule. Edition-aware rejection of unraw strict/reserved prefixes can also require an adopter to change the boundary string to `r#head`, so that migration must be called out if retained in implementation. R3 removes the current release's accidental default-only reaction for `::foo`, keeping default clean for both spellings and strict mode enforcing both. R4's local shadow and R6's associated-item path remain clean. Those corrections do not add adopter work and are patch-class on the measured evidence.

Before implementation, review should decide whether the keyword-prefix spelling change is required in this same breaking change and whether the declared cannot-judge perimeter is narrow enough. If accepted, the `CHANGELOG.md` Migration section must tell adopters to regenerate affected baselines and use `r#head` in boundary prefixes for edition-reserved names.
