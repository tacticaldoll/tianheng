## 1. The judgement

- [ ] 1.1 In `crates/kanhe/src/gate_identity.rs`, add the judgement *which enumerated scripts carry no
      `--exact` citation at all*, returning the shared kinded `Refusal` — a `Violation` per uncited script,
      naming it. It reuses `citations`; it does not change `citations`, `logical_lines`, `registered_names` or
      `offences`.
- [ ] 1.2 Confirm the enumeration's own emptiness stays a **cannot-judge**: a corpus that never arrived is not
      a corpus in which every script cites a gate.

## 2. The failure matrix

- [ ] 2.1 In `crates/kanhe/src/tests/gate_identity.rs`, add rows beside the existing nine: a script that cites
      a gate is clean; a script that cites none is named; a script whose only invocation is commented out is
      named, composing with `a_commented_invocation_cites_nothing`; and one script citing twice does **not**
      excuse a sibling citing none.
- [ ] 2.2 Confirm the empty-corpus row is a cannot-judge rather than a clean verdict.

## 3. The repository direction

- [ ] 3.1 In `crates/kanhe/tests/gate_identity.rs`, assert the judgement over the tracked set, inside the
      existing enumeration rather than beside a second one.
- [ ] 3.2 Keep **both** existing vacuity guards. `!cited.is_empty()` is implied by the new direction only while
      the script set is non-empty, and dropping a floor because a sibling currently implies it is how one is
      lost.

## 4. The negative run

- [ ] 4.1 Track a citation-free script, run the repository direction, and record the observed failure verbatim
      — then revert. The matrix fails before the function exists, which says nothing about whether the
      direction is wired to the tree; this is the evidence that it is.
- [ ] 4.2 Confirm both tracked scripts pass unchanged, so nothing existing is refused.

## 5. Record

- [ ] 5.1 Add the `CHANGELOG.md` entry under `### Self-governance`, stating the closed-category consequence
      rather than only the repair.
- [ ] 5.2 Confirm the workspace version does not move and no manifest, pin or `Cargo.lock` entry changes.

## 6. Definition of Done and lifecycle

- [ ] 6.1 Run the full Definition of Done block from `AGENTS.md`, including the env-gated lines.
- [ ] 6.2 Sync the delta into `openspec/specs/repository-checks/spec.md` and prune the dated archive copy.
- [ ] 6.3 Open the pull request into `release/0.5.0` with the five required sections, then squash-merge it
      through `bash scripts/merge-pr.sh`.
