## MODIFIED Requirements

### Requirement: Tracked checkout content is the reference evidence

The reference-integrity gate SHALL judge repository paths against Git-tracked content and tracked ancestor directories, never untracked filesystem state. Its workspace-member classification SHALL be derived from tracked `crates/<name>/Cargo.toml` paths, so an untracked crate manifest cannot make an illustrative crate reference enforceable. Outside active `openspec/changes/` plans, it SHALL inspect tracked Markdown document text and every Rust, TOML, or `.gitignore` line whose first non-whitespace token is that format's line-comment marker, including Rust rustdoc forms. A Rust test source SHALL NOT be excluded wholesale; its admitted comment lines are judged through the same region rule as other Rust. Before judging references it SHALL require the repository's governance-document surface, at least one tracked workspace member under `crates/`, and at least one inspected source; absence of any prerequisite SHALL fail loudly rather than read as clean.

#### Scenario: A complete tracked checkout is inspectable

- **WHEN** the repository contains the required governance documents, a tracked workspace member, and a tracked Markdown document or an admitted Rust, TOML, or `.gitignore` line-comment source
- **THEN** the gate builds its tracked-path evidence and evaluates the corpus without consulting untracked files

#### Scenario: Required evidence is absent

- **WHEN** a required governance document, every tracked workspace member, or every inspectable source is absent
- **THEN** the reaction fails, naming the missing prerequisite instead of reporting clean

#### Scenario: An untracked manifest cannot create a workspace member

- **WHEN** tracked prose names a missing path under an illustrative crate and only an untracked `crates/<name>/Cargo.toml` gives that crate member shape
- **THEN** the gate leaves the reference outside its existence judgment and retains the verdict produced from tracked evidence alone

#### Scenario: An active OpenSpec plan names future paths

- **WHEN** a tracked file under `openspec/changes/` references a path the plan intends to create
- **THEN** that transient plan is excluded from the inspected corpus and does not produce a stale-reference verdict

#### Scenario: A test source names a deleted live path

- **WHEN** a tracked Rust test comment names a recognized path this repository deleted rather than a path the test fixture constructs
- **THEN** the reaction fails and names the stale reference instead of excluding the whole test source

### Requirement: Reference syntax determines path resolution

The gate SHALL recognize repository-relative paths under Tianheng's own top-level directories, Markdown link targets, bare `tests/*.rs` references written inside member crates, and unambiguous bare root filenames. Markdown links SHALL resolve lexically relative to the referring file. Bare test references SHALL be satisfied by the matching tracked test under any workspace member. A `crates/<name>/...` reference SHALL be judged only when `<name>` is a real workspace member; ambiguous bare filenames, illustrative non-member crates, and glob patterns SHALL remain outside the existence judgment.

#### Scenario: A stale repository-relative prose path reacts

- **WHEN** tracked prose names a recognized repository-relative path that is not tracked
- **THEN** the reaction fails and names the stale reference and referring file

#### Scenario: A stale Markdown link reacts

- **WHEN** a Markdown link resolves lexically from its referring document to a path that is not tracked
- **THEN** the reaction fails and names the stale link target

#### Scenario: A bare member-test reference is absent everywhere

- **WHEN** Rust source in a workspace member names `tests/*.rs` and no workspace member tracks that test path
- **THEN** the reaction fails and reports that the reference is tracked under no workspace member

### Requirement: Observation failures fail loudly rather than reading as clean

The gate SHALL **fail, naming the observation it could not make**, when it cannot build the tracked-path index,
enumerate the inspected corpus, read an inspected source, or read the deletion history. A failed read is not an
empty result, and reporting one as the other is the vacuity direction the Core Contract forbids.

A Rust test passes or fails, so every observation failure fails loudly and says which read it was. That is the
safe direction: the alternative is a check reporting clean over content it never read.

#### Scenario: The tracked-path index cannot be built

- **WHEN** the Git enumeration that owns every tracked-path answer fails
- **THEN** the gate fails, naming the tracked-path index failure

#### Scenario: Extracted references cannot be normalized

- **WHEN** the normalization pipeline fails for references extracted from an inspected file
- **THEN** the gate fails, naming that file instead of silently examining an empty stream

#### Scenario: An unhandled command fails

- **WHEN** an unwrapped command fails while the gate is running
- **THEN** the gate fails, naming the read it could not make rather than reporting a clean corpus

### Requirement: The gate SHALL be read-only and fail observably

The reference-integrity gate SHALL be read-only. A clean judgment SHALL pass. One or more stale references SHALL be aggregated into a failing assertion with remediation. An invalid invocation, missing prerequisite, or observation failure SHALL fail loudly and name what could not be judged. No verdict SHALL alter tracked, untracked, or commit state in the repository being judged.

#### Scenario: A clean repository passes

- **WHEN** every judged reference resolves or falls within a declared exclusion
- **THEN** the reaction passes without requiring a particular stdout or process-exit vocabulary

#### Scenario: Stale references are an enforced failure

- **WHEN** one or more judged references do not resolve to tracked or deliberately ignored paths
- **THEN** the reaction reports every offence with remediation and fails

#### Scenario: Judging a repository does not mutate it

- **WHEN** the gate evaluates a fixture it has not previously inspected
- **THEN** the fixture's tracked tree, untracked state, and HEAD remain unchanged

## RENAMED Requirements

- FROM: `### Requirement: The gate is a read-only 0/1/2 reaction`
- TO: `### Requirement: The gate SHALL be read-only and fail observably`
