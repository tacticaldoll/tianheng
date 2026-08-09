## Why

Three judgements can report clean over something they did not read, or could not see. Review found them; each
was verified against the code and one against a running git.

- **The publish gate's cleanliness answer depends on the machine.** `publish_source_gate.rs` defines a
  `hermetic()` builder and uses it for its **fixtures**; the judgement's own `git()` does not. A `core.excludesFile`
  outside the repository makes the exact status command return empty for an untracked file — reproduced. The
  fixtures are isolated and the verdict is not, which is the wrong way round.
- **Two enumerations in `release_coherence_gate.rs` drop I/O failures.** A failed directory entry is discarded,
  and an example manifest that exists but cannot be read is skipped **identically to one that is absent**. Other
  readable examples then satisfy the counters and the run reports clean.
- **A bound carrying two `UNPINNED` citations keeps the last and drops the rest.** The requirement is exactly
  one citation; the comment directly above the code describes this defect and the code implements it only for
  the `PINNED-BY`+`UNPINNED` pair.

## What Changes

- **`clean` is defined by the repository, not by the checkout.** A file ignored by **tracked** repository
  content is clean, because `cargo publish` would not package it either. A file hidden by this clone
  (`.git/info/exclude`) or this machine (`core.excludesFile`, including its XDG default) is **not** — the same
  commit must not get different verdicts in different places.
- The judgement's git runs hermetically and neutralises `core.excludesFile`; what no configuration can
  neutralise is classified by **source** rather than refused wholesale, so a legitimate `.gitignore` does not
  block a release.
- Both release-coherence enumerations propagate what they could not read as a cannot-judge, and skip only what
  genuinely is not there.
- Repeated `UNPINNED` becomes an invalid citation state naming the bound.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publish-source-integrity`: what *clean* means, and which exclusion sources a verdict may depend on.
- `release-coherence`: an enumeration SHALL NOT pass over content it failed to read.
- `observation-bound-register`: repeated `UNPINNED` is a citation answered twice.

## Impact

- **Amended**: `crates/tianheng/tests/support/publish_source_gate.rs`,
  `crates/tianheng/tests/support/release_coherence_gate.rs`,
  `crates/tianheng/tests/support/bound_register_parse.rs`, and the three matrices beside them.
- **Amended**: three specs, `CHANGELOG.md`, and the exempt-site census if any new site proves unconstructible.
- **Not changed**: what `cargo publish` packages, and the nine historical subjects. No version bump.
