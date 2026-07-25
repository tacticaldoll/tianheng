## ADDED Requirements

### Requirement: Mid-path relative import and symbol call dogfood coverage

The repository unit and integration test suites SHALL include lock assertions verifying that mid-path `super` and `self` imports inside grouped `use` trees, inline submodules, and inline symbol call paths are normalized to canonical `crate::...` paths and trigger module boundary and inline symbol confinement violations when targeting forbidden subtrees.

#### Scenario: Mid-path super grouped import triggers module boundary violation

- **WHEN** the test harness evaluates a module boundary against source containing `use crate::a::b::{super::forbidden::X};`
- **THEN** the harness detects `crate::a::forbidden` and reports the expected violation

#### Scenario: Mid-path super inline symbol call triggers confinement violation

- **WHEN** the test harness evaluates an inline symbol confinement against source containing `crate::a::b::super::forbidden::helper()`
- **THEN** the harness detects `crate::a::forbidden::helper` and reports the expected inline symbol call violation
