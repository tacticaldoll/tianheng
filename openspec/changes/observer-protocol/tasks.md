## 1. The protocol in 璇璣

- [ ] 1.1 `Observer` in `crates/xuanji/src/observer.rs`: `kind()`, `observe(&Path) -> Outcome`, `bounds()`.
      **No default body on any of them** (design D1) — that is the enforcement, not a style choice.
- [ ] 1.2 Name no `dyn` in the trait's own signature, so an adopter implements it concretely (D5).
- [ ] 1.3 Export from the crate root, and state in the trait's doc **why** it has three methods rather than the
      five an observation pipeline suggests: 三儀 ⊥ 三儀 forbids a shared scanner, so no dimension exposes those
      stages and every implementor would collapse them (D2).
- [ ] 1.4 Unit tests: a hand-written observer folds as expected; and record — rather than silently skip — that
      "an implementor omitting `bounds()` fails to compile" has no test, because the code expressing it does not
      compile.

## 2. Each dimension implements it

- [ ] 2.1 `guibiao`'s observer delegates to `guibiao::check` (the outcome-only face), **not** to
      `check_and_cover`: a second call would double the `cargo metadata` read (D4).
- [ ] 2.2 `hunyi`'s delegates to `check_all`.
- [ ] 2.3 `louke`'s derives member roots and the label anchor itself through `xingbiao`, as the runner does
      today, and delegates to `audit_probe_coverage`. Confirm the anchor is Cargo's resolved `workspace_root`
      with the same fallback, since that anchor is baseline identity.
- [ ] 2.4 Each observer's `bounds()` returns exactly its dimension's `observation_bounds()` — delegation, never a
      second list.

## 3. The fold in the shell

- [ ] 3.1 `tianheng::Run`: `over(constitution, manifest)`, `observe(impl Observer)` folding **eagerly**, and
      `verdict()`. Reuse `merge_outcomes` rather than reimplementing the invariant.
- [ ] 3.2 Refuse a run that composed no observer: composing nothing is a misconfiguration, not a clean run.
      Observe it failing.
- [ ] 3.3 **No `dyn` in any signature.** Do not add a trait-object exposure and do not add a boundary to
      "govern" one: measured, no module of `tianheng` is governed semantically today, and the dyn DSL has no
      allow-except form, so the declaration would have been a name with no reaction (design D5). Assert the
      absence mechanically — `grep` the crate's public signatures for `dyn` and expect nothing.
- [ ] 3.4 `check_constitution`, `run` and the CLI keep their exact path. No behaviour change under this change.

## 4. The two paths are held equal

- [ ] 4.1 A reaction asserts folding the three dimensions through the trait yields the same `Outcome` as
      `check_constitution` on this workspace. Observe it failing by perturbing one observer.
- [ ] 4.2 A reaction asserts each observer's `bounds()` equals its dimension's exported declarations, so the
      obligation cannot be satisfied by a divergent second list. Observe it failing.
- [ ] 4.3 Assert the fold's ordering directions on hand-written observers: a cannot-judge stops a later
      observer being evaluated; the earlier of two cannot-judges wins; violations from several merge into one
      report; all-clean is clean.

## 5. This capability's own bounds

- [ ] 5.1 `an_observer_may_under_declare_its_bounds` — a hand-written observer declaring one of its two limits
      is composed without complaint, demonstrating that the trait compels a declaration and not a complete one.
- [ ] 5.2 `the_fold_does_not_adjudicate_a_participant_s_verdict` — an observer returning a verdict at odds with
      the workspace it read is merged as given.
- [ ] 5.3 Classify both in `tianheng::observation_bounds()`, since this capability's reaction lives in the shell.
      The bijection installed by `observation-bound-model` will refuse the sync otherwise, which is the point.

## 6. Coherence

- [ ] 6.1 `CHANGELOG.md`: an `### Added` entry stating the new surface **and** that adopters migrate nothing.
      Note explicitly that a future stage addition is a breaking change by design.
- [ ] 6.2 `AGENTS.md`: no Definition of Done line — the reactions ride the existing `cargo test`. Name the new
      reaction beside the others in the self-governance paragraph.
- [ ] 6.3 `PROJECT.md`: record that the execution layer now has one seam, and that the fold is a fan-out with a
      short-circuit rather than a pipe, because 三儀 ⊥ 三儀 forbids the pipe.
- [ ] 6.4 `BACKLOG.md`: close the entry. Do **not** claim the protocol makes bound declarations complete — that
      is one of this capability's declared bounds, and the entry should say what changed and what did not.
- [ ] 6.5 The register's census guard will catch `CHANGELOG.md`'s bound count if the total moves. Correct it
      against the run's figures, never by recounting.

## 7. Verification — a guard is not a guard until it has been seen to fail

- [ ] 7.1 Record, for every assertion in tasks 3–5, the failure observed **without** the change: the offending
      state, the message, the exit status. In the pull request's `## Verification`.
- [ ] 7.2 Full Definition of Done, then again from a clean clone, since two reactions read tracked content.
- [ ] 7.3 Confirm the additive claim mechanically: no existing public signature moved, `cargo doc` clean, the
      packaged-tarball self-test green.
