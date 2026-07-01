## 1. DSL (hunyi)

- [ ] 1.1 Add `forbidden_operands: Vec<String>` to `DynTraitBoundary` (empty ⇒ shape-only / any dyn; non-empty ⇒ operand-scoped). `must_not_expose_dyn()` constructs it empty (unchanged behavior); add a getter mirroring the sibling boundaries' accessors.
- [ ] 1.2 Add `DynTraitModuleDraft::must_not_expose_dyn_of<I: IntoIterator<Item = &str /* or S: Into<String> */>>(self, operands)` returning the existing `DynTraitBoundaryDraft` (so `.warn()` / `.because()` compose unchanged), carrying the operand set. Doc states: empty ⇒ any dyn (loud, safe-by-direction); module-prefix operands are honored via the shared matcher; auto-trait markers are never operands.

## 2. Observation (hunyi)

- [ ] 2.1 Extract each `dyn` node's **principal trait** path: the **first `TypeParamBound::Trait`** in a `syn::Type::TraitObject`'s `bounds` (the base trait, guaranteed syntactically first by Rust's grammar — do NOT skip by name; a bare `dyn Send` correctly has principal `Send`). Take the path segments' idents only (drop generic/parenthesized args). Capture it alongside the rendered shape so the finding stays the shape string.
- [ ] 2.2 Add an operand-aware findings path (parallel to `dyn_module_findings`, but resolving like `module_findings`): resolve the module's `use` map + re-export closure, canonicalize each principal trait (`canonical_path_str` → `resolve_path(BareFallback::Ignore)` → `canonicalize_through_reexports`), and keep a `dyn` finding iff `forbidden_operands.is_empty() || matches_forbidden(canon, &forbidden_operands)`. The shape-only path stays resolution-free.

## 3. Reaction (hunyi)

- [ ] 3.1 Wire the operand filter into the dyn-trait `check_*` path: an empty operand set uses the existing shape-only findings (unchanged); a non-empty set uses the operand-aware path. Same `Violation` shape (finding = rendered `dyn …`), severity, and constitution-error behavior (unresolvable crate/module → exit 2).

## 4. Shell projection (tianheng)

- [ ] 4.1 Re-export `must_not_expose_dyn_of` surface as needed (the builder is reached via `DynTraitBoundary`; confirm no new re-export is required beyond the existing `DynTraitBoundary`). Add the operand set to the dyn-trait `list` JSON projection as a `forbidden` param **when non-empty**; confirm it flows through the generic markdown projection. Add a projection test asserting the operand set appears and that a shape-only boundary projects unchanged (no `forbidden` param).

## 5. Tests (hunyi)

- [ ] 5.1 Principal-trait matching (pure, against synthesized module sources as the existing dyn tests do): a `dyn` of a listed trait is flagged; a `dyn` of an unlisted trait passes; a **module-prefix** operand flags a trait under it; a **re-exported/aliased** operand matches its defining path; **auto-trait markers ignored** (`dyn Port + Send` flags on `Port`; forbidding only `Send` matches nothing); a `dyn` nested in `Box<…>` / generics is still matched by its principal trait.
- [ ] 5.2 **Empty operand degeneracy**: `must_not_expose_dyn_of([])` reacts to any `dyn` (identical to shape-only), never a silent no-op.
- [ ] 5.3 Shape-only unchanged: `must_not_expose_dyn()` still reacts to any `dyn` (regression guard).
- [ ] 5.4 Severity + baseline parity; unresolvable crate/module is a constitution error (exit 2).

## 6. Self-governance & quality gates

- [ ] 6.1 `hunyi` keeps its `syn` quarantine (no new dependency); self-governance (`tianheng_governs_itself`, `self_law_projection_is_fresh`) green.
- [ ] 6.2 `cargo test --all-features --workspace`, `cargo clippy --all-features --all-targets`, `cargo doc --all-features`, `cargo fmt --all --check`, default-feature `build`+`clippy` — all clean.

## 7. Docs

- [ ] 7.1 `hunyi` README: add the operand-scoped dyn boundary beside the shape-only one, with an example (`must_not_expose_dyn_of([...])`) and the stated bounds (empty ⇒ any; auto-trait markers not operands; resolver coverage bound).
- [ ] 7.2 Root README + `crates/tianheng/README.md` 渾儀 rows: note dyn exposure is now shape-only **and** named-operand (v0.1.2).
- [ ] 7.3 `BACKLOG.md`: mark operand-scoped dyn **BUILT (v0.1.2)** under 渾儀's forward depths (it was the named next depth). `PROJECT.md`: add a terse decision recording it as the named-operand depth of the shape-only dyn (the `name → shape → named-operand` stair), with the empty-⇒-any safe-by-direction choice.
