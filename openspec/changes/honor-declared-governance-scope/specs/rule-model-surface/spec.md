## ADDED Requirements

### Requirement: Every generic module rule honors its declared ScanDepth

Every Guibiao generic `ModuleBoundaryDraft` rule that exposes `.depth(ScanDepth)` SHALL use that
depth in its observation and matching. `Shallow` SHALL restrict the governed or permitted module
scope to the exact anchored seam; legacy `Subtree` SHALL retain `::`-delimited descendant matching.
No rule family MAY retain the selected depth only in projection, identity, or misconfiguration
checking while evaluating with a hard-coded subtree.

#### Scenario: Outbound rules honor Shallow

- **WHEN** `must_not_import` or `restrict_imports_to` is configured as `Shallow`
- **THEN** an import found only in a descendant module is outside the observation

#### Scenario: Inbound rules honor Shallow

- **WHEN** `must_not_be_imported_by` or `must_only_be_imported_by` protects a module with `Shallow`
- **THEN** an external importer of only a descendant module does not violate the exact-seam
  boundary, while importing the anchored module still reacts

#### Scenario: External confinement honors Shallow

- **WHEN** `confine_external_crate` permits an external crate at an anchored module with `Shallow`
- **THEN** that external crate remains forbidden in a descendant importer, while legacy `Subtree`
  permits the descendant
