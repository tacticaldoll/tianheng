# composed-library-check Specification

## Purpose

Define a presentation-free library entrypoint that evaluates one unified Constitution through the
same three-dimension reaction semantics as the CLI runner.

## Requirements

### Requirement: A unified Constitution has an inspectable library check

The `tianheng` crate SHALL provide
`check_constitution(&Constitution, &Path) -> Outcome`. It SHALL evaluate the static boundaries, the
full semantic boundary bundle, and the runtime probe-coverage CI face declared by that Constitution
against the explicit manifest workspace. It SHALL return the raw combined Outcome without parsing
CLI arguments, discovering a manifest from the current directory, printing output, applying a
baseline, writing a baseline, or emitting coverage advisories.

#### Scenario: Multiple dimensions return one report

- **WHEN** a workspace violates both a static boundary and a semantic boundary in one Constitution
- **THEN** `check_constitution` returns one `Outcome::Violations` report containing findings from both dimensions

#### Scenario: A clean composed Constitution is inspectable

- **WHEN** every declared boundary is satisfied and runtime probes are consistent
- **THEN** `check_constitution` returns `Outcome::Clean`

#### Scenario: The manifest is explicit

- **WHEN** a caller invokes `check_constitution`
- **THEN** the observed workspace is selected only by the supplied manifest path, without current-directory discovery

### Requirement: Library and CLI share composition semantics

`check_constitution` and CLI `run` SHALL use one composition implementation for dimension order,
violation merging, constitution-error precedence, and runtime probe audit. Static evaluation SHALL
run first; semantic evaluation SHALL run only when static evaluation did not error and semantic
boundaries exist; runtime audit SHALL run only when prior evaluation did not error and SHALL run even
when the declared runtime-boundary set is empty so an orphan probe reacts.

#### Scenario: A constitution error supersedes violations

- **WHEN** an earlier dimension returns a constitution error alongside a workspace that would violate another dimension
- **THEN** the combined library Outcome is that constitution error and later dimensions are not evaluated

#### Scenario: An orphan runtime probe still reacts

- **WHEN** source contains an `assert_boundary!` probe but the Constitution declares no matching runtime boundary
- **THEN** `check_constitution` includes the runtime undeclared-seam violation rather than treating an empty runtime declaration as a no-op

#### Scenario: CLI verdict remains derived from the shared Outcome

- **WHEN** CLI `run` checks the same Constitution and explicit manifest without a baseline
- **THEN** its clean, violation, or constitution-error exit class mirrors the Outcome returned by the shared evaluator

### Requirement: Library checking remains separate from gate presentation

The library check SHALL NOT accept format, baseline, baseline-write, or coverage-warning options.
Callers that need process exit codes, text/JSON/SARIF projection, baseline gate modes, nearest-
manifest discovery, or uncovered-crate advisories SHALL continue to use `run`.

#### Scenario: A caller needs structured findings without process output

- **WHEN** a test calls `check_constitution`
- **THEN** it can inspect the returned Outcome without capturing stdout/stderr or decoding an ExitCode

#### Scenario: A caller needs baseline gate mode

- **WHEN** a caller needs to load or write a baseline
- **THEN** it uses `run` (or existing lower-level baseline APIs) rather than options on `check_constitution`

### Requirement: Composed runtime audit preserves Cargo target roots

The composed check SHALL obtain every workspace member's exact library-or-binary target root from
the shared workspace-data substrate and pass those files to the runtime probe audit. It SHALL NOT
reduce roots to directories or guess conventional filenames. Root-resolution or reachable-source
errors SHALL retain the composed check's constitution-error precedence.

#### Scenario: Composed check rejects orphan-only coverage

- **WHEN** a workspace member's declared seam is probed only in an orphan `.rs` file outside the module graph
- **THEN** the composed check reports the seam unprobed and exits according to its declared severity

#### Scenario: Custom Cargo root is passed exactly

- **WHEN** a workspace member declares a custom library or binary source path
- **THEN** the composed runtime audit begins at Cargo's reported source file and observes its reachable probes

### Requirement: Tianheng self-governance uses the composed library evaluator

The repository's primary self-governance gate SHALL evaluate the live self-Constitution through
`check_constitution` against the workspace manifest. It SHALL therefore exercise the same ordered
static, semantic, and runtime-audit composition used by adopters, including the runtime audit when
the Constitution declares no runtime boundaries. Declaration-integrity assertions MAY remain
separate because their observation source is the Constitution itself rather than workspace code.

#### Scenario: The live self-law is clean through the adopter entrypoint

- **WHEN** Tianheng's self-governance test evaluates its declared Constitution
- **THEN** it calls the composed library evaluator and requires one clean Outcome for the workspace

#### Scenario: An undeclared reachable runtime probe cannot bypass self-governance

- **WHEN** a reachable production source under the workspace target roots contains a runtime probe absent from the self-Constitution
- **THEN** the self-governance gate receives the runtime audit reaction instead of passing because its runtime declaration set is empty
