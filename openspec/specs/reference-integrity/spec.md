# reference-integrity Specification

## Purpose

Keep tracked in-repository path references and Tianheng's required governance surface honest under a hermetic
policy, so a checkout's verdict does not depend on ambient process state.

## Requirements

### Requirement: The real governance-document policy SHALL be hermetic

The reference-integrity gate SHALL carry Tianheng's required governance-document set literally. Ambient
environment variables SHALL NOT replace or narrow that set, so the same checkout receives the same required-surface
judgment regardless of its parent process.

#### Scenario: Ambient state names a smaller set

- **WHEN** a required governance document is absent and the process environment names a smaller document set
- **THEN** the gate still exits 2 naming the absent required document

### Requirement: Fixture policy narrowing SHALL be explicit and confined

The gate SHALL accept an explicit fixture-only governance-document set when judging a repository other than
Tianheng's own physical workspace. The set SHALL be non-empty. The option SHALL be refused on the real workspace,
and unknown or incomplete argument shapes SHALL exit 2 cannot-judge.

#### Scenario: The zero-corpus fixture narrows its prerequisite set

- **WHEN** the failure matrix explicitly supplies a non-empty fixture set for a throwaway repository
- **THEN** the gate uses it, allowing the later zero-inspected-files refusal to be observed

#### Scenario: Fixture policy targets the real workspace

- **WHEN** fixture-only policy narrowing is requested for Tianheng's own physical workspace
- **THEN** the gate exits 2 rather than weakening the required set

#### Scenario: Fixture policy is empty, surplus, or an argument is unknown

- **WHEN** the fixture option has no non-empty value or has surplus values, or an unknown argument is supplied
- **THEN** the gate exits 2 naming the invalid invocation
