# violation-baseline Specification

## Purpose

Let a dirty project adopt a boundary without first fixing every pre-existing violation. A baseline
is a generated snapshot of accepted violations, not policy; the gate suppresses recorded violations,
fails only on new drift, and reports stale entries so accepted debt can ratchet down.

## Requirements

### Requirement: Baseline records accepted violations

A baseline SHALL record a set of accepted violations. A version-2 entry SHALL be identified by
`(target, rule, finding_key)` and SHALL also carry the human-readable `finding` string for diagnosis.
The structured finding key SHALL contain its namespace, fact code, and canonically ordered named
string fields. The baseline SHALL be a generated artifact (a projection), never the constitution:
boundaries, rules, and severity remain declared in Rust. Identity SHALL exclude the human finding
text, boundary reason, severity, file, anchor, and polarity, so presentation or metadata changes
SHALL NOT make an existing version-2 violation count as new. The baseline SHALL serialize as JSON
carrying a `version` and a list of `violations` sorted by identity, so the file is stable and
diffable. Every baseline newly generated from current violations SHALL declare version `2`.
Re-serializing a parsed version-1 snapshot without current observations SHALL preserve version 1
rather than fabricate structured keys.

A baseline entry SHALL be able to carry two optional governance-tracking fields — `owner` (who owns
this accepted debt) and `tracker` (the external issue tracking its fix). These fields SHALL be
metadata only and SHALL NOT enter identity. The baseline SHALL NOT carry a per-entry `anchor`. On
disk, `owner` and `tracker` SHALL be emitted only when set and parsed as absent when omitted.
Entries SHALL remain sorted and de-duplicated by structured identity; when two entries share an
identity, the first in identity order SHALL be kept.

The reader SHALL also accept version-1 entries containing `(target, rule, finding)`. Such an entry's
public `ViolationId` SHALL carry no structured key and SHALL match only a current violation with the
exact same old text triple. Version-aware legacy matching SHALL be confined to baseline operations
and SHALL NOT become fallback equality for structured `ViolationId`, because mixed text/key equality
would not form a valid equivalence relation. A malformed document or any version other than 1 or 2
SHALL be an error.

#### Scenario: A version-2 baseline round-trips through JSON

- **WHEN** a baseline of known violations is written to JSON and read back
- **THEN** the parsed baseline contains the same `(target, rule, finding_key)` identities, human findings, and optional metadata

#### Scenario: A malformed or unknown-version baseline is an error

- **WHEN** a baseline file is malformed or declares a version other than 1 or 2
- **THEN** the system reports an error rather than silently treating the baseline as empty

#### Scenario: Owner and tracker round-trip when set

- **WHEN** a version-2 baseline entry carries an `owner` and a `tracker`, is written to JSON, and read back
- **THEN** the parsed entry preserves the same metadata and structured identity

#### Scenario: Metadata does not affect matching

- **WHEN** a violation's identity is in the baseline, whether or not that entry carries owner/tracker
- **THEN** the violation is suppressed exactly as an identity match, with metadata changing neither classification nor count

#### Scenario: A duplicate structured identity keeps the first entry

- **WHEN** a version-2 baseline carries two entries with the same `(target, rule, finding_key)` but different metadata or finding text
- **THEN** it de-duplicates to one entry by structured identity and keeps the first entry's presentation and metadata

#### Scenario: A version-1 baseline remains readable

- **WHEN** a version-1 baseline containing only `target`, `rule`, and `finding` is read
- **THEN** it parses successfully as legacy entries with any owner/tracker metadata preserved

#### Scenario: Re-serializing a legacy snapshot does not invent keys

- **WHEN** a parsed version-1 baseline is serialized without rebuilding it from current violations
- **THEN** it remains version 1 with the same legacy text identities rather than emitting fabricated structured keys

### Requirement: Optional baseline metadata has a strict input type

For every version-1 and version-2 baseline entry, the parser SHALL accept `owner` and `tracker` only
when each field is absent, JSON null, or a JSON string. Absent and null fields SHALL represent no
annotation, while a string SHALL be preserved as metadata. Any other JSON type SHALL make the
baseline malformed rather than silently removing the annotation. Generated baselines SHALL continue
to omit unset metadata fields.

#### Scenario: Omitted and null metadata are absent

- **WHEN** an entry omits `owner` or `tracker`, or supplies either field as JSON null
- **THEN** the baseline parses with that annotation absent and serialization omits the unset field

#### Scenario: String metadata is preserved

- **WHEN** an entry supplies `owner` or `tracker` as a JSON string
- **THEN** the baseline preserves that exact string across parse and serialization

#### Scenario: Wrong-typed metadata invalidates the baseline

- **WHEN** a present `owner` or `tracker` is a number, boolean, array, or object
- **THEN** baseline parsing fails with an error identifying the malformed metadata field

#### Scenario: Gating fails loud on malformed metadata

- **WHEN** `tianheng check --baseline` reads a baseline containing wrong-typed optional metadata
- **THEN** the command reports an invalid baseline and exits as a scan error rather than gating with the annotation silently absent

#### Scenario: Explicit rewrite warns before metadata loss

- **WHEN** `tianheng check --write-baseline` reads a prior baseline containing wrong-typed optional metadata
- **THEN** it warns that the prior baseline could not be parsed and that metadata will not be carried forward before writing a fresh snapshot

### Requirement: Gate suppresses baselined violations and fails only on new ones

In gate mode the system SHALL classify each current violation against the baseline. A current
violation matching a version-2 entry by `(target, rule, finding_key)`, or a version-1 legacy entry by
the exact `(target, rule, finding)` text triple, SHALL be suppressed and SHALL NOT cause failure. A
violation not matched by the active baseline SHALL react according to its severity. The reaction
SHALL fail (exit 1) only when a non-baselined enforce-severity violation exists; warn-severity and
baselined violations SHALL NOT fail.

#### Scenario: A structured pre-existing violation does not fail

- **WHEN** every current enforce-severity violation has a matching version-2 structured identity
- **THEN** the gate does not fail (exit 0), even if a matching violation's human finding wording changed

#### Scenario: A version-1 text match does not fail

- **WHEN** every current enforce-severity violation exactly matches a legacy baseline entry's target, rule, and finding text
- **THEN** the gate does not fail (exit 0)

#### Scenario: A version-1 wording mismatch is new

- **WHEN** a current violation has the same stable fact as a legacy entry but different finding text
- **THEN** it is reported as new because the version-1 artifact contains no structured key proving identity

#### Scenario: A new enforce violation fails

- **WHEN** an enforce-severity violation matches neither a version-2 structured entry nor a version-1 legacy entry
- **THEN** the gate fails (exit 1) and reports that violation as new

### Requirement: Stale baseline entries are reported but do not fail

The system SHALL report any baseline entry that matches no current violation as stale, using
structured matching for version 2 and exact legacy text matching for version 1, so the baseline can
be ratcheted down as violations are fixed. A stale entry SHALL retain its human finding text for
diagnosis and SHALL NOT cause the reaction to fail.

#### Scenario: A fixed structured violation leaves a stale baseline entry

- **WHEN** a version-2 baseline contains an entry whose structured identity matches no current violation
- **THEN** the system reports that entry, including its human finding and structured key, as stale and does not fail on account of it

#### Scenario: An unmatched legacy entry is stale

- **WHEN** a version-1 baseline entry has no exact current target, rule, and finding text match
- **THEN** the system reports that legacy entry as stale and does not fail on account of it

### Requirement: Writing a baseline records the current violations

The system SHALL provide a write action that records the current violations as a version-2 baseline
and SHALL exit 0, since recording is not judging. The write action SHALL refuse to write when the
constitution cannot be evaluated, exiting 2.

The write action SHALL preserve entry metadata across regeneration. It SHALL read an existing
baseline at the target path when present and carry `owner`/`tracker` forward for each current
violation matched by that baseline's version-aware rule. A new violation SHALL receive no metadata;
a stale entry SHALL be dropped with its metadata. Rewriting a readable version-1 baseline SHALL
therefore upgrade it to version 2 while preserving metadata for exact legacy matches. An existing
file that cannot be read or parsed SHALL NOT block a fresh write, but the action SHALL warn on
standard error that metadata was not carried forward. A missing file SHALL NOT warn.

#### Scenario: Write records current violations as version 2 and exits 0

- **WHEN** the write action runs against a workspace with violations
- **THEN** it writes their human findings and structured keys in a version-2 baseline and exits 0

#### Scenario: Write refuses on a constitution error

- **WHEN** the write action runs but the constitution cannot be evaluated
- **THEN** it reports the error and exits 2 without writing a baseline

#### Scenario: Rewriting version 2 preserves metadata for surviving violations

- **WHEN** a version-2 entry has owner/tracker metadata and its structured identity is still present
- **THEN** the rewritten version-2 baseline carries that metadata forward even if its finding wording changed

#### Scenario: Rewriting version 1 upgrades and preserves matching metadata

- **WHEN** a version-1 entry has owner/tracker metadata and a current violation exactly matches its target, rule, and finding text
- **THEN** the rewritten file declares version 2, records the current structured key, and carries the metadata forward

#### Scenario: A resolved violation drops its entry and metadata

- **WHEN** the write action is re-run after a previously baselined violation no longer occurs
- **THEN** the rewritten baseline omits that entry and its metadata

#### Scenario: An unreadable existing baseline warns rather than silently discarding metadata

- **WHEN** an existing baseline cannot be read or parsed during a write action
- **THEN** the action warns that metadata is not carried forward, writes a fresh version-2 baseline, and exits 0

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
