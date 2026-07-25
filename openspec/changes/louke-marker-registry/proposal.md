# proposal: Configurable Custom Probe Macro Markers in Louke CI Audit

## Intent

Allow adopters who wrap `louke::assert_boundary!` inside custom project-specific macros (e.g. `my_assert_seam!(...)`) to register custom macro marker names with `louke`'s `audit_probe_coverage` scanner, preventing custom probe wrappers from being missed or reported as un-auditable during CI coverage audit.

## Motivation

Currently, `louke`'s CI audit scanner (`crates/louke/src/audit/scan.rs`) hardcodes the probe marker identifier to `b"assert_boundary"`. When an adopter wraps `assert_boundary!` inside a custom macro (such as `company_assert_seam!("seam", obj)`), the CI audit scanner ignores the outer call site, reporting declared seams as unprobed (a false-positive audit failure) or failing to verify coverage.

Allowing `audit_probe_coverage` to accept a configurable list of custom marker names (defaulting to `["assert_boundary"]`) allows custom macro wrappers to be audited cleanly while preserving `louke`'s `syn`-free, zero-allocation byte scanner performance and core governance contract.

## Proposed Changes

1. **`crates/louke/src/audit/scan.rs`**: Refactor probe marker matching to scan against a list of allowed marker identifiers (`&[&str]`), retaining identical word-boundary, literal/comment skipping, and macro-body exclusion rules.
2. **`crates/louke/src/audit.rs`**: Expose `audit_probe_coverage_with_markers` (or options builder) accepting custom marker lists.
3. **`crates/tianheng/src/`**: Wire custom marker options through the shell if configured.
4. **`openspec/specs/runtime-origin-assertion/spec.md`**: Add requirement for configurable custom probe macro markers in the CI face.

## Compatibility

Non-breaking extension. Default marker list remains `["assert_boundary"]`, preserving 100% backward compatibility for all existing adopters and examples.
