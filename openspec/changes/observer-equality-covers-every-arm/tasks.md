# Tasks

## 1. The dimension array

- [x] 1.1 Add the `Dimension` struct and a single `DIMENSIONS: [Dimension; 3]` to
      `crates/tianheng/tests/observer_protocol.rs`, each entry declaring the violating boundary measured in
      `design.md`.
- [x] 1.2 Build the equality fixture by folding `declare` over `Constitution::new(...)`, and assert all three of
      the constitution's dimension accessors are non-empty — the guard against a deleted entry.

## 2. The equality reaction

- [x] 2.1 Build the fold by folding `fold` over `Run::over(&manifest)`, so the fixture and the fold cannot name
      different dimension sets.
- [x] 2.2 Assert, per dimension, that the built-in outcome holds a violation whose kind satisfies that
      dimension's `reacted` — failing with the dimension's label.
- [x] 2.3 Keep the whole-outcome equality assertion between the two paths.
- [x] 2.4 **Verify by perturbation**: replacing `SemanticObserver::observe`'s body with `Outcome::Clean` fails
      the reaction; same for `StaticObserver` and `RuntimeObserver`. Measured before this change: only the
      static one failed.

## 3. The bijection becomes a reaction over the delegation's shape

- [x] 3.1 Establish that the old comparison is inert: drifting a declaration's extent with its id untouched
      leaves the suite passing, and it is `observation_bound_model`'s extent projection that reacts instead.
- [x] 3.2 Each array entry names its observer's source; the reaction reads it through `support::region::Source`
      and requires `fn bounds`'s body to hold exactly the delegation, recognized by position between its braces.
- [x] 3.3 An absent method is a refusal to judge, not a pass.
- [x] 3.4 **Verify by perturbation**: replacing each of the three bodies with a list of its own fails the
      reaction, naming that dimension.

## 4. Delegation of the runtime arm

- [x] 4.1 `evaluate_constitution`'s runtime arm delegates to `RuntimeObserver`.
- [x] 4.2 `cannot read workspace` appears **once in executed code** over tracked files — remaining occurrences
      are prose about the change, which is the region distinction this family keeps re-learning.
- [x] 4.3 The CLI's and `check_constitution`'s observable behaviour is unchanged: the full suite passes, and the
      self-governance projection is byte-identical.

## 5. Definition of Done

- [x] 5.1 The full Definition of Done in `AGENTS.md` clean — build, three clippy passes, fmt, workspace
      tests under `TIANHENG_WORKSPACE_TESTS=1`, rustdoc, `cargo deny`, and all fifteen shell gates.
- [x] 5.2 `CHANGELOG.md` `[Unreleased]` records the reaction's strengthening — no version bump.
- [ ] 5.3 Sync the delta into `openspec/specs/observer-protocol/spec.md` and prune the dated archive copy.
