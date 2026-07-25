## ADDED Requirements

### Requirement: Transparent control-flow macro body unstripping

The system SHALL recognize transparent control-flow macros (specifically `cfg_if!`) during macro body stripping and SHALL NOT remove their inner structural body contents. Enclosed `use` import declarations, `mod` module declarations, and inline symbol call paths inside `cfg_if!` macro bodies SHALL be observed by `use_scan`, `reachability`, and `symbol_scan` as real items, matching the system's cfg-blind union-scanning policy. Other code-generating or declarative macro bodies (`macro_rules!` definitions and non-transparent macro invocations) SHALL continue to be stripped as macro-generated items.

#### Scenario: A use declaration inside a cfg_if macro body is observed

- **WHEN** a governed file contains a `cfg_if!` macro invocation containing `use crate::projection::Thing;` and a boundary forbids `crate::projection`
- **THEN** the system observes `crate::projection::Thing` inside the `cfg_if!` body and emits an enforced violation (exit 1), rather than silently stripping the import

#### Scenario: An inline mod declaration inside a cfg_if macro body is reachable

- **WHEN** a governed file contains a `cfg_if!` macro invocation declaring `mod child;` and `child.rs` exists containing a forbidden import
- **THEN** the system reaches `child.rs` through the `cfg_if!` declaration and reports the forbidden import violation

### Requirement: Ancestor glob import fail-closed hazard detection

The system SHALL detect when an observed glob import's base path (`crate::a`) is an ancestor of a forbidden target path (`crate::a::b`) under a `must_not_import` module boundary. When an observed glob import base path is equal to or an ancestor of the forbidden target path (`path_within(forbidden_target, glob_base)` is true and `is_glob` is true), the system SHALL treat the wildcard import as a Glob Hazard violation and emit an enforced violation (exit 1), preventing bypass of module boundaries via wildcard ancestor imports. Plain non-glob ancestor module imports (`use crate::a;`) SHALL NOT be treated as a Glob Hazard violation and SHALL remain clean.

#### Scenario: An ancestor glob import of a forbidden module is a violation

- **WHEN** a file in a governed module declares `use crate::a::*;` and a boundary governs the module forbidding `crate::a::b`
- **THEN** the system emits an enforced Glob Hazard violation (exit 1), because `crate::a` is an ancestor of the forbidden path `crate::a::b`

#### Scenario: A glob import of an unrelated module is not a glob hazard

- **WHEN** a file in a governed module declares `use crate::c::*;` and a boundary governs the module forbidding `crate::a::b`
- **THEN** the system reports no violation for that glob import, because `crate::c` is not an ancestor of `crate::a::b`

#### Scenario: A plain non-glob ancestor import of a forbidden module is clean

- **WHEN** a file in a governed module declares `use crate::a;` (non-glob) and a boundary governs the module forbidding `crate::a::b`
- **THEN** the system reports no violation for that import, because `use crate::a;` does not bring `crate::a::b` into scope
