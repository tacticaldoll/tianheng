## Context

`check_reference_integrity.sh` already builds a Git-derived index containing every tracked path and ancestor.
Reference existence uses that index, but the earlier member census still globs the working tree. The two sources
can disagree when an illustrative or generated crate manifest is untracked.

## Goals / Non-Goals

**Goals:**

- Make member classification a projection of the existing tracked-path index.
- Prove an untracked crate manifest cannot change a clean verdict.
- Retain loud refusal when no tracked member exists.

**Non-Goals:**

- Parse Cargo workspace membership or change the existing `crates/<name>/Cargo.toml` member convention.
- Expand the recognized reference syntax.
- Change any public Rust surface.

## Decisions

Build the tracked index before the member census and enumerate exact `crates/<name>/Cargo.toml` entries from
that file. Reusing the index keeps one observation source and avoids a second Git command whose failure would
need a parallel exit path. A fixture will add an untracked manifest beside prose naming a missing path under
that crate; the verdict must remain clean, which fails against the current implementation.

## Risks / Trade-offs

- Moving the census after index construction changes failure order when both Git enumeration and member
  presence are broken. This is correct: the gate cannot know whether members exist until its tracked evidence
  has been built.
- This preserves the repository's directory convention rather than parsing Cargo membership. The change is
  intentionally limited to eliminating untracked-state influence already forbidden by the contract.
