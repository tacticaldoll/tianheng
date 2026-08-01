## 1. guibiao — thread the governed crate into `ModuleFact` identity

- [x] 1.1 Change `ModuleFact::into_finding(self)` (`crates/guibiao/src/finding.rs`) to
      `into_finding(self, governing_package: &str)`, adding `("governing_package", governing_package)`
      to every variant's `fact(...)` field list. **Do not** name it `"package"` — that literal is
      already used by `CrateFact::dependency`/`CrateFact::feature` (same file, lines 52/68/82) for
      the *observed dependency's* name, a different referent; reusing it would collide two meanings
      under one field name (caught in propose-stage adversarial review).
- [x] 1.2 Update `push_module_violation` (`crates/guibiao/src/module_check.rs:37`) to accept and
      forward `boundary.crate_package` into `finding.into_finding(...)`.
- [x] 1.3 Update all four `push_module_violation` call sites (`module_check.rs:265,338,405,477`) —
      each already has `boundary` in scope, so `&boundary.crate_package` requires no new plumbing.
      (The function's own signature was unchanged, so all four call sites needed no edits at all —
      only its body did.)
- [x] 1.4 Update guibiao's fact-compatibility catalog test (the `#[cfg(test)]` module in
      `finding.rs`, `assert_key`/`IntoFinding` helpers and `published_crate_fact_identity_schema_is_
      exact_and_exhaustive`-style test) to pass a `governing_package` value through `into_finding`
      and assert the new field appears in every module-fact case.

## 2. hunyi — thread the governed crate through the shared emitter

- [x] 2.1 Change `SemanticFact::into_finding(self)` (`crates/hunyi/src/finding.rs`) to accept
      `governing_package: &str`, adding `("governing_package", governing_package)` to every
      variant's `fact(...)` fields — **except** the variant `unsafe_confinement.rs` produces (check
      which `SemanticFact` variant that is before starting; do not add the field there — see 2.6).
- [x] 2.2 Add `crate_package: &'a str` to both `SingleModuleViolationContext` and
      `MultiModuleViolationContext` (`crates/hunyi/src/emit.rs`) — added uniformly to both structs
      even though `unsafe_confinement`'s own fact conversion won't consume it, so every caller's
      construction stays structurally the same.
- [x] 2.3 Update `push_single_module_violations` and `push_multi_module_violations` to pass
      `context.crate_package` into `finding.into_finding(...)`.
- [x] 2.4 Update every context-construction call site to supply `crate_package: &boundary.
      crate_package` — confirmed sites (verified during propose-stage review, re-grep both context
      struct names before calling this task done): `visibility.rs:45`, `exposure.rs:62`,
      `dyn_trait.rs:61` (Single); `async_exposure.rs:77` (Single) and `:54` (Multi);
      `impl_trait.rs:106` (Single) and `:70` (Multi); `forbidden_marker.rs:52` (Multi);
      `trait_impl.rs:66` (Multi); `unsafe_confinement.rs:54` (Multi, see 2.6). Exactly 10 sites,
      matching the count established in review.
- [x] 2.5 Update hunyi's fact-compatibility catalog test in `finding.rs` to pass a
      `governing_package` value and assert the new field for every affected `SemanticFact` variant.
      (`every_public_seam_shape_is_named_and_identity_injective`,
      `every_exposure_kind_has_its_exact_published_type_and_shape`,
      `every_semantic_fact_family_has_its_exact_named_identity_schema`,
      `every_async_seam_form_has_exact_structured_identity`, plus two direct-assertion tests in
      `tests.rs` — all updated and green.)
- [x] 2.6 Confirmed `unsafe_confinement.rs:55`'s `target: &boundary.crate_package` really is already
      crate-scoped (re-verified against current source, not just trusted from the review), and its
      `UnsafeSite` fact-construction site deliberately does **not** gain `governing_package` — a doc
      comment on `into_finding` plus an inline comment at the `UnsafeSite` branch in
      `into_finding_with_text` explain why, so a future reader doesn't "fix" the asymmetry.

## 3. Cross-crate collision regression

- [x] 3.1 Added `two_crates_with_the_identical_module_boundary_stay_distinct_violations`
      (`crates/guibiao/src/tests.rs`) mirroring `self_governance.rs`'s own shared-boundary shape: two
      synthetic workspace member crates (`alpha`, `beta`, real temp source trees so the module scan
      genuinely runs), each governed by the identical module path + rule, each independently
      violating it, evaluated together via `guibiao::evaluate`. Asserts the composed report contains
      **two** distinct violations, one per crate, each with its own `file`. (Scoped to module-boundary
      only, not extended to a semantic capability too — the mechanism is identical and hunyi's own
      catalog tests in 2.5 already prove its threading independently; a second full synthetic
      multi-crate semantic fixture would duplicate this coverage for no new risk surface.)
- [x] 3.2 Folded into the same test: writes a baseline from crate `alpha`'s violation alone, applies
      it to both crates' violations, and asserts exactly one violation stays unbaselined (`beta`'s) —
      it reacts as new rather than being suppressed.
- [x] 3.3 Non-vacuous verification performed directly: temporarily hardcoded a shared
      `governing_package` value in `push_module_violation`, reran the new test, confirmed it failed
      exactly as predicted (`left: 1, right: 2` — the collision reproduces byte-for-byte), then
      restored the real fix and confirmed it passes again. See this PR's Verification section.

## 4. Documentation

- [x] 4.1 Added a CHANGELOG `[Unreleased]` entry under `### Fixed`, marked **BREAKING**, naming the
      cross-crate collision this closes and stating that any existing module/semantic baseline must
      be regenerated with `--write-baseline` (identity gained a required field). No version bump —
      `Cargo.toml` stays at `0.3.0`, confirmed via `check_release_coherence.sh`.
- [x] 4.2 Confirmed the `structured-violation-identity` spec delta reads correctly against the
      landed code — no further text changes needed.

## 5. Definition of Done

- [x] 5.1 Ran the full local gate list from `AGENTS.md` — all green: `cargo build --workspace`
      (`--all-targets`); the three clippy passes (`--all-targets --all-features`, `--workspace`
      default-features, `-p louke` isolated); `cargo fmt --all --check`;
      `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` (all suites, 0 failed);
      both `cargo doc` passes (default and `--all-features`); `cargo deny check`
      (`advisories ok, bans ok, licenses ok, sources ok`); `scripts/test_release_coherence.sh` and
      `check_release_coherence.sh` (`ok release coherence (development: 0.3.0)`);
      `scripts/test_examples.sh` (`all examples reacted as declared`).
- [x] 5.2 Adversarial apply-stage review performed — see 3.3's revert-and-confirm-red, done for real
      (not merely asserted) and reported above.
