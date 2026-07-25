# runtime-origin-assertion Specification Delta

## Delta Requirements

### Requirement: CI face accepts configurable custom probe macro markers

The `audit_probe_coverage` scanner SHALL support custom probe macro marker names (`&[&str]`), defaulting to `["assert_boundary"]`. Each custom marker identifier SHALL be matched at a valid word boundary followed by optional whitespace and `!`, applying the same lexical scanning, seam argument decoding, and macro-body exclusion rules as `assert_boundary!`. A custom marker probe referencing a declared seam SHALL count toward probe coverage for that seam.

#### Scenario: A custom probe macro wrapper is recognized in CI coverage

- **WHEN** a project wraps `assert_boundary!` in a custom macro `company_seam!` and runs `audit_probe_coverage_with_markers` configured with `["assert_boundary", "company_seam"]`
- **THEN** calls to `company_seam!("seam-name", obj)` are scanned as valid auditable probes for seam `"seam-name"`

#### Scenario: Unregistered custom macro markers are ignored

- **WHEN** a file contains `other_macro!("seam-name", obj)` where `"other_macro"` is not in the configured marker list
- **THEN** the scanner ignores it and does not record a probe
