# tasks: Hunyi Observes cfg_if Arm Contents Implementation Plan

Implementation notes carried from the feasibility spike (see `design.md` for the measured results, so
none of this needs re-deriving):

- Arm extraction: for a `syn::Item::Macro` whose path's last segment is `cfg_if`, iterate
  `mac.mac.tokens.clone().into_iter()`, keep `TokenTree::Group` with `Delimiter::Brace`, and
  `syn::parse2::<syn::File>(group.stream())` each. A `#[cfg(..)]` predicate is a `#` plus a **bracket**
  group, so top-level braces are exactly the arms. Recurse when a parsed item is itself an
  `Item::Macro`. A parse failure degrades to "no items", never a panic.
- No brace/scope model is needed: `cfg_if!` in a function body is a statement, not `Item::Macro`, so
  the arm-versus-item-body problem 圭表 solved with `MacroScope` cannot arise.
- Candidate call sites consuming `&[syn::Item]`: `collect.rs` (public-API collection),
  `scan.rs::resolve_child_modules` and `scan.rs::collect_crate_root_extern_renames`,
  `module_resolve.rs::descend`, `crate_scope.rs::{local_type_namespace_names, child_module_names}`,
  `resolve/mod.rs::collect_uses`. Confirm which are reached before touching each; the goal is the
  smallest set that covers item collection plus the module walkers.

- [x] Add the transparent-macro arm-flattening helper (name test + arm extraction + recursion) beside `has_cfg_attr` in `crates/hunyi/src/syn_util.rs`, with the name gate documented as load-bearing and citing the measured `impl`-body-braces false positive. <!-- id: 0 -->
- [x] Apply flattening at the item-collection entry point so signature-coupling, visibility, dyn/impl-trait, and async-exposure observe arm items; verify each capability rather than assuming the shared entry covers all four. <!-- id: 1 -->
- [x] Apply flattening in the module walkers (`scan::resolve_child_modules`, `module_resolve::descend`) so arm-declared `mod` declarations enter the graph and their files are scanned. <!-- id: 2 -->
- [x] Treat an arm-declared module as cfg-conditional for absence tolerance, adopting 圭表's rule from `a567211`; confirm the ambiguity reaction (both conventional forms present) still fires under arm membership. <!-- id: 3 -->
- [x] Add regression coverage in `crates/hunyi/src/tests.rs` for the ten spike shapes — if/else, if-only, else-if chain, nested `cfg_if!`, arm `mod` and inline `mod`, paren-delimited invocation, `cfg_if!` inside an inline `mod`, a non-parsing generative body, and the `impl`-body-braces case proving the name gate holds. <!-- id: 4 -->
- [x] Add the control test without which an `expect_err`/`is_empty` assertion could pass vacuously: the identical exposure at module top level still reacts. <!-- id: 5 -->
- [x] Add a cross-dimension conformance ledger pinning 圭表 and 渾儀 on ONE fixture whose `cfg_if!` arm carries both a forbidden `use` (圭表's construct) and a forbidden exposure (渾儀's), asserting both react — and stating in its module doc that 漏刻 is deliberately absent until its own change lands. <!-- id: 6 -->
- [x] Write the two residual bounds into the spec delta: only `cfg_if` is transparent (with the reason), and arms are unioned cfg-blind so a non-selected arm's violation still reacts. <!-- id: 7 -->
- [x] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry, naming the measured exposure false negative, that 圭表 already surfaced its half in 0.2.x, and that new violations are baseline-absorbable. <!-- id: 8 -->
- [x] Run the full Definition of Done from the workspace root and report actual output, including the two isolated clippy passes, both release-coherence scripts, and `test_examples.sh`. <!-- id: 9 -->
