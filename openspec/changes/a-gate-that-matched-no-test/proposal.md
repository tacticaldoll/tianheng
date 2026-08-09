## Why

Both wrappers standing in front of an act that cannot be undone reach their gate the same way:

```sh
cargo test … --test publish_source -- --exact the_publish_source_is_the_signed_release_snapshot
cargo test … --test merge_message  -- --exact the_squash_message_is_the_pull_request_it_records
```

**`libtest` exits 0 when the filter matches nothing.** Measured against a prebuilt binary on this tree, an
unknown name reports `0 passed; 0 failed; … filtered out` and exits `0`. So renaming, moving, or
`#[ignore]`-ing either gate disarms its wrapper **silently**, and the script proceeds to
`cargo publish --workspace` / `gh pr merge --squash`.

Nothing pins either name. Each appears in exactly two places — the script and the `fn` — and
`reference-integrity` holds references to *paths*, never to test identifiers.

This is the one bug the Core Contract forbids, at the two places nothing can be undone: a gate that judged
nothing reports the same exit status as a gate that judged and found nothing wrong.

## What Changes

- **Both wrappers require the run to report exactly one passing test**, printing what they saw when it does
  not. This is the load-bearing guard, because it stands where the act is launched — the position
  `scripts/publish.sh` already gives its environment scrub, and for the same reason: a guard the disarming
  could itself disable is not a guard.
- **A reaction pins the identity.** For every tracked shell script, each `--exact <ident>` is joined to the
  `--test <target>` of the same invocation, and that target must register the name exactly once. A test
  identifier is a reference into this repository exactly as a path is, and the reference gate matches paths
  only.
- The new reaction's refusal sites join the enumeration `refusal_bites` perturbs.

Both are needed and neither substitutes for the other. Measured: `--list` includes an `#[ignore]`d test, so
the reaction cannot see a silenced gate; `--exact` on one reports `0 passed; 1 ignored` and exits 0, so the
wrapper can. One covers the rename, the other covers the silencing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rust-repository-reactions`: a gate a wrapper asks for SHALL be observed to have run, and the identifier it
  is asked for by SHALL be pinned. Its subject covers `scripts/*.sh` and the reaction added under
  `crates/kanhe/`.
- `publish-source-integrity`: `cargo publish` is reachable only from a source where its stated conditions
  hold, and a wrapper whose gate did not run reaches it anyway. Its subject covers `scripts/publish.sh`.

`release-coherence` claims `CHANGELOG.md`, which records this change under `[Unreleased]`. Recording a change
is what that capability requires; nothing about its requirements changes.

## Impact

- `scripts/publish.sh`, `scripts/merge-pr.sh` — the gate invocation grows a checked result.
- `crates/kanhe/` — one new reaction with its failure matrix and its shared judgement module.
- No public API changes. No version change.
