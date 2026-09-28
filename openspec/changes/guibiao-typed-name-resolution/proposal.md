# Proposal: resolve inline paths from one typed scope table

## Status

Design and measurement only. No product code is changed by this proposal. The proposal branch is based on `release/0.8.0` at `8dae20662188316c55fd104953632961ad4966c3`.

## Problem

The inline-path scanner currently answers related questions through separate heuristics: a module-local use map, a crate-wide alias/re-export closure, glob-hazard walks, prefix spelling checks, and a token-position test for leading `::`. The Cargo probes show these answers diverge at scope inheritance, block scope, spelling identity, local-versus-external shadowing, edition-2015 root paths, cfg-exclusive imports, and qualified associated paths. In particular, the current release misses a forbidden `SystemTime::now()` inherited through `use super::*` (R1), and every measured revision binds a name to the last `use` of it written anywhere in the file, so a function-local `use` changes what the rest of the module resolves to (B3/B4), while the release reports `<W>::md5x()` as an external-root path (R6).

The proposed change asks one question for a path head: **which typed binding does this first identifier resolve to from the occurrence's lexical scope, edition, and root form?** The resolver returns a canonical binding or a set of candidates, never a refusal of its own. Prefix matching and default/strict observation policy consume that answer; they do not repeat Rust-name heuristics.

## Proposed outcome

- Build one per-package scope graph for module and block scopes, named imports and re-exports, glob edges, item aliases, dependency roots, sysroot roots, `extern crate` aliases, and editions.
- Resolve direct bindings and glob imports to a fixed point. Preserve competing candidates instead of choosing one; an occurrence reacts when any candidate resolves under the prefix.
- Normalize raw identifiers and equivalent extern-root spellings before constructing `RuleKey`; distinguish an actual local shadow from an external-prelude binding.
- Keep `.strict_external()` as the existing opt-in observation policy for fully-qualified dependency paths. In each mode, `foo::f()` and `::foo::f()` receive the same answer. The measured R3 target is default clean for both spellings and strict violation for both, matching the documented default bound and the older release behavior.
- Hold a boundary-prefix head to one question — can it name a crate or module? — refusing only the finite set the Rust Reference's identifier grammar excludes, with no edition keyword table.
- Keep guibiao independent of `syn` and `hunyi::resolve`. Consolidate duplicate use-tree walks so every consumer reads the same collected scope facts.

The complete per-scenario measurements and raw commands/output are in [measurements.md](measurements.md), [measurements.raw.log](measurements.raw.log) and [measurements.revision.raw.log](measurements.revision.raw.log). The requirement delta is a draft in [spec.md](spec.md); it is not yet the active capability spec.

## Capabilities

- `inline-symbol-path-confinement` — modified: its path heads resolve from the scope table, its prefix-head refusal becomes the finite set, and it gains the bounds this change declares.
- `external-crate-confinement`, `crate-dependency-boundary`, `crate-source-boundary` — their subjects include the guibiao scanner files this change edits (`path_vocab.rs`, the scanner module), but their requirements do not change: import reports are read by the existing `use_scan` walk, which this change leaves byte-identical.
- `rule-model-surface` — `module_rule.rs` now keys an inline rule on the prefix's canonical path; the requirement that a parameter enters the key exactly when changing it changes what the boundary forbids does not change, and a prefix's root form (`::std::time` beside `std::time`) is such a parameter it keeps out.
- `observation-bound-model`, `observation-bound-register` — the change declares one bound and gives another a second pin through `bounds.rs` and the capability spec, and regenerates the two projections; their requirements on how a bound is declared, classified and projected do not change.
- `release-coherence` — `CHANGELOG.md` gains this change's `[Unreleased]` and Migration entries; the release-coherence requirements on that file do not change.

## Compatibility direction

Against v0.7.1, R2's normalized key and the closed false negatives — R2's `::std::time`, block-local imports (B3/B4), cfg-exclusive imports (E6) and edition-2015 root paths (D) — each require adopter action and are **BREAKING** under this repository's versioning rule; so is refusing a `_` prefix head, which never named anything that compiles. R1, R3, R4 and every keyword prefix head keep v0.7.1's answer: a boundary prefix is a name, so `async` and `r#async` are one head and neither needs rewriting. R6 and the block-local item shadow (F) remove false positives and are patch-class. The full table is design i.

The `CHANGELOG.md` Migration section must tell adopters to regenerate affected baselines, replace a `_` prefix head, and that un-`use`d external dependency paths remain strict-only.
