# Tasks

## 1. The bijection reads through the trait

- [ ] 1.1 `declared_bounds()` in `crates/tianheng/tests/observation_bound_model.rs` asks each dimension through
      `Observer::bounds`; the shell's own stay on its free function, with the asymmetry stated.
- [ ] 1.2 **Verify by perturbation**: one dimension's `bounds()` returning `Vec::new()` fails the bijection,
      naming the unclassified ids. Measured before: nothing read the method at all.

## 2. The third-party example

- [ ] 2.1 `examples/observer-participant/` — a crate outside the family implementing `Observer`, composed into a
      run with the dimensions via `Run::over(...).observe(...)`.
- [ ] 2.2 At least one declared bound's id is built with `format!` from what the participant observed.
- [ ] 2.3 Written against the **published** surface only. If an export is missing, report it rather than adding
      it.
- [ ] 2.4 A `tests/reaction.rs` binding the participant's contribution to the verdict — not the exit code alone,
      which the dimensions would carry on their own.
- [ ] 2.5 Wired into `scripts/test_examples.sh` with `fulfill_example`, and listed in `examples/README.md`.
- [ ] 2.6 **Verify by perturbation**: dropping the participant from the composed run leaves the reaction failing.

## 3. The COOKBOOK entry

- [ ] 3.1 A *Cross-cutting* entry showing the same shape at teaching size, pointing at the example.

## 4. Definition of Done

- [ ] 4.1 The full Definition of Done in `AGENTS.md` clean, including `test_examples.sh` and
      `test_published_family_coverage.sh`.
- [ ] 4.2 `CHANGELOG.md` `[Unreleased]` records it — no version bump.
- [ ] 4.3 Sync both deltas into `openspec/specs/*` and prune the dated archive copy.
