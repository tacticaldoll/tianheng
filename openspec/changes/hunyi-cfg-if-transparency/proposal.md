# proposal: Hunyi Observes cfg_if Arm Contents

## Why

圭表 treats `cfg_if!` as a **transparent control-flow macro**: its arms wrap human-authored items
without transforming identities, so enclosed `use`, `mod`, and inline symbol paths are observed as
real code (shipped 0.2.3; the arm-membership cfg-conditional rule completed in the sibling change
`a567211`). 渾儀 has no such notion. `syn` parses a `cfg_if!` invocation as an opaque
`syn::Item::Macro`, and nothing in 渾儀 handles that variant — `grep 'Item::Macro' crates/hunyi/src`
finds only a `Type::Macro` rendering helper.

Measured with a control/treatment probe (one `SemanticBoundary`, one forbidden type, same file):

| `crate::child`'s body | 渾儀 | 圭表 |
| --- | --- | --- |
| `pub fn leak() -> crate::forbidden::Thing` at top level (control) | exit 1 | exit 1 |
| the identical function wrapped in `cfg_if! { if #[cfg(unix)] { … } else { … } }` | **exit 0** | exit 1 |

Same file, same `cfg_if!` block: 圭表 reacts, 渾儀 does not. That is an exposure **false negative** —
the one bug the core contract forbids — and it is reachable on ordinary, compilable source, since
`cfg-if` is a foundational ecosystem crate any platform-branching adopter is likely to use.

The gap is not confined to item collection. `mod` declarations inside arms are equally invisible, so a
module declared only inside a `cfg_if!` arm is absent from 渾儀's crate-wide walk — meaning `unsafe`
sites, forbidden markers, and trait impls in that module's file are never scanned either. 圭表 already
observes those declarations (`declared_modules_observes_mod_inside_cfg_if_macro_body`).

## What Changes

- Flatten transparent-macro arms into the item stream 渾儀 already walks: for a `cfg_if!`
  `Item::Macro`, take each top-level brace group of `mac.tokens` (the arms — a `#[cfg(..)]` predicate
  is a `#` plus a *bracket* group, never a brace), parse each with `syn::parse2::<syn::File>`, and
  recurse for a nested `cfg_if!`. Feasibility spike-verified across ten shapes; see `design.md`.
- Apply the flattening at both item-walk families: item collection (signature-coupling, visibility,
  dyn/impl-trait, async-exposure) and the module walkers (`scan::resolve_child_modules`,
  `module_resolve::descend`), so arm-declared modules enter the graph.
- Treat an arm-declared module as **cfg-conditional** for absence tolerance, adopting the rule 圭表
  settled in `a567211` rather than re-deriving one: the arm's predicate lives in the macro header, and
  every arm is conditionally compiled by construction.
- Gate on the macro **name** (`cfg_if`), matching 圭表. This is load-bearing, not conservatism: applied
  to arbitrary macros, arm extraction reads an `impl Foo { … }` body's braces as an arm and invents
  items from it (measured — see `design.md`).
- State the two residual bounds in the spec instead of leaving them silent: only `cfg_if` is
  transparent, and observation is a cfg-blind union so a violation in a non-selected arm still reacts.

## Capabilities

### Modified Capabilities

- `semantic-signature-coupling`: the anchor- and crate-resolution properties it already documents on
  behalf of every semantic capability gain transparent-macro arm contents, with the name-gating and
  cfg-blind-union bounds declared.

## Impact

- `crates/hunyi/src/syn_util.rs` (or a new sibling): the arm-flattening helper and the transparent-name
  test.
- `crates/hunyi/src/collect.rs`, `crates/hunyi/src/scan.rs`, `crates/hunyi/src/module_resolve.rs`: the
  call sites that consume `&[syn::Item]`.
- `crates/hunyi/src/tests.rs`: the ten spike shapes as regression coverage, plus the arm-declared
  module and cfg-conditional cases.
- `crates/tianheng/tests/`: a cross-dimension conformance ledger pinning 圭表 and 渾儀 on one fixture.
- `CHANGELOG.md`: `[Unreleased]` → `### Fixed`.
- Non-breaking: no public API, DSL, or wire-format change. Adopters using `cfg_if!` may see new
  violations — a false-negative closure, the same class 0.2.2/0.2.3 shipped as patches, absorbable by
  baseline.
- 漏刻's own two scanning passes are **out of scope** and sequenced after this: see `design.md`.
