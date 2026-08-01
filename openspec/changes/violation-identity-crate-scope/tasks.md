## 1. guibiao — thread the governed crate into `ModuleFact` identity

- [ ] 1.1 Change `ModuleFact::into_finding(self)` (`crates/guibiao/src/finding.rs`) to
      `into_finding(self, governing_package: &str)`, adding `("governing_package", governing_package)`
      to every variant's `fact(...)` field list. **Do not** name it `"package"` — that literal is
      already used by `CrateFact::dependency`/`CrateFact::feature` (same file, lines 52/68/82) for
      the *observed dependency's* name, a different referent; reusing it would collide two meanings
      under one field name (caught in propose-stage adversarial review).
- [ ] 1.2 Update `push_module_violation` (`crates/guibiao/src/module_check.rs:37`) to accept and
      forward `boundary.crate_package` into `finding.into_finding(...)`.
- [ ] 1.3 Update all four `push_module_violation` call sites (`module_check.rs:265,338,405,477`) —
      each already has `boundary` in scope, so `&boundary.crate_package` requires no new plumbing.
- [ ] 1.4 Update guibiao's fact-compatibility catalog test (the `#[cfg(test)]` module in
      `finding.rs`, `assert_key`/`IntoFinding` helpers and `published_crate_fact_identity_schema_is_
      exact_and_exhaustive`-style test) to pass a `governing_package` value through `into_finding`
      and assert the new field appears in every module-fact case.

## 2. hunyi — thread the governed crate through the shared emitter

- [ ] 2.1 Change `SemanticFact::into_finding(self)` (`crates/hunyi/src/finding.rs`) to accept
      `governing_package: &str`, adding `("governing_package", governing_package)` to every
      variant's `fact(...)` fields — **except** the variant `unsafe_confinement.rs` produces (check
      which `SemanticFact` variant that is before starting; do not add the field there — see 2.6).
- [ ] 2.2 Add `crate_package: &'a str` to both `SingleModuleViolationContext` and
      `MultiModuleViolationContext` (`crates/hunyi/src/emit.rs`) — added uniformly to both structs
      even though `unsafe_confinement`'s own fact conversion won't consume it, so every caller's
      construction stays structurally the same.
- [ ] 2.3 Update `push_single_module_violations` and `push_multi_module_violations` to pass
      `context.crate_package` into `finding.into_finding(...)`.
- [ ] 2.4 Update every context-construction call site to supply `crate_package: &boundary.
      crate_package` — confirmed sites (verified during propose-stage review, re-grep both context
      struct names before calling this task done): `visibility.rs:45`, `exposure.rs:62`,
      `dyn_trait.rs:61` (Single); `async_exposure.rs:77` (Single) and `:54` (Multi);
      `impl_trait.rs:106` (Single) and `:70` (Multi); `forbidden_marker.rs:52` (Multi);
      `trait_impl.rs:66` (Multi); `unsafe_confinement.rs:54` (Multi, see 2.6).
- [ ] 2.5 Update hunyi's fact-compatibility catalog test in `finding.rs` to pass a
      `governing_package` value and assert the new field for every affected `SemanticFact` variant.
- [ ] 2.6 Confirm `unsafe_confinement.rs:55`'s `target: &boundary.crate_package` really is already
      crate-scoped (re-verify, don't just trust this file), and confirm its fact variant's
      compatibility-catalog test entry deliberately does **not** gain `governing_package` — add a
      one-line comment at the fact-construction site (not a redundant field) explaining why, so a
      future reader doesn't "fix" the asymmetry by adding it back.

## 3. Cross-crate collision regression

- [ ] 3.1 Add a conformance test mirroring `self_governance.rs`'s own shared-boundary shape: two
      workspace member crates, each governed by the identical module path + rule (module-boundary
      first; extend to one semantic capability if the harness allows both cheaply), each
      independently violating it. Assert the composed report contains **two** distinct violations,
      one per crate, each with its own `file`.
- [ ] 3.2 Add the baseline-suppression regression: write a baseline for crate A's violation only,
      then introduce the identical-shape violation in crate B. Assert crate B's violation reacts as
      new (`baselined: false`, contributes to exit 1) rather than being suppressed.
- [ ] 3.3 Non-vacuous verification: temporarily revert the `into_finding` package-threading (or the
      context field) and confirm both 3.1 and 3.2 fail (the collision reproduces); restore and
      confirm both pass. Record this check in the PR's Verification section — an untested claim is
      itself a defect per this project's adversarial-review discipline.

## 4. Documentation

- [ ] 4.1 Add a CHANGELOG `[Unreleased]` entry under `### Fixed`, marked **BREAKING**, naming the
      cross-crate collision this closes and stating that any existing module/semantic baseline must
      be regenerated with `--write-baseline` (identity gained a required field). No version bump.
- [ ] 4.2 Confirm the `structured-violation-identity` spec delta reads correctly once the code
      matches it (no further text changes expected).

## 5. Definition of Done

- [ ] 5.1 Run the full local gate list from `AGENTS.md`: `cargo build --workspace`; the three
      clippy passes (`--all-targets --all-features`, `--workspace` default-features,
      `-p louke` isolated); `cargo fmt --all --check`;
      `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`; both `cargo doc` passes;
      `cargo deny check`; `scripts/test_release_coherence.sh` and `check_release_coherence.sh`;
      `scripts/test_examples.sh`.
- [ ] 5.2 Adversarial apply-stage review: confirm the declared reaction still bites the boundary the
      proposal claims (not a taste call) — specifically, that 3.3's revert-and-confirm-red actually
      happened and is reported, not merely asserted.
