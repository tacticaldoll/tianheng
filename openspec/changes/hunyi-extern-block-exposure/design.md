## Context

Reproduced directly before designing the fix: a module with `extern "C" { pub fn handle() ->
crate::infra::Secret; pub static S: crate::infra::Secret; }` under a boundary forbidding
`crate::infra` produces zero findings for either declaration — only a sibling ordinary `pub fn
control()` in the same file reacts (`OUT: Ok(["crate::infra::Secret exposed by fn crate::api::control"])`).

`collect_item_exposures`'s match (`crates/hunyi/src/collect.rs`, ~line 260-484) enumerates
`Fn`/`Struct`/`Enum`/`Union`/`Type`/`Const`/`Static`/`Trait`/`Impl`/`Use`/`ExternCrate`, falling
through every other `syn::Item` variant — including `ForeignMod` — to a bare `_ => {}`.

## Goals / Non-Goals

**Goals:**
- A `pub fn`/`pub static` inside an `extern` block reacts exactly like the same-shaped ordinary item
  would: same seam shape (`fn_seam`/`item_seam(ItemKind::Static, …)`), same path-collection
  (`paths_in_signature`/`paths_in_type`).
- A non-`pub` extern-block item (no visibility qualifier) is NOT observed, matching every other
  item kind's own-visibility rule.

**Non-Goals:**
- Fixing the identical `Item::Fn`/`Item::Static` + `is_public` pattern's missing `ForeignMod` arm in
  `collect_item_async_exposures`, `collect_item_return_impl_traits` (both in this same file,
  `crates/hunyi/src/collect.rs`), or `collect_item_dyn_exposures` further down it — an independent
  apply-stage review confirmed all three have the identical gap. None has its own audit
  reproduction, regression test, or spec-text amendment; fixing them here would fold un-reproduced
  findings into an unrelated PR. Named explicitly as follow-up candidates, not silently left out.
- The visibility capability's own item observer (`crates/hunyi/src/syn_util.rs:439`) has a separate,
  already-tracked unverified audit finding for the identical shape — its own change, not this one.
- Any change to how a *body-having* item (a regular `fn`) is collected — untouched.

## Decisions

- **Reuse `fn_seam`/`item_seam` verbatim, no new seam kind.** An extern-block `pub fn`/`pub static`
  occupies the identical namespace slot an ordinary same-named item would (Rust forbids declaring
  both), so there is no identity collision to design around — the existing seam shapes already
  correctly distinguish this item from anything else in the module.
- **Spec text amendment, not left implicit.** `semantic-signature-coupling`'s existing requirement
  enumerates the observed surface exhaustively in prose but omits `extern` blocks entirely — matches
  the same textual-gap pattern found in the char-literal-brace-leak change (module-boundary's
  comment/string enumeration also omitted char literals) rather than treating this purely as an
  implementation bug against fully-stated behavior.

## Risks / Trade-offs

- **[Risk] A sibling hunyi capability has the identical gap, silently left unfixed by this narrowly-
  scoped change.** → **Mitigation**: explicitly named as a Non-Goal and left for the audit's own
  verification-pass phase — the unverified finding at `syn_util.rs:439` is the pointer to follow up
  on, not silently absorbed here without its own reproduction.

## Migration Plan

1. Land the `ForeignMod` arm in `collect_item_exposures`.
2. Add three regression tests: `pub fn`, `pub static`, and the non-`pub` control (must NOT react).
3. Verify non-vacuous: revert the fix, confirm the two positive tests fail (empty findings) while
   the non-`pub` control still passes, restore.

## Open Questions

None outstanding.
