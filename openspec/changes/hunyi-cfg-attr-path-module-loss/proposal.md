## Why

`crates/hunyi/src/scan.rs`'s crate-wide walk (`resolve_child_modules`) treated ANY `cfg_attr`-wrapped
`#[path]` remap as a blanket skip bound — for a module reached only that way, the whole subtree's
observation vanished, not just the alternate target the predicate might select. `cfg_attr` never
removes the `mod` item itself (unlike a bare `#[cfg]`), so the module is present, and needs SOME file
to back it, on every configuration; skipping it outright is a false negative, not a stated bound.

Two shapes, both reproduced directly (matching the audit findings):

- **Inline**: `#[cfg_attr(windows, path = "x.rs")] pub mod inner { pub fn f() { unsafe {} } }` with no
  `x.rs` — none is needed, since `#[path]` has NO effect on an inline module at all (rustc always
  compiles the body; verified against a real build). The whole inline body — its `unsafe` sites,
  markers, re-exports, everything — was silently dropped from every crate-wide capability. 圭表 and
  漏刻 both already react on the identical file; 渾儀 alone stayed silent (exit 0).
- **File-based**: `#[cfg_attr(any(), path = "never.rs")] pub mod imp;` with `imp.rs` present and
  `never.rs` absent — `any()` is always false, so rustc always compiles the conventional file. It was
  never read either: the whole module and its subtree vanished from `scan_crate`'s maps
  (re-exports/aliases/trait-impls/type-defs), so every downstream capability that reads them computed
  against an incomplete crate.

Since `scan_crate` backs signature-coupling's own alias/re-export closures (`exposure.rs`), the shared
principal-trait resolver dyn-trait/impl-trait's operand-scoped boundaries use
(`crate_scope.rs::extern_resolution`), forbidden-marker, trait-impl-locality, and unsafe-confinement —
five capabilities, not only the two the original audit findings measured against — independently
reproduced and confirmed to have the identical gap before being folded into this one fix.

An independent adversarial apply-stage review then found two more consumers sharing the exact same
`resolve_child_modules` mechanism via a SEPARATE entry point, `walk_subtree_modules` (used by
async-exposure's and impl-trait's subtree-scope opt-in, `including_submodules()`): both had the
identical gap, and both were independently reproduced before being counted as fixed. The review also
found the fix's own doc comments had two stale references describing the pre-fix "skip" behavior.

A THIRD independent review then re-examined this change's own claim that `crates/hunyi/src/
module_resolve.rs`'s single-module-anchored descent was "already correct, fails loud" on this shape —
and found that claim false. `descend()`'s `has_path_attr` skip only fails loud when the dropped
`cfg_attr`-wrapped declaration is the SOLE candidate for that segment; whenever a mutually-exclusive
sibling declaration (a bare `#[cfg]` twin or a `cfg_if!` arm sibling) for the identical name ALSO
resolves successfully, the branch-emptiness check that would trigger the error never fires, so the
`cfg_attr` target's own file is silently dropped with exit 0, not exit 2 — reproduced directly (both
plain-`#[cfg]` and `cfg_if!` forms). This is a pre-existing gap, not introduced by this change, but
this change's own prior claim about the function's correctness did not survive scrutiny. Fixed by
extending `descend()` with the identical union `resolve_child_modules` already applies — closing it
for EVERY consumer of the single-module-anchored descent: signature-coupling's own anchor resolution,
visibility, dyn-trait's and impl-trait's module-scoped (non-subtree) variants, and
trait-impl-exposure.

## What Changes

- `resolve_child_modules` no longer skips a `cfg_attr`-wrapped `#[path]` module outright:
  - An **inline** module's body is always descended, unconditionally — `#[path]`/`cfg_attr(path)` is
    irrelevant to it.
  - A **file** module's conventional file AND its `cfg_attr` target (if it exists on disk) are both
    read as separate sources for the same module name, unioned — cfg-blind observation cannot know
    which one a given build actually compiles. Neither existing, with no other cfg-conditional gate
    on the declaration, remains a genuine scan error (exit 2) — the module is never removed by
    `cfg_attr`, so on every configuration something must back it.
- A new `cfg_attr_path_value` (`syn_util.rs`) extracts the target path from a `cfg_attr`-wrapped
  `#[path]` (including arbitrarily nested `cfg_attr`), mirroring `direct_path_value`'s pattern for the
  unconditional form.
- `crates/hunyi/src/module_resolve.rs`'s SEPARATE single-module-anchored descent (`descend`) gets the
  identical union fix: a `cfg_attr`-wrapped `#[path]` module's conventional file and its `cfg_attr`
  target are both read when they exist on disk — even with no sibling declaration to keep the branch
  count non-empty. Found by a third adversarial review to have the identical false-negative class
  this change already closed for the crate-wide walk, contradicting this change's own earlier claim
  that the function was "already correct." Its now-fully-dead `has_path_attr`/`is_path_remap`/
  `applied_metas_remap`/`meta_is_path_remap` helpers are deleted (its only remaining caller adopted
  `cfg_attr_path_value` instead).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `semantic-unsafe-confinement`: the crate-wide walk's `cfg_attr`-wrapped `#[path]` bound is replaced
  by the union-observation rule above.
- `semantic-trait-impl-locality`: same — `trait_impl_findings`' `scan_crate` call benefits directly.
- `semantic-forbidden-marker`: same — `forbidden_marker_findings`' `scan_crate` call benefits
  directly.
- `semantic-signature-coupling`: BOTH the crate-wide alias/re-export closure `exposure.rs` builds via
  `scan_crate`, AND the capability's own single-module ANCHOR resolution (via `module_resolve.rs`)
  now include/follow a `cfg_attr`-wrapped `#[path]` module's content.
- `semantic-dyn-trait-operand-boundary` / `semantic-impl-trait-operand-boundary`: the shared
  `resolve_principal`'s re-export closure (via `crate_scope.rs::extern_resolution`, itself a
  `scan_crate` caller) benefits the same way.
- `semantic-async-exposure-boundary` / `semantic-impl-trait-boundary`: their subtree-scope opt-in
  (`including_submodules()`, via `walk_subtree_modules` → `resolve_child_modules`) benefits the same
  way — found by adversarial review, independently reproduced.
- `semantic-visibility-boundary`: its module-anchor resolution (via `module_resolve.rs`) now follows
  a `cfg_attr`-wrapped `#[path]` anchor/ancestor.
- `semantic-dyn-trait-boundary`: its shape-only, module-scoped resolution (via `module_resolve.rs`)
  benefits the same way.
- `semantic-trait-impl-exposure`: shares signature-coupling's crate-wide closure, benefits the same
  way.

## Impact

- Affected code: `crates/hunyi/src/scan.rs`, `crates/hunyi/src/syn_util.rs`,
  `crates/hunyi/src/module_resolve.rs`.
- No public API/DSL/builder change, no baseline format change (this fixes false negatives, not an
  identity shape — an adopter's existing baseline is unaffected either way).
- `impl-trait-boundary`'s and `async-exposure-boundary`'s own SEAM-ONLY (non-subtree) module
  resolution ALSO goes through `module_resolve.rs` and is therefore ALSO fixed, though neither spec
  independently stated the now-corrected claim in its own text (only their subtree-scope requirements
  did, already updated above) — no further spec delta needed for the seam-only case since none made
  a wrong claim to correct.
