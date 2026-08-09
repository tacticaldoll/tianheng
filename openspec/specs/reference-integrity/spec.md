# reference-integrity Specification

## Purpose

Keep tracked in-repository path references and Tianheng's required governance surface honest under a hermetic
policy, so a checkout's verdict does not depend on ambient process state.

## Subject

- `crates/kanhe/tests/reference_integrity.rs`

## Requirements

### Requirement: The real governance-document policy SHALL be hermetic

The reference-integrity gate SHALL carry Tianheng's required governance-document set literally. Ambient
environment variables SHALL NOT replace or narrow that set, so the same checkout receives the same required-surface
judgment regardless of its parent process.

#### Scenario: Ambient state names a smaller set

- **WHEN** a required governance document is absent and the process environment names a smaller document set
- **THEN** the reaction still fails, naming the absent required document

### Requirement: Fixture policy narrowing SHALL be explicit and confined

The gate SHALL accept an explicit fixture-only governance-document set when judging a repository other than
Tianheng's own physical workspace. The set SHALL be non-empty. The option SHALL be refused on the real workspace,
and an unreadable or incomplete input SHALL fail, naming what could not be read.

#### Scenario: The zero-corpus fixture narrows its prerequisite set

- **WHEN** the failure matrix explicitly supplies a non-empty fixture set for a throwaway repository
- **THEN** the gate uses it, allowing the later zero-inspected-files refusal to be observed

#### Scenario: Fixture policy targets the real workspace

- **WHEN** fixture-only policy narrowing is requested for Tianheng's own physical workspace
- **THEN** the reaction fails rather than weakening the required set

#### Scenario: Fixture policy is empty, surplus, or an argument is unknown

- **WHEN** the fixture option has no non-empty value or has surplus values, or an unknown argument is supplied
- **THEN** the reaction fails, naming the invalid invocation

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
- **THEN** the reaction fails, naming the missing prerequisite instead of reporting clean

#### Scenario: An untracked manifest cannot create a workspace member

- **WHEN** tracked prose names a missing path under an illustrative crate and only an untracked `crates/<name>/Cargo.toml` gives that crate member shape
- **THEN** the gate leaves the reference outside its existence judgment and retains the verdict produced from tracked evidence alone

#### Scenario: An active OpenSpec plan names future paths

- **WHEN** a tracked file under `openspec/changes/` references a path the plan intends to create
- **THEN** that transient plan is excluded from the inspected corpus and does not produce a stale-reference verdict

### Requirement: Reference syntax determines path resolution

The gate SHALL recognize repository-relative paths under Tianheng's own top-level directories, Markdown
link targets, bare `tests/*.rs` references written inside member crates, and unambiguous bare root
filenames. Markdown links SHALL resolve lexically relative to the referring file. Bare test references
SHALL be satisfied by the matching tracked test under any workspace member. A `crates/<name>/...`
reference SHALL be judged only when `<name>` is a real workspace member; ambiguous bare filenames,
illustrative non-member crates, and glob patterns SHALL remain outside the existence judgment.

#### Scenario: A stale repository-relative prose path reacts

- **WHEN** tracked prose names a recognized repository-relative path that is not tracked
- **THEN** the gate exits 1 and names the stale reference and referring file

#### Scenario: A stale Markdown link reacts

- **WHEN** a Markdown link resolves lexically from its referring document to a path that is not tracked
- **THEN** the gate exits 1 and names the stale link target

#### Scenario: A bare member-test reference is absent everywhere

- **WHEN** Rust source in a workspace member names `tests/*.rs` and no workspace member tracks that test path
- **THEN** the gate exits 1 and reports that the reference is tracked under no workspace member

### Requirement: Deliberate absence does not become a stale-reference finding

The gate SHALL skip a recognized target when Git reports that target ignored, because prose may
deliberately describe an absent generated or local artifact. It SHALL ask Git with directory semantics so
the answer does not depend on whether an ignored directory happens to exist in the checkout.

#### Scenario: Prose names an ignored path

- **WHEN** a recognized untracked reference is covered by the repository's ignore rules
- **THEN** the gate emits no stale-reference finding for that path

### Requirement: Observation failures fail loudly rather than reading as clean

The reaction SHALL **fail, naming the observation it could not make**, when it cannot build the tracked-path
index, enumerate the inspected corpus, read an inspected source, or read the deletion history. A failed read
is not an empty result, and reporting one as the other is the vacuity direction the Core Contract forbids.

The three-way exit contract does not survive as an exit code — a Rust test passes or fails — so every
cannot-judge condition fails loudly and says which read it was. That is the safe direction: the alternative is
a reaction reporting clean over content it never read.

#### Scenario: The tracked-path index cannot be built

- **WHEN** the Git enumeration that owns every tracked-path answer fails
- **THEN** the reaction fails, naming the tracked-path index failure

#### Scenario: Extracted references cannot be normalized

- **WHEN** the normalization pipeline fails for references extracted from an inspected file
- **THEN** the reaction fails, naming that file instead of silently examining an empty stream

#### Scenario: An unhandled command fails

- **WHEN** an unwrapped command fails while the gate is running
- **THEN** the reaction fails, naming the read it could not make rather than reporting a clean corpus

### Requirement: The gate is a read-only 0/1/2 reaction

The reference-integrity gate SHALL be read-only. A clean judgment SHALL exit 0, print its positive summary
on standard output, and print nothing on standard error. One or more stale references SHALL be aggregated
and exit 1 with remediation. An invalid invocation, missing prerequisite, or observation failure SHALL
fail loudly. No verdict SHALL alter tracked, untracked, or commit state in the repository being judged.

#### Scenario: A clean repository passes silently on standard error

- **WHEN** every judged reference resolves or falls within a declared exclusion
- **THEN** the gate exits 0, prints the inspected-file summary on standard output, and leaves standard error empty

#### Scenario: Stale references are an enforced violation

- **WHEN** one or more judged references do not resolve to tracked or deliberately ignored paths
- **THEN** the gate reports every offense, prints remediation, and exits 1

#### Scenario: Judging a repository does not mutate it

- **WHEN** the gate evaluates a fixture it has not previously inspected
- **THEN** the fixture's tracked tree, untracked state, and HEAD remain unchanged
