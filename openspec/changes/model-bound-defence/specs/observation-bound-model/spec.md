## ADDED Requirements

### Requirement: A declaration's defence state SHALL match the register vocabulary

Every typed declaration SHALL carry exactly one `Defence`: either `PinnedBy { first, additional }`, with at least
one pinning-test slot, or `Unpinned { tracker }`. The two states SHALL be mutually exclusive in the type, matching
the register's `PINNED-BY` / `UNPINNED` grammar. Multiple `PINNED-BY` lines on one scenario SHALL all be retained
in declaration order. An unpinned declaration SHALL carry a tracker and no fabricated test name.

#### Scenario: A bound has no pinning test yet

- **WHEN** a declaration is created without a pinning test
- **THEN** it is expressible as `Unpinned` with its tracker, and cannot simultaneously claim `PinnedBy`

#### Scenario: One bound is defended by more than one test

- **WHEN** a scenario carries several `PINNED-BY` citations
- **THEN** the typed declaration retains every test while its pinned state keeps at least one test slot by construction
