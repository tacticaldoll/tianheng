## 1. Observation source — make the harness able to see what the act would record

- [x] 1.1 In `crates/kanhe/tests/merge_workflow.rs`, teach the controlled `gh` to **resolve** the body it is
      given the way the real tool does: `--body-file P` by reading `P`, `--body V` by taking `V`. Record the
      resolved body where a direction can read it, separate from the argument log.
- [x] 1.2 Make the controlled `gh` log its arguments **newline-safely**, so one invocation occupies one log
      line even when an argument contains newlines. Today it logs `"$*"`, which would split the merge
      invocation across lines once a multi-line body travels in `argv`.
- [x] 1.3 Add a controlled-`cargo` mode that rewrites the body file while standing where the gate runs, so the
      interval between the judgement and the merge is reproducible rather than argued about.
- [x] 1.4 Run the whole of `merge_workflow.rs` with the harness changed and the wrapper **unchanged**; every
      existing direction must still pass. Name in the pull request any assertion whose text had to move, and
      why — a log-shape change is a change to a dozen directions' observation source at once.

## 2. The direction, and its negative run

- [x] 2.1 Add the direction for the delta spec's scenario *The body file changes between the gate and the
      merge*: give the wrapper a body file, have the controlled `cargo` rewrite it, assert the body the merge
      invocation would record is the **judged** value. Assert the recorded body, never the flag name — a
      wrapper spelling `--body "$(cat "$body_file")"` at merge time re-reads and must still fail.
- [x] 2.2 Run that direction against the **unchanged** wrapper and record the observed failure verbatim. This
      is the negative run the pull request's `## Verification` must carry; a guard is not a guard until it has
      been seen to fail.

## 3. The repair

- [x] 3.1 In `scripts/merge-pr.sh`, replace `--body-file "$body_file"` with `--body "$body"` on the final
      `exec`. Nothing else moves: the read stays once, guarded, before the gate, with its cannot-judge refusal
      intact.
- [x] 3.2 Record in the pull request that `gh pr merge` accepts `-b, --body text` at **gh 2.95.0** — the same
      version the wrapper's allowlist is classified against — so the substitution is measured at a named
      version rather than assumed.
- [x] 3.3 Confirm the allowlist still refuses a caller's `--body`, `--body=*`, `--body-file`, `--body-file=*`,
      `-b*`, `-F*` in every spelling. The wrapper's own `--body` cannot be overridden by a later occurrence
      only because `passthrough` can never contain one; that safety belongs to the allowlist, not to the
      argument order.
- [x] 3.4 Sweep the tree for any other direction or document asserting the wrapper passes `--body-file`, and
      repair each — the retired spelling must not survive anywhere a reader would take it as current.

## 4. Verify the repair

- [x] 4.1 The new direction passes, and the whole of `merge_workflow.rs`, `merge_message.rs`,
      `gate_exit_classes.rs` and `gate_identity.rs` passes.
- [x] 4.2 Confirm the wrapper's exit classes are untouched: `2` for a misconfigured invocation or an input it
      could not read, `1` only for a gate that ran and refused.

## 5. Record

- [x] 5.1 Add the `CHANGELOG.md` entry under `### Self-governance` in `[Unreleased]`. State the guarantee and
      the interval it closes; the adopter-narrative reaction refuses a `scripts/` path under any of the eight
      adopter headings, and there is no adopter-visible guarantee to restate.
- [x] 5.2 Confirm the workspace version does **not** move and no manifest, pin or `Cargo.lock` entry changes;
      the bump belongs to release preparation.
- [x] 5.3 Run `TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test release_coherence` and confirm the
      changelog placement is clean.

## 6. Definition of Done and lifecycle

- [x] 6.1 Run the full Definition of Done block from `AGENTS.md` from the workspace root, including the
      env-gated `pin_bites`, `whitespace_hygiene`, `reference_integrity` and `examples_suite` lines.
- [ ] 6.2 Sync the delta into `openspec/specs/repository-checks/spec.md`, carrying the new scenario's
      observation evidence with it, and prune the dated archive copy.
- [ ] 6.3 Open the pull request into `release/0.5.0` with `## Why`, `## What changed`, `## Adversarial review`,
      `## Verification` (naming the commands run and the failure observed in 2.2) and `## Compatibility`.
- [ ] 6.4 Stop before merging. The squash is confirm-first, and it runs through `bash scripts/merge-pr.sh`.
