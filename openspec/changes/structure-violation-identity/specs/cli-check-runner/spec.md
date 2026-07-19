## ADDED Requirements

### Requirement: Machine-readable reports expose structured finding identity

The runner's JSON report SHALL add a `finding_key` to the existing human-readable violation and
stale-baseline projections without replacing or weakening any field, format, or exit-code contract
of the existing machine-readable report requirement. For a current violation and a version-2 stale
entry, `finding_key` SHALL contain its `namespace`, fact `code`, and named string `fields`.

Each stale version-2 baseline entry SHALL carry `target`, `rule`, human `finding`, and the same
`finding_key` shape. A stale version-1 entry SHALL retain its target, rule, and human finding and
SHALL represent the structured key as `null`, faithfully reflecting that the legacy artifact never
recorded one. The structured key SHALL affect baseline identity but SHALL NOT change outcome, exit
code, severity, file, anchor, polarity, or violation count. Text and SARIF SHALL remain unchanged
because they are human/CI diagnostic projections rather than baseline interchange formats.

#### Scenario: JSON emits a structured finding key without removing human text

- **WHEN** the runner reports an enforced violation under `--format json`
- **THEN** the violation carries both its existing human `finding` and a `finding_key` object with namespace, code, and named string fields

#### Scenario: A presentation-only change keeps the same machine identity

- **WHEN** a violation's human finding wording changes while its observed fact does not
- **THEN** JSON shows the new `finding` and the unchanged `finding_key`, with outcome and count unchanged

#### Scenario: Gate-mode JSON projects structured stale entries

- **WHEN** a version-2 baseline entry matches no current violation under `--format json`
- **THEN** `stale_baseline` carries its target, rule, human finding, and structured finding key

#### Scenario: Gate-mode JSON faithfully projects a legacy stale entry

- **WHEN** a version-1 baseline entry has no exact current text match under `--format json`
- **THEN** `stale_baseline` carries its target, rule, and finding with `finding_key` set to `null`

#### Scenario: Existing violation metadata remains unchanged

- **WHEN** a file-bearing, anchored, or on-axis violation is projected under JSON or SARIF
- **THEN** its file, anchor, polarity, SARIF location, and shared property-bag behavior remain as previously specified

#### Scenario: The default text format is unchanged

- **WHEN** the runner is invoked without `--format`
- **THEN** it prints the human-readable report exactly as before

#### Scenario: SARIF continues to mirror the reaction

- **WHEN** the runner reports a violation under `--format sarif`
- **THEN** it emits the same SARIF 2.1.0 diagnostic shape and process exit as before, without requiring a structured-key extension
