## Context

The accepted shell dependency rule and its projection are correct, but the reason duplicates the
rendered parameters. This is a reason-only amendment: reaction identity and outcomes must remain
byte-equivalent apart from presentation of the reason.

## Goals / Non-Goals

**Goals:** restore imitable architectural direction and keep every clause within direct normal-edge
observation.

**Non-Goals:** change the allowlist, infer source-level composition, claim the shell is the only
composer, alter dependencies, or change a baseline.

## Decisions

The reason will describe the shell as an outward composition layer whose direct normal edges end at
observation dimensions and projection serialization, not at the lower reaction model or metadata
substrate. These are classifications of the exact allowed/forbidden direct edges; it will not claim
how calls or data flow transitively.

The generated projection will be refreshed solely through the blessed self-law test.

## Risks / Trade-offs

- **Layer prose could exceed observation.** → Restrict every clause to direct normal edges.
- **A reason edit could hide a semantic rule change.** → Diff the rule key/parameters and run the
  same forbidden-edge witness before and after.
- **Projection could become a second source.** → Never hand-edit it; bless from the constitution.

## Migration Plan

No product migration exists. Preserve accepted evidence, edit only the reason, regenerate, prove
identity/outcome stability, and obtain steward review through the authorized PR workflow.
