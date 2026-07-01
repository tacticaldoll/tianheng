## 1. Model (guibiao)

- [x] 1.1 Add `pub enum SourceKind { Registry, Git, Path }` (Debug/Clone/Copy/PartialEq/Eq) with a stable string label each (feeds the rule's text/JSON projection).
- [x] 1.2 Add the `#[non_exhaustive]` `Rule` variant `RestrictDependencySourcesTo { allowed: Vec<SourceKind> }`.
- [x] 1.3 Handle the new variant in all four exhaustive `Rule` matches: `label` ("restrict dependency sources to"), `text` (list allowed kinds; empty → "forbid all dependencies (by source)"), `json_params` (`allowed_sources`), `findings` (see 2.x). Compiler enforces exhaustiveness.

## 2. Observation (guibiao cargo_metadata.rs)

- [x] 2.1 Add a classifier: given a dependency `Value`, return its `SourceKind` from the declared `source` — null → `Path`, starts with `git+` → `Git`, else → `Registry`. Mirror `external_dependencies`'s conventions (the non-null / `git+` reasoning documented inline; the residual stays robust to alt-registries).
- [x] 2.2 Add a helper returning the **real package names** (`dependency["name"]`, not `rename`) of the target's dependencies (of a given `DependencyKind`, via the existing `kind_matches`) whose classified `SourceKind` is not in an allowed set; wire it into `Rule::findings` for the new variant. Optional deps are included (they are in the declared `dependencies[]`).

## 3. Builder (guibiao)

- [x] 3.1 Add `CrateBoundaryBuilder::restrict_dependency_sources_to<I: IntoIterator<Item = SourceKind>>(self, allowed)` returning the existing `CrateBoundaryDraft` (so `.warn()`, `.dependency_kind(...)`, `.because(...)` compose unchanged). Doc states the two bounds: governs the **declared** source (patch/replace not observed — a future resolved capability); source-kind hygiene, not a publish oracle (a git/path dep with a `version` key is still flagged).

## 4. Shell projection (tianheng)

- [x] 4.1 Confirm the new rule projects through the existing generic `CrateBoundary` text/json/markdown projection (it reads `Rule::{label,text,json_params}`) with no per-rule code; add a projection test asserting the allowed source kinds appear.

## 5. Tests (guibiao)

- [x] 5.1 Classifier unit tests against synthesized dependency `Value`s: null→Path, `git+…`→Git, `registry+…`/`sparse+…`→Registry.
- [x] 5.2 Reaction tests (pure, against a synthesized `cargo metadata --no-deps` `Value`, as the existing static tests do): git dep violates `[Registry,Path]`; path dep violates `[Registry]`; all-registry clean under `[Registry]`; **optional git dep is governed (violates)**; **renamed git dep reports its real package name**; **`{git, version}` dep classifies Git and violates** (stated hygiene bound); **patch is not modeled at the declared layer — a registry dep stays Registry** (a `--no-deps` value with `source = registry+…` yields no violation, documenting the declared-vs-resolved bound); **an inherited workspace git dep reads `git+…` and is governed**; dev dep not governed by a Normal-scoped boundary.
- [x] 5.3 Severity + baseline parity; empty-allowlist forbids all; absent-target-crate constitution error (exit 2).

## 6. Self-governance & quality gates

- [x] 6.1 `guibiao` allowlist still `serde_json`-only (no new dep); self-governance (`tianheng_governs_itself`, `self_law_projection_is_fresh`) green.
- [x] 6.2 `cargo test --all-features --workspace`, `cargo clippy --all-features --all-targets`, `cargo doc --all-features`, default-feature `build`+`clippy` — all clean.

## 7. Docs

- [x] 7.1 `guibiao` README: add the declared dependency-source boundary beside the existing crate rules, with an example and the two stated bounds (declared not resolved; hygiene not publish oracle).
- [x] 7.2 Root README + `crates/tianheng/README.md` 圭表 dimension rows: add "declared dependency source" to what 圭表 observes.
- [x] 7.3 `BACKLOG.md`: note crate-source-boundary BUILT (v0.1.2) as a 圭表 **depth** addition (declared layer); **and record capability B — resolved dependency-source / build-provenance — as a named, deferred 圭表 depth** (resolved-layer read of `cargo metadata` with deps, catches `[patch]`/`replace-with`→git, misses optional-off; born when built). Update `PROJECT.md` with a terse decision recording the declared-vs-resolved two-layer distinction and why A (declared) is the right SSOT for manifest hygiene / publishability.
