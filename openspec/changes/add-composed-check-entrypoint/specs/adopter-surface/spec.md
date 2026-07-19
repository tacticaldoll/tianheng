## MODIFIED Requirements

### Requirement: The prelude supports reaction inspection

`tianheng::prelude::*` SHALL expose `Boundary`, `BoundaryKind`, `Rule`, `ModuleRule`, `Baseline`,
`BaselineEntry`, `Finding`, `FindingKey`, `Outcome`, `Polarity`, `Report`, `Violation`, `ViolationId`,
the pure static `check` entrypoint, and the unified `check_constitution` entrypoint. These names SHALL
form an inspection tier, not a second construction path around builder-owned rule models. The
declaration/execution and inspection tiers SHALL have the same 0.2.x compatibility status; the tier
names distinguish purpose only.

#### Scenario: A consumer inspects a reaction

- **WHEN** an external crate receives an `Outcome` through a prelude entrypoint
- **THEN** it can inspect reports, violations, stable identity, finding data, polarity, boundary kind, and baseline metadata using the existing prelude names

#### Scenario: A consumer inspects the composed reaction

- **WHEN** an external crate imports the wildcard prelude and checks a unified `Constitution`
- **THEN** `check_constitution` returns an inspectable combined Outcome without driving CLI presentation

#### Scenario: Rule inspection remains builder-owned

- **WHEN** an external crate inspects `Rule` or `ModuleRule` through a builder-produced boundary
- **THEN** the prelude provides both read-side types without enabling direct construction of a non-exhaustive rule variant

#### Scenario: Module rule inspection is symmetric

- **WHEN** an external crate imports the wildcard prelude and matches the value returned by `ModuleBoundary::rule()`
- **THEN** `ModuleRule` is nameable beside `Rule` without an additional root or dimension-crate import
