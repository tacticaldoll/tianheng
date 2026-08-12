## 1. The corpus

- [ ] 1.1 `crates/kanhe/tests/gate_exit_classes.rs` — both scans and the verdict window read one `Executed`
      region from `region::Source`, so a third scan inherits the decision.
- [ ] 1.2 `crates/kanhe/tests/dod_coherence.rs` — `ci_effective` is built from the classified region, closing
      the comment-only-command hole.

## 2. The acquisition sweep

- [ ] 2.1 Strip leading `NAME=value` tokens before the tool-name test, so an environment-prefixed acquisition
      enters the corpus.
- [ ] 2.2 Confirm the central gate invocation in **both** wrappers is now swept, and still reported guarded.

## 3. The negative runs

- [ ] 3.1 Perturb: remove a `|| {` guard from an env-prefixed acquisition and observe the sweep name it. Before
      the change it is silent.
- [ ] 3.2 Perturb: put a Definition-of-Done command into a YAML comment only, and observe the comparison name it.
- [ ] 3.3 Perturb: move the violation-class exit away from the verdict branch with a comment nearby, and observe
      the window reject it.

## 4. The residual

- [ ] 4.1 Declare the bound for the absence case and file its tracker entry in `BACKLOG.md`.
- [ ] 4.2 Record the rejected reaction with the measurement that rejected it, in the requirement.

## 5. The allowlist claim

- [ ] 5.1 `AGENTS.md` — stop half-enumerating the publish wrapper's forwarded and refused sets; name the parser
      as the owner and mark the remaining examples as examples.

## 6. Record and lifecycle

- [ ] 6.1 `CHANGELOG.md` under `### Self-governance`; confirm no version surface moves.
- [ ] 6.2 Full Definition of Done; sync the delta; prune the dated copy; PR; squash-merge through the wrapper.
