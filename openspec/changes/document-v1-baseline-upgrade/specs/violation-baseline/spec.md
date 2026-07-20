## ADDED Requirements

### Requirement: Legacy baseline upgrade is a documented bounded operation

Adopter-facing baseline documentation SHALL identify the existing `--write-baseline` action as the
explicit opt-in upgrade from a readable version-1 text baseline to a version-2 structured snapshot.
It SHALL explain that version-1 suppression depends on exact finding wording, that metadata is
preserved only for exact current matches, and that stale legacy entries drop because rewriting is a
fresh observation snapshot. The documentation SHALL NOT imply automatic migration, a dedicated
migration command, a deprecation deadline, or a perpetual read warning.

#### Scenario: Adopter prepares for presentation changes

- **WHEN** an adopter with a version-1 baseline expects human finding wording to change and needs existing suppressions or metadata preserved
- **THEN** the documentation directs them to run the existing `--write-baseline` operation before the wording change

#### Scenario: Upgrade consequences are explicit

- **WHEN** an adopter reviews the version-1 upgrade guidance
- **THEN** it states that exact live matches retain metadata and stale entries are omitted from the version-2 snapshot

#### Scenario: Continued version-1 support is not a deprecation

- **WHEN** an adopter chooses not to rewrite a version-1 baseline
- **THEN** the documentation still describes it as readable and exact-text matched without announcing a time-based removal or new warning
