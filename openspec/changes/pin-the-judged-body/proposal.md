## Why

The squash wrapper judges a body it read into a variable and then tells `gh` to merge a body it reads from
a **path**, so the two are the same only while nobody touches the file in between — and the interval between
them is a whole `cargo test` run. The record that lands cannot be amended: a squash commit's hash is cited by
the pull request's merge record, so correcting it afterwards decouples the two.

Every other input the wrapper hands to that act is already pinned. The subject travels as a value, the
repository is resolved once and named on every call, the head is captured before the commit set and supplied
as `--match-head-commit`, and the live commit subjects are pinned through that head. The body is the one
input left as a reference the act re-resolves. The wrapper also **refuses a caller's** `--body` / `-F` / `-b`
for exactly this reason — *"this would have the gate judge one message and the merge write another"* — while
its own invocation reopens a narrower version of the same split.

## What Changes

- `scripts/merge-pr.sh` hands `gh pr merge` the body **value** it read and gave the gate, replacing
  `--body-file "$body_file"` with `--body "$body"`. The read stays where it is: once, guarded, before the
  gate, with its cannot-judge refusal intact.
- `repository-checks` gains the obligation that what the gate judged is what the act records, stated as a
  property of every judged input rather than as a fix to one of them, plus the scenario that falsifies it.
- A direction in `crates/kanhe/tests/merge_workflow.rs` observes that the wrapper's merge invocation carries
  no re-resolvable reference to a judged input.
- `CHANGELOG.md` records it under `### Self-governance`; the adopter-narrative reaction refuses a `scripts/`
  path under any of the eight adopter headings, and no adopter-visible guarantee moves.

Not breaking, and no version moves: `scripts/` ships in zero packages.

## Capabilities

### New Capabilities

<!-- none: the obligation belongs to the capability that already owns the squash wrapper -->

### Modified Capabilities

- `repository-checks`: the requirement *A squash message SHALL be judged before the merge that records it*
  gains the value-passing obligation — a judged input SHALL reach `gh pr merge` as the value the gate
  received, never as a reference the tool re-resolves — with the scenario for a body file that changes
  between the gate and the merge.

## Impact

- `scripts/merge-pr.sh` — one argument on the final `exec`. Nothing else in the script moves; the passthrough
  allowlist already refuses every caller spelling of a body flag, so the wrapper's own `--body` cannot be
  overridden by a later occurrence.
- `crates/kanhe/tests/merge_workflow.rs` — one direction, with its negative run recorded in the pull
  request's `## Verification`.
- `openspec/specs/repository-checks/spec.md` — one requirement extended, one scenario added.
- `CHANGELOG.md` — one entry under `### Self-governance`.
- No published crate, public signature, wire format, exit class, baseline, or manifest is touched.
