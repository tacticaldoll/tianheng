# violation-baseline Specification

## Purpose

Let a dirty project adopt a boundary without first fixing every pre-existing
violation. A baseline is a generated snapshot of accepted violations (a projection,
not policy — see the baseline decision in `PROJECT.md`); the gate fails only on violations new since the baseline,
suppresses recorded ones, and reports stale entries so the baseline can only
ratchet down. This is the `gate` rung of the adoption ramp above `warn`.

## Requirements
### Requirement: Baseline records accepted violations

A baseline SHALL record a set of accepted violations, each identified by the triple `(target, rule, finding)`. The baseline SHALL be a generated artifact (a projection), never the constitution: boundaries, rules, and severity remain declared in Rust. The identity SHALL exclude the boundary's reason and severity, so editing a reason or moving a boundary between warn and enforce SHALL NOT make an existing violation count as new. The baseline SHALL serialize as JSON carrying a `version` and a list of `violations` sorted by their identity, so the file is stable and diffable.

A baseline entry SHALL be able to carry two optional governance-tracking fields — `owner` (who owns this accepted debt) and `tracker` (the external issue tracking its fix) — so an accepted violation points at a tracker instead of accreting a silent, never-shrinking exemption table. These fields describe *how the accepted violation is tracked after acceptance*, not the basis of the law; they SHALL be **metadata only** and SHALL NOT enter the entry's identity `(target, rule, finding)`, so adding, changing, or omitting them SHALL NOT make an existing violation count as new nor change the number of violations reported. The baseline SHALL NOT carry a per-entry `anchor`: the governance anchor already rides the live boundary→violation, so a baseline copy would only duplicate and drift from that source of truth. On disk, `owner`/`tracker` SHALL be emitted **only when set**, so an entry without them is byte-identical to the pre-metadata form and the format `version` SHALL remain `1`; parsing SHALL read them when present and treat them as absent (`None`) otherwise, so a baseline written without them parses unchanged and a reader that predates the fields still parses one that carries them (unknown fields ignored). Entries SHALL remain sorted and de-duplicated **by identity** — the `owner`/`tracker` metadata is NOT part of the sort or de-duplication key — so the file stays stable and diffable; when two entries share an identity (e.g. a hand-edited duplicate), the first in identity order SHALL be kept and the other dropped.

#### Scenario: A baseline round-trips through JSON

- **WHEN** a baseline of known violations is written to JSON and read back
- **THEN** the parsed baseline contains the same `(target, rule, finding)` entries

#### Scenario: A malformed or unknown-version baseline is an error

- **WHEN** a baseline file is malformed or declares an unknown `version`
- **THEN** the system reports an error rather than silently treating the baseline as empty

#### Scenario: An entry without metadata is byte-identical to the pre-metadata form

- **WHEN** a baseline of violations that carry no owner/tracker is serialized to JSON
- **THEN** each entry carries only `target`, `rule`, and `finding` (no `owner`/`tracker` keys), the document declares `version` 1, and it is byte-identical to a baseline written before the metadata fields existed

#### Scenario: Owner and tracker round-trip when set

- **WHEN** a baseline entry carries an `owner` and a `tracker`, is written to JSON, and read back
- **THEN** the parsed entry preserves the same `owner`, `tracker`, and `(target, rule, finding)` identity

#### Scenario: A baseline without the fields still parses

- **WHEN** a baseline written before the metadata fields (entries with only `target`/`rule`/`finding`) is read
- **THEN** it parses successfully with each entry's `owner` and `tracker` absent, and matching is unchanged

#### Scenario: Metadata does not affect matching

- **WHEN** a violation's identity is in the baseline, whether or not that entry carries owner/tracker
- **THEN** the violation is suppressed exactly as an identity match, the metadata never changing the classification or the count

#### Scenario: A duplicate identity keeps the first entry

- **WHEN** a baseline carries two entries with the same `(target, rule, finding)` identity but different metadata (a hand-edit)
- **THEN** it de-duplicates by identity to a single entry, keeping the first in identity order

### Requirement: Gate suppresses baselined violations and fails only on new ones

In gate mode the system SHALL classify each current violation against the baseline: a violation whose identity is in the baseline SHALL be suppressed and SHALL NOT cause failure, while a violation not in the baseline SHALL react according to its severity. The reaction SHALL fail (exit 1) only when a non-baselined enforce-severity violation exists; warn-severity and baselined violations SHALL NOT fail.

#### Scenario: A pre-existing violation in the baseline does not fail

- **WHEN** every current enforce-severity violation is present in the baseline
- **THEN** the gate does not fail (exit 0)

#### Scenario: A new enforce violation fails

- **WHEN** an enforce-severity violation is present that is not in the baseline
- **THEN** the gate fails (exit 1) and reports that violation as new

### Requirement: Stale baseline entries are reported but do not fail

The system SHALL report any baseline entry that matches no current violation as stale, so the baseline can be ratcheted down as violations are fixed. A stale entry SHALL NOT cause the reaction to fail.

#### Scenario: A fixed violation leaves a stale baseline entry

- **WHEN** the baseline contains an entry that matches no current violation
- **THEN** the system reports that entry as stale and does not fail on account of it

### Requirement: Writing a baseline records the current violations

The system SHALL provide a write action that records the current violations as a baseline and SHALL exit 0, since recording is not judging. The write action SHALL refuse to write when the constitution cannot be evaluated (a constitution or scan error), exiting 2, because a baseline cannot be pinned from an unevaluable constitution.

The write action SHALL preserve entry metadata across regeneration: it SHALL read the existing baseline at the target path when one is present, and for each current violation whose identity matches an existing entry SHALL carry that entry's `owner`/`tracker` forward; a violation with no matching existing entry SHALL be recorded with no metadata; a previously-recorded entry whose violation is no longer present SHALL be dropped (its metadata with it). Re-running the write action therefore SHALL NOT silently discard hand-added governance records for violations that still exist. An existing file that cannot be read or parsed SHALL NOT block writing a fresh baseline (the write falls back to recording the current violations with no carried metadata), but the write action SHALL **warn** (to standard error) that the existing baseline could not be read and its metadata is therefore not carried forward — so the metadata loss is visible, never silent. A missing file is the normal first write and SHALL NOT warn.

#### Scenario: Write records current violations and exits 0

- **WHEN** the write action runs against a workspace with violations
- **THEN** the system writes those violations as a baseline and exits 0

#### Scenario: Write refuses on a constitution error

- **WHEN** the write action runs but the constitution cannot be evaluated
- **THEN** the system reports the error and exits 2 without writing a baseline

#### Scenario: Re-writing preserves metadata for surviving violations

- **WHEN** a baseline entry has been annotated with an `owner`/`tracker`, and the write action is re-run against a workspace where that violation still exists
- **THEN** the rewritten baseline still carries that entry's `owner` and `tracker`

#### Scenario: A resolved violation drops its entry and metadata

- **WHEN** the write action is re-run against a workspace where a previously-baselined (and annotated) violation no longer occurs
- **THEN** the rewritten baseline omits that entry entirely, its metadata gone with it

#### Scenario: An unreadable existing baseline warns rather than silently discarding metadata

- **WHEN** the write action runs and an existing baseline file is present but cannot be read or parsed
- **THEN** it warns that the existing baseline could not be read and its metadata is not carried forward, then writes a fresh baseline of the current violations and exits 0 (never blocking the write, never silently discarding the metadata)

