## Context

The implementation already has the desired migration shape: reads preserve V1 and continue exact
text matching; an adopter-triggered `--write-baseline` observes current violations and writes V2.
Exact V1 matches can carry metadata forward, while stale entries are absent from the fresh snapshot.
The remaining gap is communication, not mechanism.

## Goals / Non-Goals

**Goals:**

- Make the existing bounded opt-in path discoverable where adopters learn baseline use.
- State the ordering constraint around presentation changes and the metadata consequence honestly.

**Non-Goals:**

- A migration subcommand, automatic read-time rewrite, read warning, deprecation clock, or V1 removal.
- Any change to matching, snapshot, metadata, or output behavior.

## Decisions

### Extend the existing adoption paragraph

The root README already owns the full `--write-baseline` / `--baseline` workflow. Adding the V1 note
there keeps one user journey rather than creating a migration guide for a one-command operation.

### Describe a conditional recommendation, not a forced migration

V1 remains supported. The documentation says to rewrite before presentation changes only when the
adopter needs existing suppressions and metadata carried forward. This exposes the real pressure
without inventing a deprecation policy.

## Risks / Trade-offs

- The guidance cannot identify V1 automatically. That is intentional: a perpetual runtime warning
  would add output behavior and pressure without a deprecation decision.
