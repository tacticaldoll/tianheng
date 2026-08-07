## MODIFIED Requirements

### Requirement: Tracked checkout content is the reference evidence

The reference-integrity gate SHALL judge repository paths against Git-tracked content and tracked
ancestor directories, never untracked filesystem state. Its workspace-member classification SHALL be
derived from tracked `crates/<name>/Cargo.toml` paths, so an untracked crate manifest cannot make an
illustrative crate reference enforceable. It SHALL inspect tracked Markdown and Rust files outside active
`openspec/changes/` plans. Before judging references it SHALL require the repository's governance-document
surface, at least one tracked workspace member under `crates/`, and at least one inspected source; absence
of any prerequisite SHALL be cannot-judge rather than clean.

#### Scenario: A complete tracked checkout is inspectable

- **WHEN** the repository contains the required governance documents, a tracked workspace member, and tracked Markdown or Rust source
- **THEN** the gate builds its tracked-path evidence and evaluates the corpus without consulting untracked files

#### Scenario: Required evidence is absent

- **WHEN** a required governance document, every tracked workspace member, or every inspectable Markdown and Rust source is absent
- **THEN** the gate exits 2 and names the missing prerequisite instead of reporting clean

#### Scenario: An untracked manifest cannot create a workspace member

- **WHEN** tracked prose names a missing path under an illustrative crate and only an untracked `crates/<name>/Cargo.toml` gives that crate member shape
- **THEN** the gate leaves the reference outside its existence judgment and retains the verdict produced from tracked evidence alone

#### Scenario: An active OpenSpec plan names future paths

- **WHEN** a tracked file under `openspec/changes/` references a path the plan intends to create
- **THEN** that transient plan is excluded from the inspected corpus and does not produce a stale-reference verdict
