## 1. Fix the observation source

- [x] 1.1 Narrow `is_self_dependency` in `crates/guibiao/src/cargo_metadata.rs` to also require
      `dependency["source"].is_null()`, and update its doc comment to state the narrowed
      condition explicitly (path-only, not name-only).

## 2. Regression coverage

- [x] 2.1 Add a unit-level fixture test in `crates/guibiao/src/tests.rs` (alongside the existing
      `workspace_rule_never_flags_a_crates_own_self_referential_dev_dependency` and
      `no_dependency_rule_ever_flags_a_crates_own_self_referential_dependency`) proving a
      same-named `git`-sourced dependency is now flagged by `ForbidDependencyOn`,
      `RestrictDependenciesTo`, `RestrictDependencySourcesTo`, and `RestrictWorkspaceDependenciesTo`.
- [x] 2.2 Add a real-entry-point integration test under `crates/guibiao/tests/` that builds a
      hermetic probe crate (`foo` declaring `foo = { git = "https://example.invalid/foo.git" }`,
      `--no-deps`-only so no network is touched) and runs it through `guibiao::check` with the
      three rule constructors the audit names (`restrict_dependency_sources_to`,
      `restrict_dependencies_to([])`, `forbid_dependency_on(["foo"])`), asserting each now returns
      `Outcome::Violations(..)` with exit code 1, not `Outcome::Clean`.
- [x] 2.3 Re-run the existing self-path-dependency exemption tests unmodified and confirm they
      still pass (the legitimate exemption must survive the narrowing).

## 3. Verification

- [x] 3.1 Run the full Definition of Done command list from `AGENTS.md` and confirm every command
      passes.

## 4. Spec sync

- [ ] 4.1 Fold the `crate-dependency-boundary` delta spec into `openspec/specs/crate-dependency-boundary/spec.md`.
- [ ] 4.2 Prune the dated archive copy under `openspec/changes/archive/`, keeping only `.gitkeep`.
