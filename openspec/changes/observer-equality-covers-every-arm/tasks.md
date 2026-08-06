# Tasks

## 1. The dimension array

- [ ] 1.1 Add the `Dimension` struct and a single `DIMENSIONS: [Dimension; 3]` to
      `crates/tianheng/tests/observer_protocol.rs`, each entry declaring the violating boundary measured in
      `design.md`.
- [ ] 1.2 Build the equality fixture by folding `declare` over `Constitution::new(...)`, and assert all three of
      the constitution's dimension accessors are non-empty — the guard against a deleted entry.

## 2. The equality reaction

- [ ] 2.1 Build the fold by folding `fold` over `Run::over(&manifest)`, so the fixture and the fold cannot name
      different dimension sets.
- [ ] 2.2 Assert, per dimension, that the built-in outcome holds a violation whose kind satisfies that
      dimension's `reacted` — failing with the dimension's label.
- [ ] 2.3 Keep the whole-outcome equality assertion between the two paths.
- [ ] 2.4 **Verify by perturbation**: replacing `SemanticObserver::observe`'s body with `Outcome::Clean` fails
      the reaction; same for `StaticObserver` and `RuntimeObserver`. Measured before this change: only the
      static one failed.

## 3. Whole-declaration bound comparison

- [ ] 3.1 Compare `declared_bounds()` against `exported_bounds()` as `Vec<BoundDecl>` sorted by id, per array
      entry.
- [ ] 3.2 **Verify by perturbation**: changing one declaration's extent in a dimension's `observation_bounds()`
      while leaving its id alone fails the reaction. Measured before this change: it passed.

## 4. Delegation of the runtime arm

- [ ] 4.1 `evaluate_constitution`'s runtime arm delegates to `RuntimeObserver`.
- [ ] 4.2 `cannot read workspace` appears **once** in the tree (`git grep -c` over tracked files).
- [ ] 4.3 The CLI's and `check_constitution`'s observable behaviour is unchanged: the full suite passes, and the
      self-governance projection is byte-identical.

## 5. Definition of Done

- [ ] 5.1 `bash scripts/check_all.sh` (or the repo's full gate set) clean.
- [ ] 5.2 `CHANGELOG.md` `[Unreleased]` records the reaction's strengthening — no version bump.
- [ ] 5.3 Sync the delta into `openspec/specs/observer-protocol/spec.md` and prune the dated archive copy.
