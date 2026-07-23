# rule-model-surface Specification Delta

## Requirements

### Requirement: Boundary builders SHALL expose explicit ScanDepth toggles

The public reaction model SHALL provide a strongly-typed `ScanDepth` enum (`Shallow`, `Subtree`, `Audit`) with `#[default]` set to `Shallow`. Boundary builders SHALL expose `.depth(ScanDepth)` to allow explicit configuration of observation depth. Existing ergonomic builders (such as `.including_submodules()`) SHALL map to `.depth(ScanDepth::Subtree)` and SHALL remain fully compatible.

#### Scenario: Default boundary construction uses Shallow depth

- **WHEN** a boundary is declared without an explicit depth modifier
- **THEN** its scan depth defaults to `ScanDepth::Shallow` and existing evaluation behavior is preserved

#### Scenario: Explicit depth configuration via ScanDepth enum

- **WHEN** a boundary builder is configured with `.depth(ScanDepth::Subtree)` or `.depth(ScanDepth::Audit)`
- **THEN** the boundary retains the specified depth and evaluates matching targets accordingly

#### Scenario: Existing builder ergonomics delegate to ScanDepth

- **WHEN** an adopter calls an existing modifier like `.including_submodules()`
- **THEN** the boundary configures its depth to `ScanDepth::Subtree` without breaking caller code
