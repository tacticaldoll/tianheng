## ADDED Requirements

### Requirement: Mixed direct and conditional path remaps remain observable
When one file-module declaration carries both direct `#[path = "…"]` and one or more
`cfg_attr(..., path = "…")` remaps, 圭表 SHALL conservatively resolve every physically existing
written candidate and SHALL scan their union. Attribute order SHALL NOT silently remove a candidate,
and canonically identical candidates SHALL be evaluated once.

#### Scenario: Conditional remap after a direct remap is observed
- **WHEN** a module declares a direct path followed by a conditional path whose physical target contains a forbidden import
- **THEN** the forbidden import reacts even though current rustc configurations may select only one candidate

#### Scenario: Conditional remap before a direct remap is observed
- **WHEN** the same two remaps are written in the opposite order and either physical target contains a forbidden import
- **THEN** each physical candidate remains in the governed source union

### Requirement: Module projection preserves legacy depth shape
A legacy module boundary whose evaluation depth is `Subtree` SHALL omit `scan_depth` from its
projection so its JSON and derived Markdown remain byte-compatible. A boundary configured with the
non-legacy `Shallow` depth SHALL emit `scan_depth: "shallow"`.

#### Scenario: Legacy subtree projection is unchanged
- **WHEN** a module boundary is constructed without an explicit depth modifier
- **THEN** its projection contains no `scan_depth` field

#### Scenario: Shallow projection is explicit
- **WHEN** a module boundary is configured with `ScanDepth::Shallow`
- **THEN** its projection contains `scan_depth: "shallow"`

