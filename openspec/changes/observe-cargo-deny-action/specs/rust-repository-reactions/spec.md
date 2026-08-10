## ADDED Requirements

### Requirement: Definition-of-Done coherence SHALL compare effective CI commands

Every command in AGENTS.md's Definition of Done SHALL have an effective counterpart in CI. Commands expressed by `run:` SHALL be compared directly after the existing normalization. The repository's `EmbarkStudios/cargo-deny-action` step SHALL contribute `cargo deny <command>` from its declared `with.command` value. A DoD command SHALL NOT be exempted merely because CI normally expresses it through an action.

The action projection is intentionally limited to the cargo-deny action whose command semantics this repository uses; the reaction SHALL NOT claim to interpret arbitrary GitHub Actions.

#### Scenario: Cargo deny is supplied by its action

- **WHEN** the DoD contains `cargo deny check` and CI contains an `EmbarkStudios/cargo-deny-action` step whose `with.command` is `check`
- **THEN** the coherence reaction recognizes the effective command and does not report it missing

#### Scenario: The supply-chain step is absent or misconfigured

- **WHEN** the DoD contains `cargo deny check` and CI omits the cargo-deny action or gives it a different or absent command
- **THEN** the coherence reaction fails and names `cargo deny check` as missing from CI
