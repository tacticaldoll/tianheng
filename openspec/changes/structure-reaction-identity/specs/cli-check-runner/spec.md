## ADDED Requirements

### Requirement: Tianheng-owned machine contracts identify semantics

A Tianheng-owned JSON machine contract SHALL identify its semantics with a stable string `format`
rather than a numeric Tianheng schema generation. Baselines SHALL use
`tianheng.baseline/structured-facts`, reaction reports SHALL use
`tianheng.reaction/structured-facts`, and constitution projections SHALL use
`tianheng.constitution/declared-boundaries`. Their existing substantive fields and exit behavior
SHALL remain unchanged. External standards and process protocols, including SARIF 2.1.0 and exit
codes 0/1/2, SHALL retain their specified numbers.

A semantic format identifier SHALL be immutable in meaning. Adding a compatible fact family,
dimension, optional diagnostic, or open field SHALL NOT create a new global format. An incompatible
fact-local identity change SHALL use a new semantic fact shape, not a baseline v3/v4.

#### Scenario: A baseline names its semantic contract

- **WHEN** `--write-baseline` produces JSON
- **THEN** the document declares `format: "tianheng.baseline/structured-facts"` and no numeric Tianheng baseline version

#### Scenario: A new fact does not re-version the document

- **WHEN** an observation dimension adds a new cataloged fact family
- **THEN** the enclosing baseline format remains `tianheng.baseline/structured-facts`

#### Scenario: Reaction and constitution documents name distinct semantics

- **WHEN** `check --format json` and `list --format json` emit their documents
- **THEN** they declare `tianheng.reaction/structured-facts` and `tianheng.constitution/declared-boundaries` respectively

#### Scenario: External numeric standards remain numeric

- **WHEN** the runner emits SARIF or returns a process outcome
- **THEN** SARIF remains 2.1.0 and the process retains exit codes 0, 1, and 2

### Requirement: SARIF fingerprints derive from canonical violation identity

Every non-baselined SARIF result SHALL carry a partial fingerprint under the semantic property key
`tianheng/structured-fact-identity`. Its value SHALL be derived only from a canonical
serialization of governed target, semantic rule key, and structured fact identity. Rule/finding
presentation, reason, severity, file, anchor, polarity, signature diagnostics, owner/tracker, and
result order SHALL NOT affect the fingerprint. The prior `tianhengViolationId/v1` property SHALL
NOT be emitted.

#### Scenario: Presentation changes preserve the fingerprint

- **WHEN** only rule/finding wording or diagnostics change for the same violation identity
- **THEN** the SARIF partial fingerprint remains byte-identical

#### Scenario: A fact change changes the fingerprint

- **WHEN** the target, rule key, or any identity-bearing fact role changes
- **THEN** the partial fingerprint changes

#### Scenario: Reordering results preserves each fingerprint

- **WHEN** unrelated findings are inserted or SARIF results are emitted in another order
- **THEN** every pre-existing violation retains its fingerprint

## MODIFIED Requirements

### Requirement: Machine-readable reports expose structured finding identity

The runner's JSON report SHALL expose each current violation and stale baseline entry with both
human rule/finding presentation and its structured identity roles: governed `target`, semantic
`rule_key`, and `fact` containing semantic type, semantic shape, and canonical named scalar fields.
The structured identity SHALL affect baseline matching but SHALL NOT change outcome, exit code,
severity, file, anchor, polarity, or violation count. No current or stale entry SHALL carry a null
fact or a legacy text-identity provenance because unsupported baselines fail before gating.

Text and SARIF SHALL remain diagnostic projections of the same reaction. SARIF SHALL expose the
canonical identity through its semantic partial fingerprint rather than copying the full fact
object into its message.

#### Scenario: JSON emits structured identity without removing human text

- **WHEN** the runner reports an enforce violation under `--format json`
- **THEN** it carries target, rule key, structured fact, and existing human presentations

#### Scenario: A presentation-only change keeps machine identity

- **WHEN** human wording changes while target, rule key, and observed fact do not
- **THEN** JSON shows the new presentation and unchanged structured identity with the same outcome and count

#### Scenario: Gate-mode JSON projects a structured stale entry

- **WHEN** a semantic baseline entry matches no current violation
- **THEN** `stale_baseline` carries its complete structured identity, presentation, and annotations

#### Scenario: Unsupported baseline data is never projected as a null fact

- **WHEN** a numeric, unmarked, unknown-format, or malformed baseline is supplied
- **THEN** the runner exits 2 before producing a gated stale-entry projection

#### Scenario: Existing violation metadata remains diagnostic

- **WHEN** a file-bearing, anchored, or on-axis violation is projected
- **THEN** its file, anchor, polarity, SARIF location, and property-bag behavior remain available but outside identity
