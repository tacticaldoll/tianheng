## ADDED Requirements

### Requirement: Root-aware audit excludes unreachable source files

When `audit_probe_coverage` receives a Rust target root file, it SHALL count probes only from that
file and source files reachable through its lexical module declarations. An undeclared `.rs` file,
or a conventional file shadowed by an inline-only module, SHALL NOT count as coverage. Every
reachable selected source file SHALL be read fail-loud; an unreadable file SHALL produce a
constitution error. The walker SHALL remain louke-local, std-only, and audit-feature-only. Directory
inputs SHALL remain accepted as the legacy recursive corpus for source compatibility.

#### Scenario: Orphan probe cannot cover a seam

- **WHEN** a target root declares no module for `orphan.rs` and that orphan file contains the only probe for a declared seam
- **THEN** the audit reports the seam unprobed because the compiler-unreachable file is absent from the root-aware corpus

#### Scenario: Reachable external module covers a seam

- **WHEN** a target root declares `mod adapter;` and the resolved `adapter.rs` or `adapter/mod.rs` contains the seam's probe
- **THEN** the audit counts the probe as coverage

#### Scenario: Inline module shadow does not activate a sibling file

- **WHEN** a root declares only `mod adapter { ... }` and a sibling `adapter.rs` contains a probe
- **THEN** the sibling file does not count because the inline body, not the conventional file, is the compiled module

#### Scenario: Custom target root remains auditable

- **WHEN** Cargo reports a custom library root filename and its reachable modules contain probes
- **THEN** the audit starts from that exact file rather than guessing `src/lib.rs`

#### Scenario: Legacy directory callers remain compatible

- **WHEN** a direct caller passes a source directory instead of a target root file
- **THEN** the audit retains the recursive directory scan and the caller requires no source change
