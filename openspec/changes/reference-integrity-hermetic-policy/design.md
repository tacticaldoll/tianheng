## Context

The gate owns a literal set of documents Tianheng must carry. Its matrix needs a smaller set only for the fixture
that deliberately removes every tracked Markdown and Rust file; otherwise the earlier required-document check
makes the zero-corpus direction unreachable. An environment override solved fixture construction by exposing
policy mutation to every caller.

## Goals / Non-Goals

**Goals:**

- keep Tianheng's required governance set invariant under ambient process state;
- retain a narrow, explicit seam for the zero-corpus fixture;
- prevent that seam from weakening the real workspace;
- fail loud on missing or unknown arguments.

**Non-Goals:**

- alter reference extraction or path resolution;
- change the required document membership;
- make fixture policy a supported adopter-facing interface.

## Decisions

### Literal real policy, explicit fixture argument

The script will always initialize the full literal set. A second argument named
`--fixture-governance-documents` may replace it only when judging a target other than the script's own physical
workspace root and only with a non-empty third argument. Additional argument shapes exit 2.

This makes policy changes visible in command history and test source. Comparing physical paths after entering the
target prevents a spelling difference from bypassing the own-workspace refusal.

## Verification

The matrix will poison `GOVERNANCE_DOCUMENTS` while a required document is absent and require the normal exit-2
diagnostic. Separate directions will prove the fixture option refuses this workspace, an empty set, and unknown
arguments. The zero-corpus fixture will continue reaching its intended refusal through the explicit seam.
