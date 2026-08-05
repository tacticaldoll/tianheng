## 1. A cited pinning test must resolve to a test

- [x] 1.1 Probe first: confirm on the current tree that every `PINNED-BY` citation resolves to a definition
      whose attribute run contains `#[test]`, and that `#[should_panic]` interleaving exists in the tree, so
      the rule is written against a measurement rather than an assumption.
- [x] 1.2 In `scripts/check_bound_register.sh`, make `definitions_of` (or its caller) reject a resolved
      definition whose upward attribute run carries no `#[test]`, walking past attributes, comments, and
      blank lines. The refusal names the bound id and the definition site.
- [x] 1.3 Add the refusal fixture to `scripts/test_bound_register.sh`: a citation resolving to exactly one
      plain `pub fn` of that name fails with exit 1.
- [x] 1.4 Add the passing fixture: a citation whose test carries `#[test]` then `#[should_panic]` above the
      `fn` resolves, so the attribute-run read is pinned rather than incidental.
- [x] 1.5 Update the existing fixtures' Rust so the passing directions still pass, then run
      `bash scripts/test_bound_register.sh` and `bash scripts/check_bound_register.sh`.

## 2. An unpinned citation must name a tracked path

- [x] 2.1 In the reaction, require an `UNPINNED` citation to contain a token that `git ls-files` resolves
      inside the judged repository. Failing that, fail naming the bound id.
- [x] 2.2 Add the refusal fixture: `- **UNPINNED** no test exists` fails with exit 1.
- [x] 2.3 Add the second refusal fixture: a tracker naming an untracked path fails with exit 1.
- [x] 2.4 Confirm the existing passing `UNPINNED` fixture still passes, and that all five real citations in
      `openspec/specs/` still pass, by running both register lines.

## 3. Regeneration carries the exit contract

- [x] 3.1 Move the `declared -gt 0` cannot-judge condition **ahead** of the projection write, so a vacuous
      register produces no document.
- [x] 3.2 Make the blessing path write the projection and then fall into the shared verdict rather than
      exiting 0, with a message that distinguishes "regenerated" from "valid".
- [x] 3.3 Rework the fixture helper in `scripts/test_bound_register.sh` to assert the projection **exists**
      after blessing instead of asserting exit 0, and hand-prepare the projection for the `no-bounds`
      fixture, which can no longer be blessed.
- [x] 3.4 Add the two directions as fixtures: blessing a register with an offense writes the projection and
      exits 1; blessing a register with no declared bound writes nothing and exits 2.

## 4. The shared-bound claim narrows to what it observes

- [x] 4.1 Verify the two historical restatements' actual shapes in `v0.4.0` so the corrected record is a
      fact: which capability stated the `#[path]` bound as prose and which as a scenario.
- [x] 4.2 Verify the live same-heading pair across the two operand-boundary capabilities, so the rejection
      of a similarity key rests on a case in the tree rather than on a hypothetical.
- [x] 4.3 Add the second floor to `render_projection`'s header, beside the undeclared-prose floor.
- [x] 4.4 Regenerate `docs/observation-bounds.md` with `BLESS=1` and confirm the non-blessing run is clean.

## 5. Sync, then the spec's own Purpose

- [ ] 5.1 Sync the delta spec into `openspec/specs/observation-bound-register/spec.md`.
- [ ] 5.2 Replace the archive-generated `TBD` Purpose with the capability's actual purpose — the only spec
      of 30 still carrying the placeholder.
- [ ] 5.3 Run `openspec validate --specs --strict`.

## 6. Definition of Done

- [ ] 6.1 Run the full DoD gate list from `AGENTS.md`, not only the register's two lines.
- [ ] 6.2 Update `CHANGELOG.md` under the unreleased heading.
- [ ] 6.3 Archive the change, prune the dated archive copy so only `openspec/changes/archive/.gitkeep`
      remains tracked, and open one squash PR into `release/0.4.1`.
