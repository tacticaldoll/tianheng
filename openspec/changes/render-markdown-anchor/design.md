## Context

Markdown is rendered from the same JSON document as the machine projection. The renderer divides
each boundary object into structural fields and rule parameters with a hand-maintained key list.
`anchor` was added to the JSON base but not to that structural list, so it is currently rendered
inside the rule's parenthesized parameters. The existing constitution-projection contract instead
describes it as a distinct, Some-only element.

## Goals / Non-Goals

**Goals:**

- Restore the declared separation between durable governance metadata and rule parameters.
- Preserve byte-identical Markdown for boundaries that declare no anchor.
- Add a regression reaction that checks both exclusion from params and separate rendering.
- Restore clean `v0.2.3...HEAD` diff hygiene.

**Non-Goals:**

- Changing JSON shape, text projection, violation identity, or public Rust APIs.
- Pinning the complete Markdown layout byte-for-byte.
- Revisiting cfg-blind union scanning or any other review slice with no accepted finding.

## Decisions

- Add `anchor` to the renderer's structural-key set. This fixes the classification at the shared
  parameter filter rather than special-casing one rule or boundary kind.
- Render the anchor after the rule as its own Markdown list element. This keeps it distinct from
  the foregrounded reason and inline parameters while retaining the existing target → reason →
  rule → classification flow.
- Test semantic properties rather than snapshotting the whole document: the anchor must not occur
  in `boundary_params`, must occur as a standalone element in `boundary_markdown`, and absence must
  leave no element.
- Remove only the reported extra EOF blank lines; do not reflow authoritative prose.

## Risks / Trade-offs

- [Risk] A hand-maintained structural list can drift again when base metadata grows. → Keep the
  focused test named around structural base keys and make the anchor expectation explicit.
- [Risk] A broad Markdown snapshot would freeze presentation unnecessarily. → Assert element
  separation and presence/absence only.
