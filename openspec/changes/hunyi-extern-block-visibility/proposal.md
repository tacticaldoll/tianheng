# proposal: Hunyi Extern-Block Visibility Observation

## Why

`ed19dce` (`fix(hunyi): observe pub fn/pub static inside an extern block`) closed a false negative
in 渾儀's **signature-coupling** exposure query: `collect_item_exposures` had no `syn::Item::ForeignMod`
arm, so a forbidden type named only in an `extern` block's `pub fn`/`pub static` escaped
`must_not_expose` entirely. That fix touched exactly one collector, in one file
(`crates/hunyi/src/collect.rs`).

The **visibility-boundary** capability (`must_not_declare_pub` / `max_visibility`) governs a
different question — not "does a signature leak a forbidden type" but "does this module declare an
item more visible than its ceiling" — and collects a module's direct items through a **separate**
per-item function: `item_observation_parts` in `crates/hunyi/src/syn_util.rs`. The two capabilities
share only the underlying module-item enumerator
(`crate::module_resolve::resolve_module_items_with_files` /
`resolve_module_items_with_cfg_tags`); each has always applied its own independent "what do we
observe" logic on top, matching the existing 渾儀 pattern where the crate-wide walk
(`scan.rs::resolve_child_modules`) and the anchored descent (`module_resolve.rs::descend`) are
independent siblings rather than one shared collector.

`ed19dce`'s fix therefore left `item_observation_parts` untouched. Measured with a
control/treatment probe (one `VisibilityBoundary`, the same `unsafe extern "C" { pub fn open(...);
pub static K: u8; }` block `ed19dce`'s own regression tests use for signature-coupling):

| capability | boundary | `extern` block | result |
| --- | --- | --- | --- |
| signature-coupling | `must_not_expose("crate::infra")` | `unsafe extern "C" { pub fn open(h: *mut crate::infra::Db) -> u8; }` | exit 1 (reacts — `ed19dce`) |
| visibility-boundary | `must_not_declare_pub()` | the identical block | **exit 0 (Clean)** |
| visibility-boundary | `max_visibility(Module)` (the strictest ceiling) | the identical block | **exit 0 (Clean)** — not a ceiling-rank artifact; the item is never observed regardless of ceiling |

`item_observation_parts`'s `match` has an arm for every other item kind (`Fn`, `Struct`, `Static`,
`Type`, `Mod`, `Use`, …) and falls through to `_ => None` for `Item::ForeignMod`, dropping the
entire block regardless of what is declared `pub` inside it. This reproduces identically for both
the 2024-edition `unsafe extern "C"` form and the plain 2021-edition `extern "C"` form — `syn`
parses both to the same `Item::ForeignMod`, so there is no edition-specific half of this gap.

`openspec/specs/semantic-visibility-boundary/spec.md`'s "Bare-pub item observation" requirement
already states the observed item-kind list exhaustively (`fn`, `struct`/`enum`/`union`, `type`,
`const`/`static`, `trait` incl. alias, `extern crate`, `mod`, `use`) and separately states "within
the observed scope there SHALL be no false negative" — the extern-block gap contradicts both: the
list omits extern-block declarations entirely, and an observed-kind item (`pub fn`) silently
escaped.

## What Changes

- Widen `item_observation`/`item_observation_parts` (`crates/hunyi/src/syn_util.rs`) from
  `Option`-returning to `Vec`-returning, and add a `syn::Item::ForeignMod` arm that walks each
  foreign item's own declared visibility through the same `VisibleItemKind`/`visibility_rank`/
  `vis_prefix` machinery every other item kind already uses — no new kind, for the same
  no-identity-collision reason `ed19dce` gave for signature-coupling (Rust cannot declare both an
  ordinary item and a foreign one under the same name in one module).
- Cover `ForeignItem::Fn`, `ForeignItem::Static`, **and** `ForeignItem::Type` — a deliberate scope
  decision, not an omission (see `design.md`): unlike signature-coupling, which only needed
  `Fn`/`Static` because only those carry an exposable signature, visibility-boundary cares about
  the declared keyword alone, and a bare `pub type` (an extern type declaration) is exactly that.
  `ForeignItem::Macro` (a macro invocation, no visibility keyword) and `ForeignItem::Verbatim`
  (unparsed tokens) stay out of scope, matching this function's existing attribute-derived/opaque
  bounds.
- Update `visibility_findings`'s one call site (`crates/hunyi/src/visibility.rs`) from
  `filter_map` to `flat_map` to consume the widened `Vec`.
- Add regression coverage in `crates/hunyi/src/tests.rs`: multiple `pub` foreign items in one block
  (exercising the `Option`→`Vec` widening itself, not just a single-item case the old shape could
  accidentally have passed), both edition forms, a `pub type` case, an all-non-pub control, and a
  restricted-visibility (`pub(crate)`/`pub(super)`) foreign item under a non-`Crate` ceiling.
- Modify the `semantic-visibility-boundary` spec's "Bare-pub item observation" requirement to name
  the extern-block-declared surface, phrased consistently with `ed19dce`'s own
  `semantic-signature-coupling` delta.
- Add a `CHANGELOG.md` `[Unreleased]` entry (`### Fixed`), placed beside `ed19dce`'s own entry.

## Capabilities

### Modified Capabilities

- `semantic-visibility-boundary`: the "Bare-pub item observation" requirement's observed item-kind
  list now names a `pub fn`/`pub static`/`pub type` declared inside an `extern` block.

## Impact

- `crates/hunyi/src/syn_util.rs`: `item_observation_parts`/`item_observation` return shape
  (`Option` → `Vec`) and a new `ForeignMod` arm.
- `crates/hunyi/src/visibility.rs`: `visibility_findings`'s one call site (`filter_map` →
  `flat_map`).
- `crates/hunyi/src/tests.rs`: five new regression tests, verified non-vacuous (all fail against
  the pre-fix code; the existing non-pub control and `ed19dce`'s own exposure tests are unaffected).
- `CHANGELOG.md`: `[Unreleased]` entry.
- Non-breaking: no public API, DSL, or wire-format change (`VisibleItemKind` gains no variant;
  `SemanticFact::Visibility`'s shape is unchanged). The adopter-facing effect is a new violation
  reachable only where a bare-`pub`/restricted-visibility item was written inside an `extern`
  block and previously escaped — absorbable by baseline like any other newly-caught finding.
