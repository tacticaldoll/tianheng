## MODIFIED Requirements

### Requirement: A declaration's defence state SHALL match the register vocabulary

Every typed declaration SHALL carry exactly one `Defence`: either `PinnedBy { first, additional }`, with at least
one pinning-test slot, or `Unpinned { tracker }`. The two states SHALL be mutually exclusive in the type, matching
the register's `PINNED-BY` / `UNPINNED` grammar. Multiple `PINNED-BY` lines on one scenario SHALL all be retained
in declaration order. An unpinned declaration SHALL carry a tracker and no fabricated test name. The comparison
path SHALL be exercised with both states even when the live declaration set contains no unpinned entry.

#### Scenario: A bound has no pinning test yet

- **WHEN** a declaration is created without a pinning test
- **THEN** it is expressible as `Unpinned` with its tracker, and cannot simultaneously claim `PinnedBy`

#### Scenario: One bound is defended by more than one test

- **WHEN** a scenario carries several `PINNED-BY` citations
- **THEN** the typed declaration retains every test while its pinned state keeps at least one test slot by construction

#### Scenario: No live bound is currently unpinned

- **WHEN** the comparison reaction runs while every live declaration is pinned
- **THEN** a local unpinned declaration still exercises the same typed conversion used by the live comparison and preserves its tracker without fabricating a live bound

### Requirement: The extents SHALL be projected into a generated, staleness-checked document

The reaction SHALL emit a projection grouping every declared bound by its extent, blessed by an environment
variable and diffed on every run, in the manner `AGENTS.self-law.md` and `docs/observation-bounds.md` already
are. It SHALL lead with the count of declared false negatives, because that figure is the family's own audit
backlog and a number in a footnote is not read — the same reason the register's projection leads with its
unpinned count.

The projection SHALL state what it does not claim, in its own header. Its rendering path SHALL be exercised for
both defence states even when the checked-in projection contains no unpinned entry.

#### Scenario: The projection is stale

- **WHEN** a declaration's extent changes and the projection is not regenerated
- **THEN** the reaction fails and names the blessing command

#### Scenario: A reader can count the declared false negatives without reading code

- **WHEN** a reader opens the projection
- **THEN** the number of under-reacting bounds and their owners lead the document

#### Scenario: The live projection has no unpinned entry

- **WHEN** a local unpinned declaration is rendered independently of the live declaration set
- **THEN** the projection path emits its tracker in the register vocabulary without changing the checked-in projection
