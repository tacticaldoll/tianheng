## Why

`collect_item_exposures` (`crates/hunyi/src/collect.rs`) has no `syn::Item::ForeignMod` arm — a
forbidden type named only in an `extern` block's `pub fn` or `pub static` declaration escapes the
signature-coupling exposure query entirely, silently passing (exit 0 Clean) on source with a real,
callable public API leak. Reproduced directly: `extern "C" { pub fn handle() -> crate::infra::Secret;
pub static S: crate::infra::Secret; }` produces zero findings for either item, while an ordinary
`pub fn control() -> crate::infra::Secret { .. }` in the same module correctly reacts.

An `extern` block's `pub fn`/`pub static` is a real item in the enclosing module's own namespace —
the FFI *declaration*, not a definition, but exactly as callable and exactly as public as a
same-shaped ordinary item (Rust cannot even declare both under the same name in one module, so there
is no identity collision risk in treating them identically for exposure purposes).

## What Changes

- `collect_item_exposures` gains a `syn::Item::ForeignMod` arm: each `pub fn`/`pub static` inside the
  block is walked through the existing `fn_seam`/`item_seam(ItemKind::Static, …)` + signature/type
  path collectors — verbatim reuse of the ordinary-item machinery, not new logic.
- `semantic-signature-coupling`'s existing "Public-signature observation governs exposure"
  requirement enumerates the observed surface but never mentions `extern` block items — a real
  textual gap this defect exposes, amended alongside the fix.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `semantic-signature-coupling`: amends "Public-signature observation governs exposure" to name a
  `pub fn`/`pub static` inside an `extern` block as part of the observed exposed surface, and adds
  scenarios for both forms plus the non-`pub` case.

## Impact

- Affected code: `crates/hunyi/src/collect.rs` only.
- No public API/DSL/builder change, no baseline format change (this fixes a false negative, not an
  identity shape — an adopter's existing baseline is unaffected either way).
- Out of scope, named explicitly rather than glossed over (an independent apply-stage review found
  the first draft understated this): the identical `Item::Fn`/`Item::Static` + `is_public` pattern
  with no `ForeignMod` arm also appears in `collect_item_async_exposures` and
  `collect_item_return_impl_traits` — **in this same file** — and in `collect_item_dyn_exposures`
  further down it. None of the three has its own audit reproduction, regression test, or spec-text
  amendment yet; fixing them here would be folding un-reproduced findings into an unrelated PR,
  against this project's own explore-before-propose discipline. Recorded as follow-up candidates,
  not fixed now. The visibility capability's own item observer
  (`crates/hunyi/src/syn_util.rs:439`) has a separate, already-tracked unverified audit finding for
  the identical shape.
