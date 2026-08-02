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
all five, not only the two capabilities the original audit findings measured against — independently
reproduced and confirmed to have the identical gap before being folded into this one fix.

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
- `crates/hunyi/src/module_resolve.rs`'s SEPARATE single-module-anchored descent (used by
  signature-coupling's own module-anchor resolution, visibility, async-exposure, and dyn/impl-trait's
  module-scoped variant) is untouched: it already, deliberately, fails loud (exit 2 "cannot judge") on
  a `cfg_attr`-wrapped `#[path]` rather than silently passing — an accepted, narrower, already-correct
  bound, not the false-negative class this change closes.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `semantic-unsafe-confinement`: the crate-wide walk's `cfg_attr`-wrapped `#[path]` bound is replaced
  by the union-observation rule above.
- `semantic-trait-impl-locality`: same — `trait_impl_findings`' `scan_crate` call benefits directly.
- `semantic-forbidden-marker`: same — `forbidden_marker_findings`' `scan_crate` call benefits
  directly.
- `semantic-signature-coupling`: the crate-wide alias/re-export closure `exposure.rs` builds via
  `scan_crate` now includes a `cfg_attr`-wrapped `#[path]` module's own re-exports/aliases; the
  SEPARATE single-module-anchor bound (`module_resolve.rs`) is unchanged and remains a stated,
  fail-loud bound.
- `semantic-dyn-trait-operand-boundary` / `semantic-impl-trait-operand-boundary`: the shared
  `resolve_principal`'s re-export closure (via `crate_scope.rs::extern_resolution`, itself a
  `scan_crate` caller) benefits the same way.

## Impact

- Affected code: `crates/hunyi/src/scan.rs`, `crates/hunyi/src/syn_util.rs`.
- No public API/DSL/builder change, no baseline format change (this fixes false negatives, not an
  identity shape — an adopter's existing baseline is unaffected either way).
- Out of scope, named explicitly rather than silently left: `module_resolve.rs`'s single-module-
  anchored descent (signature-coupling's own anchor resolution, visibility, async-exposure,
  dyn/impl-trait's module-scoped variant) keeps its existing, already-correct fail-loud bound for a
  `cfg_attr`-wrapped `#[path]` — a narrower, accepted scope difference from the crate-wide walk, not a
  silent pass, and not touched here.
