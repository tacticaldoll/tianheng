# violation-baseline Specification

## Purpose

Let a dirty project adopt a boundary without first fixing every pre-existing violation. A baseline
is a generated snapshot of accepted violations, not policy; the gate suppresses recorded violations,
fails only on new drift, and reports stale entries so accepted debt can ratchet down.

## Requirements

### Requirement: Baseline records accepted violations

A baseline SHALL record a set of accepted violations using the exact semantic format
`tianheng.baseline/structured-facts`. Each entry SHALL be identified by its governed target,
semantic rule key, and structured fact identity, and SHALL retain human rule/finding presentation
for diagnosis. The baseline SHALL be a generated reaction snapshot, never policy: boundaries,
rules, severity, and reasons remain declared in Rust. Presentation, reason, severity, file, anchor,
polarity, diagnostics, `owner`, and `tracker` SHALL NOT enter identity.

The JSON document SHALL carry `format` and an identity-sorted `violations` array; it SHALL NOT carry
a numeric Tianheng baseline version. Entries SHALL support optional string `owner` and `tracker`
governance annotations, emitted only when set. Entries SHALL be sorted and de-duplicated by full
structured identity; for duplicate identities, the first occurrence in the parsed document SHALL
be kept before identity sorting.

Only the exact semantic format SHALL parse. A numeric v1/v2 document, an unmarked document, a
different semantic format, or malformed data SHALL be an error rather than a legacy identity mode.

#### Scenario: A semantic baseline round-trips through JSON

- **WHEN** a baseline of known violations is written and read
- **THEN** it retains the semantic format, structured identities, presentations, and optional annotations

#### Scenario: Presentation and annotations do not affect matching

- **WHEN** an entry's presentation, owner, or tracker changes but its target/rule/fact identity does not
- **THEN** it suppresses exactly the same violation

#### Scenario: A duplicate identity keeps the first entry

- **WHEN** two entries have the same target, rule key, and fact but different presentation or annotations
- **THEN** parsing de-duplicates them and keeps the first document occurrence's attached data before identity sorting

#### Scenario: A legacy or unknown format is an error

- **WHEN** a baseline is numeric v1/v2, unmarked, malformed, or declares another semantic format
- **THEN** parsing fails loud rather than matching through presentation or assuming a migration generation

### Requirement: Optional baseline metadata has a strict input type

For each semantic baseline entry, the parser SHALL accept `owner` and `tracker` only when absent,
JSON null, or a JSON string. Absent and null SHALL mean no annotation; a string SHALL be preserved.
Any other JSON type SHALL make the baseline malformed. Generated baselines SHALL omit unset fields.

#### Scenario: Omitted and null metadata are absent

- **WHEN** an entry omits owner/tracker or supplies JSON null
- **THEN** parsing records no annotation and serialization omits the unset field

#### Scenario: String metadata is preserved

- **WHEN** an entry supplies an owner or tracker string
- **THEN** the exact string survives parse and serialization

#### Scenario: Wrong-typed metadata invalidates the baseline

- **WHEN** owner or tracker is a number, boolean, array, or object
- **THEN** parsing fails with an error identifying the malformed annotation

### Requirement: Gate suppresses baselined violations and fails only on new ones

In gate mode the system SHALL classify current violations by exact structured identity. A matching
violation SHALL be accepted and SHALL NOT cause failure. An unmatched violation SHALL react by its
severity. Exit 1 SHALL occur only when an unmatched enforce-severity violation exists; warn and
accepted violations SHALL NOT fail. No text-matching fallback SHALL exist.

#### Scenario: A structured pre-existing violation does not fail

- **WHEN** every current enforce violation has a matching target/rule/fact identity
- **THEN** the gate exits 0 even if presentation or diagnostics changed

#### Scenario: Presentation equality cannot substitute for fact identity

- **WHEN** a current violation has the same displayed text but a different structured identity
- **THEN** it is new and reacts according to severity

#### Scenario: A new enforce violation fails

- **WHEN** an enforce violation has no exact structured baseline identity
- **THEN** the gate exits 1 and reports it as new

### Requirement: Stale baseline entries are reported but do not fail

The system SHALL report every semantic baseline entry whose structured identity matches no current
violation as stale. A stale entry SHALL retain its presentations and annotations for diagnosis and
SHALL NOT cause failure.

#### Scenario: A fixed violation leaves a stale entry

- **WHEN** a baseline identity matches no current violation
- **THEN** the system reports its target, rule key, fact identity, presentation, and annotations as stale without failing

### Requirement: Writing a baseline records the current violations

The write action SHALL record current violations in `tianheng.baseline/structured-facts` and exit 0,
because recording is not judging. It SHALL refuse to write and exit 2 when the constitution cannot
be evaluated. If the target is missing, it SHALL create the semantic baseline. If the target is a
valid supported baseline, it SHALL regenerate the snapshot and carry owner/tracker forward across
exact identity matches; new entries receive no annotations and stale entries are dropped.

If an existing target is unreadable, malformed, unmarked, numeric v1/v2, or another semantic
format, the action SHALL fail loud and SHALL NOT overwrite it. Its error SHALL tell the adopter to
preserve desired annotations, move or delete the unsupported file, and invoke the same write action
again. It SHALL NOT attempt automatic migration or reconstruct identity from presentation.

#### Scenario: Write records current violations in the semantic format

- **WHEN** the write action targets a missing path and observation succeeds
- **THEN** it writes the current structured identities with the semantic format and exits 0

#### Scenario: Rewriting a supported baseline preserves matching metadata

- **WHEN** a supported entry remains present by exact identity
- **THEN** regeneration carries its owner/tracker forward even if presentation changed

#### Scenario: A resolved violation drops its metadata

- **WHEN** a previously accepted violation no longer occurs
- **THEN** regeneration omits the stale entry and its annotations

#### Scenario: Write refuses to overwrite a legacy file

- **WHEN** the target exists as numeric v1/v2, unmarked, unknown-format, malformed, or unreadable data
- **THEN** the action exits 2 without modifying the file and prints actionable regeneration guidance

#### Scenario: Write refuses on a constitution error

- **WHEN** the constitution cannot be evaluated
- **THEN** the action exits 2 without writing a baseline
