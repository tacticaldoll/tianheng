# Tasks

## 1. The trait-object reaction states where it stops

- [x] 1.1 Extract the line matcher into a named recognizer over one line of text.
- [x] 1.2 Assert the premise: every subdirectory of `src/` is reached through a non-`pub` `mod` declaration.
- [x] 1.3 **Verify by perturbation**: making `mod runner;` public fails the reaction, naming the premise.
- [x] 1.4 Pin the residual: a test giving the recognizer a wrapped signature's continuation line and showing it
      is not recognized, alongside the line that is.

## 2. The declared bound

- [x] 2.1 A bound scenario in `observer-protocol`'s spec for the continuation-line residual.
- [x] 2.2 The matching `BoundDecl` in `crates/tianheng/src/bounds.rs`, pinned by the test from 1.4.
- [x] 2.3 `check_bound_register.sh` clean; the extent projection regenerated with exactly that entry added.

## 3. Polarity's absence is stated

- [x] 3.1 `Polarity`'s own doc comment says when `None` is the right answer, and that the alternative is
      compiler-enforced for the dimensions whose rules have a direction.
- [x] 3.2 The `runtime-origin-assertion` requirement, with its three scenarios.
- [x] 3.3 No reaction added for the by-construction half — an exhaustive match is the stronger guard, and a
      second copy of a fact the compiler holds can disagree with it.

## 4. Definition of Done

- [x] 4.1 The full Definition of Done in `AGENTS.md` clean.
- [x] 4.2 `CHANGELOG.md` `[Unreleased]` records both — no version bump.
- [ ] 4.3 Sync both deltas and prune the dated archive copy.
