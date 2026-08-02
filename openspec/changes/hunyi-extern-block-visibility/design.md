## Context

`item_observation_parts(item: &syn::Item) -> Option<VisibleItem<'_>>` (`crates/hunyi/src/syn_util.rs`)
answers "what does this direct item declare, if anything governed" with a `match` over every
`syn::Item` variant this capability cares about. `syn::Item::ForeignMod` (an `extern` block) has no
arm and falls to `_ => None`, so the whole block — and everything declared `pub` inside it — is
invisible to `visibility_findings`, regardless of the boundary's ceiling.

`ed19dce` fixed the analogous gap in signature-coupling's `collect_item_exposures`
(`crates/hunyi/src/collect.rs`), which already accumulates into a caller-owned `&mut Vec` per item,
so adding a `ForeignMod` arm that pushes zero-or-more entries required no shape change at that
function's boundary. `item_observation_parts` returns a single `Option<VisibleItem>` per source
item instead — every existing arm produces at most one observation, an invariant an `extern` block
breaks: it is one `syn::Item` that can hold an arbitrary number of independently-visible foreign
items (`pub fn open(); fn hidden(); pub static K: u8;` is one `ForeignMod`, three foreign items, two
of them independently `pub`).

## Goals / Non-Goals

**Goals:**
- Make every `pub`/restricted-visibility foreign item inside an `extern` block observable, at the
  same fidelity (ceiling ranking, rendered description) as a same-shaped ordinary item.
- Decide, deliberately, which `syn::ForeignItem` shapes are in scope for *this* capability, rather
  than copying signature-coupling's scope decision by default.
- Confirm the namespace/identity reasoning `ed19dce` used for signature-coupling actually
  transfers to visibility-boundary, rather than assuming it does because the shapes look similar.

**Non-Goals:**
- Extending signature-coupling's OWN extern-block coverage further (its `collect_item_exposures`
  already covers what it needs; this change touches only `syn_util.rs`/`visibility.rs`).
- Covering the identical `ForeignMod` gap in `async_exposure.rs`/`impl_trait.rs`/`dyn_trait.rs`'s
  own collectors — `ed19dce`'s own commit message named these as separate follow-up candidates,
  each needing its own audit reproduction and regression test; none is touched here.
- Giving a foreign item its own `VisibleItemKind` variant (see Decision 2).

## Decisions

### Decision 1: Widen the return type from `Option` to `Vec`, not from special-casing `ForeignMod` at the call site

Two shapes were considered:

1. Keep `item_observation_parts` returning `Option<VisibleItem>`, and give `visibility_findings`
   its own separate branch that special-cases `Item::ForeignMod` before ever calling
   `item_observation`.
2. Widen `item_observation_parts`/`item_observation` to return `Vec<...>`, so `ForeignMod` is just
   another `match` arm like every other item kind, and the one call site (`visibility_findings`)
   changes its combinator from `filter_map` to `flat_map`.

(2) is chosen. `item_observation_parts` is the single place this capability decides "what is a
governed item and what does it look like"; splitting that decision across two functions (one for
ordinary items, one for extern-block items, glued together only in the caller) would duplicate the
ceiling-ranking and rendering logic `item_observation` already applies uniformly, and every other
existing `syn::Item` arm can trivially return a one-element `Vec` — no behavior changes for them
(`vec![observed(...)]` in place of `Some(observed(...))`). The call-site change is one line
(`filter_map` → `flat_map`, with the inner closure moving `file` into a `.map`).

`item_observation`/`item_observation_parts` have exactly one caller each
(`visibility_findings`, confirmed by `grep -rn 'item_observation\b|item_observation_parts' crates/hunyi/src`
before this change) — no second consumer's call site needed auditing.

### Decision 2: Reuse `VisibleItemKind::Fn`/`Static`/`Type` verbatim — no new kind

`ed19dce`'s reasoning for signature-coupling: an extern-block `pub fn foo` and an ordinary `pub fn
foo` cannot coexist in the same module (Rust rejects the name collision — E0428, "the name `foo` is
defined multiple times"), so there is no identity to keep apart, and reusing the existing kind loses
no information a reader would need.

That reasoning is re-verified here rather than assumed to transfer by resemblance, because
visibility-boundary's finding text is rendered differently from signature-coupling's (`pub {kind}
{name}` vs. `{Type} exposed by {kind} {seam}`), so a diagnostic-clarity argument for a distinct
label would have to stand on its own:

- **Namespace identity holds identically.** An extern-block `fn`/`static`/`type` occupies exactly
  the same namespace slot (value namespace for `fn`/`static`, type namespace for `type`) as the
  ordinary item of the same kind — verified against `rustc`: `mod m { extern "C" { pub fn f(); }
  pub fn f() {} }` is a real E0428 in every edition. The ranking mechanism (`visibility_rank`) and
  the rendering (`vis_prefix`) operate purely on the `syn::Visibility` value, which is
  syntactically identical whether it decorates an ordinary or a foreign item — there is nothing
  about being a foreign item that changes what `pub`/`pub(crate)`/`pub(super)` *mean* for this
  capability's one question ("is the declared keyword above the ceiling").
- **A new kind is a stated compatibility cost, not a free label.** `VisibleItemKind`'s own doc
  comment: "its labels are published `item_kind` wire; keeping the variants typed makes a new
  governed item kind an explicit compatibility decision." Adding e.g. `ExternFn`/`ExternStatic`
  would fork the wire vocabulary for a distinction this capability's own question does not need
  (the ceiling comparison is identical either way), and would make visibility-boundary's rendering
  diverge from signature-coupling's own "no new seam kind" precedent on the *identical* underlying
  shape — an unforced inconsistency between two sibling fixes for what a reader would reasonably
  expect to be one recurring gap.
- **The rendered finding stays unambiguous without a new kind.** `pub fn open` (extern-block) reads
  no differently from `pub fn open` (ordinary) in a violation report, which is the same
  non-distinction `ed19dce` accepted for signature-coupling's `crate::infra::Db exposed by fn
  crate::m::open`. A reader repairing the violation opens the named file and sees which shape it is;
  the boundary's own concern ("do not declare this above the ceiling") does not change based on
  which shape declared it.

Conclusion: no new kind. `Fn`/`Static`/`Type` reused verbatim, matching Decision 2's sibling in
`ed19dce`.

### Decision 3: In scope — `ForeignItem::Fn`, `Static`, AND `Type`; out of scope — `Macro`, `Verbatim`

`ed19dce` scoped signature-coupling to `Fn`/`Static` only, because those are the two foreign-item
shapes that carry an exposable signature (a `pub fn`'s parameter/return types, a `pub static`'s
type). `ForeignItem::Type` (`type Foo;`, an extern type declaration — stable grammar in `syn`
regardless of the `extern_types` feature's own stabilization state, since `syn` parses grammar, not
feature-gated semantics) has no signature to leak — a bare `type Foo;` names nothing a forbidden
type could hide inside — which is presumably why `ed19dce`'s commit message frames its own
"probed further edge cases (foreign Type/macro items, …) without finding a break" as reassurance
that skipping `Type` cost signature-coupling nothing.

That reasoning does **not** transfer to visibility-boundary, whose question is not "does a
signature leak a forbidden type" but "is this item's declared visibility keyword above the
ceiling" — full stop, independent of whether the item has a signature at all. `pub type Foo;`
inside an `extern` block is exactly as much a bare-`pub` declaration as `pub type Foo = Bar;` at
module level already is (which `item_observation_parts`'s existing `Item::Type` arm already
observes). Scoping this fix to `Fn`/`Static` only — mirroring `ed19dce`'s scope by default rather
than by re-derivation — would leave a `pub type` inside an `extern` block as a live, undiscovered
instance of the exact false negative this change closes, discoverable by the next adversarial-sweep
round exactly as this one was.

Stated honestly: `rustc --edition 2021` confirms an extern-type declaration is grammar `syn` parses
unconditionally but a real build only accepts behind `#![feature(extern_types)]` (E0658, "extern
types are experimental" — verified, not assumed). This capability, like the rest of 渾儀, observes
`syn`-parseable source rather than only-stable source (the same posture that already lets it read a
`cfg`-gated-off branch neither compiler configuration compiles), so covering `Type` is still the
correct, complete answer to "what does this capability's own question require" — it is simply a
shape reachable today only on nightly-gated adopter code, not a claim that it is common.

`syn::ForeignItem`'s remaining two variants carry no visibility to react to at all:
- `ForeignItem::Macro` — a macro invocation (`ForeignItemMacro { attrs, mac, semi_token }`, no
  `vis` field). There is no keyword to observe; this is the extern-block analogue of this
  function's existing `#[macro_export]`/`pub macro` bound ("attribute-derived public surface …
  carries no readable visibility keyword and is out of scope").
- `ForeignItem::Verbatim` — unparsed `TokenStream` `syn` could not interpret as one of the above.
  Nothing is introspectable here by construction.

Both are handled by the `_ => None` fallthrough inside the new `ForeignMod` arm's own
`filter_map`, the identical shape this function already uses at its top level for every item kind
it does not govern.

## Risks / Trade-offs

- **[Trade-off] Widening `Option` to `Vec` touches every existing match arm's syntax** (`Some(x)` →
  `vec![x]`), even though only the new `ForeignMod` arm needs the plurality. This is mechanical and
  behavior-preserving for every pre-existing arm (each still produces exactly one entry), verified
  by the full pre-existing visibility test suite passing unchanged; the alternative (Decision 1's
  option 1) would have avoided touching those arms at the cost of duplicating the ranking/rendering
  logic in a second place.
- **[Closed] No existing fixture or example declares an extern block at all.** Checked across
  `crates/` and `examples/`: `grep -rn 'extern "C"' crates/*/src examples` finds none outside the
  new test fixtures this change adds, so the Definition of Done should not require touching an
  existing fixture.
- **[Accepted] Adopter-facing effect is a new violation, not a new exit code.** Unlike the
  dual-backed-module change (which added a constitution error), this closes a false negative
  within the existing violation/constitution-error contract: a workspace with a bare-`pub` foreign
  item now gets exit 1 instead of exit 0 where it previously slipped through, absorbable by
  baseline like any other newly-caught finding — the same compatibility class `ed19dce` itself
  claimed for signature-coupling's sibling fix.
