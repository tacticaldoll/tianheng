## Why

The two sanctioned workflows that stand immediately before irreversible acts can currently judge incomplete evidence: the publish-source gate discards a failed remote read's cause, while the squash-merge wrapper can derive commit subjects from stale local remote-tracking refs. The first obscures why publishing cannot be judged; the second can silently let GitHub's default commit-list body escape the Rust gate.

## What Changes

- Preserve a failed live-remote read as an explicit cannot-judge carrying Git's error, distinct from a successful read that finds no `main` ref.
- Make the squash-merge wrapper obtain every pull-request commit subject from the live pull request, including fork heads and paginated commit lists, rather than from local `origin/*` refs.
- Refuse the merge workflow when that live commit evidence cannot be obtained or is empty.
- Add negative evidence at the changed observation levels: the publish refusal message and the merge wrapper's live commit acquisition.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publish-source-integrity`: distinguish a failed remote read from a successful response without `refs/heads/main`, preserving the failed read's cause.
- `rust-repository-reactions`: require the squash-message wrapper to judge the complete live pull-request commit set rather than a possibly stale local subset.

## Impact

- Affects `crates/kanhe/src/publish_source_gate.rs`, its publish-source tests, `scripts/merge-pr.sh`, and the squash-message workflow tests.
- Changes only unpublished repository-governance and workflow behavior; no published crate API, manifest version, or adopter migration changes.
- The sanctioned merge path will depend on GitHub's pull-request commits API already reached through the configured `gh` CLI.
