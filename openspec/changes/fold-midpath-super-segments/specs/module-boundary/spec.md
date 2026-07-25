## ADDED Requirements

### Requirement: Mid-path relative segment normalization

The system SHALL normalize embedded `self` and `super` segments appearing anywhere in an observed module import or symbol path (e.g. `crate::a::b::super::c` -> `crate::a::c`, `crate::a::self::b` -> `crate::a::b`). Over-popping `super` segments past the `crate` root SHALL resolve to `None` (an invalid path) and SHALL NOT produce a false-positive or false-negative boundary finding.

#### Scenario: A mid-path super import of a forbidden module is observed

- **WHEN** a governed module declares `use crate::a::b::{super::forbidden::Thing};` and a boundary forbids `crate::a::forbidden`
- **THEN** the system normalizes the import path to `crate::a::forbidden::Thing` and emits an enforced violation (exit 1)

#### Scenario: A mid-path self import is normalized to its parent module

- **WHEN** a governed module declares `use crate::a::{self::b::Thing};` and a boundary governs `crate::a::b`
- **THEN** the system normalizes the import path to `crate::a::b::Thing` and evaluates boundary rules against the canonical path
