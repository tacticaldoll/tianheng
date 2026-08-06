# Tasks

## 1. The discriminant

- [x] 1.1 `BoundDecl::borrows_every_string()` in `crates/xuanji/src/bound.rs`, reaching the id, shape, pin,
      extent rationale and inherited layer name via exhaustive in-crate matches.
- [x] 1.2 A `xuanji` unit test asserting `false` for a computed string in **each** position independently, and
      `true` for a fully literal declaration.

## 2. The reaction over the family's declarations

- [x] 2.1 `observation_bound_model.rs` asserts it over every family declaration, naming any that allocates.
- [x] 2.2 **Verify by perturbation**: rewriting one family declaration's string as `format!(…)` fails the suite,
      naming that bound.
- [x] 2.3 The extent projection stays **byte-identical** — this change measures declarations, never alters one.

## 3. Definition of Done

- [x] 3.1 The full Definition of Done in `AGENTS.md` clean.
- [x] 3.2 `CHANGELOG.md` `[Unreleased]` records the API addition — no version bump.
- [ ] 3.3 Sync the delta and prune the dated archive copy.
