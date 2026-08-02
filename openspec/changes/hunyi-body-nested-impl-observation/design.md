## Context

`syn` parses `const _: () = { impl Svc { … } };` with the `impl` reachable only by descending into
the const's initializer expression (`syn::Expr::Block`'s `block.stmts`, where it appears as
`syn::Stmt::Item(syn::Item::Impl(_))`); a `fn`'s body holds it the same way in `item.block.stmts`.
Every one of 渾儀's item-collecting walkers reads a module's items from a flat `&[syn::Item]` — the
crate-wide walk (`scan.rs::walk_module`, feeding `ImplSite`/`trait_defs`/`type_defs`) and the
per-module resolver (`module_resolve.rs`'s two `resolve_module_items_with_*` functions, feeding the
four `collect.rs` collectors) alike — and neither descends into a `Const`/`Fn` item's own body.
`scan.rs`'s separate `UnsafeSiteCollector` does not share this gap: it is a `syn::visit::Visit`
implementation left at its default (fully recursive) behavior for everything it does not explicitly
override, so it already sees an `unsafe {}` block nested arbitrarily deep in any body — the asymmetry
that made the audit ask whether this was one root cause or several.

## Confirmed by direct reproduction

A throwaway probe (since promoted to the permanent regression tests below) built the exact audited
trigger shapes with the existing `TempSrcTree` fixture harness and ran the five affected capabilities'
pure entrypoints directly (`module_findings`, `async_exposure_module_findings`,
`dyn_module_findings`, `impl_trait_module_findings`, `trait_impl_findings`), plus
`forbidden_marker_findings` found during re-verification:

| shape | before this change | control (same method, unwrapped) |
| --- | --- | --- |
| `const _: () = { impl Svc { pub fn leak(&self) -> crate::infra::Db {…} } };` | `[]` | `["crate::infra::Db exposed by fn <crate::api::Svc>::leak"]` |
| `fn _also() { impl Svc { pub fn leak(&self) -> crate::infra::Db {…} } }` | `[]` | same |
| `const _: () = { impl Svc { pub async fn run(&self) {} } };` | `[]` | `["async fn <crate::m::Svc>::run(&self)"]` |
| `const _: () = { impl crate::command::Command for Rogue { fn run(&self) {} } };` | `[]` | `["crate::rogue (impl crate::command::Command for crate::rogue::Rogue)"]` |

Every wrapped shape produced zero findings; every unwrapped control reacted — ruling out a broken
fixture as the explanation and confirming a genuine false negative on ordinary, compilable source.

## Goals / Non-Goals

**Goals:**
- Recover the `impl` block itself wherever any of the six affected capabilities reads a module's
  items, without teaching any downstream matcher a new item shape (each already handles a top-level
  `Item::Impl` correctly today).
- State the scope bound explicitly rather than silently generalize it.

**Non-Goals:**
- Observing any OTHER item kind (`fn`, `struct`, `mod`, `trait`, `const`, `static`) nested directly in
  a body. Unlike an `impl`, these genuinely are scoped to the body and unreachable as `crate::…` —
  the dimension's own existing "a body-nested module is a stated bound" reasoning applies to them
  without qualification, and recovering them would be a NEW, unaudited claim this change does not
  make. Pinned by `a_plain_fn_directly_in_a_const_body_stays_a_stated_bound`.
- Recursing more than one level into the body (an `impl` inside an `if`/`loop`/closure/nested `fn`
  within the const/fn body). Both audited trigger shapes are exactly one level deep; nothing has
  shown a deeper shape in practice, and walking arbitrary expression trees for one would be a
  materially different, unaudited cost class — the identical "measurably unsound to generalize"
  reasoning `cfg_if!`'s own macro-name gate rests on, applied here to depth. Pinned by
  `an_impl_nested_one_level_further_stays_a_stated_bound`.
- Extending to `static` initializers. The const-eval trick is specifically about `const` — it forces
  compile-time evaluation even when the binding is never read, which is the entire point of the
  idiom — and no audited trigger or known real-world idiom uses `static` for it. Pinned by
  `a_static_wrapped_impl_stays_a_stated_bound`.
- A `cfg_if!`-nested `const`/`fn`. Not excluded by design — `body_nested_impls` runs on the item list
  AFTER `flatten_transparent_macros` already spliced in arm contents, so a `const`/`fn` written
  inside a `cfg_if!` arm composes for free, with no special-casing. Not a new goal to design for,
  just an emergent property of applying the new primitive after the existing one at each call site.

## Decisions

### Decision 1: One extraction primitive, held to `impl`-only, one-level, `const`/`fn`-only

`body_nested_impls(items: &[syn::Item]) -> Vec<syn::Item>` matches only `Item::Const` (whose
initializer is a bare `Expr::Block`) and `Item::Fn`, and within each, only a direct
`Stmt::Item(Item::Impl(_))` of that one block — never recursing further, never widening to another
item kind or to `Item::Static`. This is the narrowest primitive that covers both audited trigger
shapes; every wider variant considered (any nested item kind, arbitrary depth, `static` too) was
rejected as inventing tolerance for a shape neither finding exhibited (see Non-Goals).

### Decision 2: Splice at the two places items already enter observation, not a third mechanism

`scan.rs::flatten_for_walk` and `module_resolve.rs`'s two `resolve_module_items_with_*` functions
each already flatten `cfg_if!` arms before any capability reads the result; the extracted impls are
appended to that SAME returned list, at the SAME (module/file/branch) attribution as their enclosing
`const`/`fn` — correct, since extraction never crosses a file or module boundary. No third code path,
no new type threaded through the capabilities: every downstream matcher already handles a top-level
`Item::Impl` and needs no change.

### Decision 3: No arm-membership tag on an extracted impl

`FlatItem`'s `in_transparent_arm`/`arm_key` fields exist for exactly two consumers —
`resolve_child_modules`'s absence tolerance and `collect_reexports`'s cfg-aware child-module shadow
— and both match only `Item::Mod`/`Item::Use`. An extracted `Item::Impl` is therefore always wrapped
`FlatItem::plain` (now `pub(crate)` for this one external caller), never carrying a synthesized arm
key: nothing consults it, and inventing one would assert cfg-conditional membership this walk never
actually observed.

### Decision 4: Re-verified the consumer set rather than trusting the earlier read

Re-grepped every consumer of `scan.rs`'s `ImplSite` collection and every consumer of
`module_resolve.rs`'s two per-module resolvers before wiring anything, rather than assuming the
audit's five-capability list was complete. Found `forbidden_marker.rs` shares `scan.impls` with
`trait_impl.rs` (its hand-`impl T for X` form gets the identical fix for free) and confirmed
`visibility.rs` — which also consumes the per-module resolver — has no `Item::Impl` arm in its own
matcher (`syn_util::item_observation_parts`) at all, so it is structurally unaffected by the same
change. Both are stated in the proposal rather than left as an unstated side effect either way.

## Risks / Trade-offs

- **[Trade-off] New violations for an adopter using the const-eval trick for a real impl.** A
  false-negative closure, the class 0.2.2/0.2.3/`hunyi-cfg-if-transparency` already shipped as
  patches, absorbable by `Baseline`.
- **[Risk] Cloning.** `body_nested_impls` clones each recovered `impl` item, same cost class as
  `flatten_transparent_macros` already accepts for a build-time scanner; unmeasured, and the fix if
  it ever matters is laziness, not a different observation model.
- **[Bound, stated not silent] `impl`-only, one level, `const`/`fn`-only.** See Non-Goals; each has
  its own pinning regression test so the scope cannot silently widen in a later change without a
  test first failing to notice.
