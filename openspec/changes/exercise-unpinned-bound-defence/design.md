## Context

`SpecDefence` parses both register states, and `BoundDecl::defence()` can return both typed states. The live family currently has no unpinned declarations, however, so `every_classification_cites_the_test_its_spec_cites` and `render_extents` execute only their pinned arms. The parser-only unit test proves the Markdown token is recognized but does not prove either typed consumer preserves the tracker.

## Goals / Non-Goals

**Goals:**

- Execute the typed `Unpinned` arm used by spec comparison.
- Execute the typed `Unpinned` arm used by the extent projection and assert its rendered tracker.
- Keep the live declaration set and generated projection unchanged.

**Non-Goals:**

- Adding an unpinned observation bound or backlog debt.
- Changing `Defence`, the register grammar, or public APIs.
- Refactoring unrelated projection or parser logic.

## Decisions

Extract the existing `Defence`-to-`SpecDefence` match into a private helper and call that helper from both the live comparison and a focused fixture test. This keeps one comparison implementation; duplicating the match in the test would execute a copy rather than the production test path.

Construct one local `BoundDecl::unpinned` fixture and render a one-entry map through the existing `render_extents` function. Assert the exact tracker-bearing line. The fixture is not returned by `declared_bounds()`, so it neither fabricates a live debt nor changes the checked-in projection.

Use separate focused tests for comparison and projection. This gives each formerly cold branch an independently attributable failure and verification record.

## Risks / Trade-offs

- [A private helper exists only to expose the real comparison path to a unit test] → Keep it as the sole implementation called by the live bijection test, so it removes rather than creates duplicated logic.
- [A fixture could be mistaken for a live bound] → Give it a clearly synthetic id and keep it local to focused tests rather than chaining it into `declared_bounds()`.
- [A substring assertion could pass on unrelated prose] → Assert the complete generated unpinned tracker line for the fixture.
