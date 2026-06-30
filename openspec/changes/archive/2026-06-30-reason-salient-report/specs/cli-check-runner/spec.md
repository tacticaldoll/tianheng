## ADDED Requirements

### Requirement: Human violation report foregrounds the reason

In the human-readable (text) report that `check` prints for an enforced or advisory violation, the runner SHALL **foreground the `reason`**: the reason SHALL be rendered before the mechanical fields (the boundary target, the rule, and the finding), so the agent reading the reaction leads with the principle and repair direction rather than the mechanical detail. The report SHALL surface the offending `file` when the violation carries one (the "where to repair"), and SHALL omit the file element when the violation has none (a faithful absence, never a fabricated location). The runner SHALL group violations by boundary — ordering the text report's violations by `(target, rule)` — so that multiple findings under one boundary appear together and the reason is read once per boundary.

This governs the **human text report only** — an ordering/grouping/presence invariant over already-observed fields. It does NOT change the JSON projection (the machine contract under "Machine-readable report format"), and it introduces no derived or invented field (no `repair_hint`): the reason is shown as declared, the file as observed.

#### Scenario: The reason leads the violation block

- **WHEN** `check` reports an enforced violation as text
- **THEN** within that violation's block the reason appears before the boundary target, the rule, and the finding

#### Scenario: The offending file is shown when present, omitted when absent

- **WHEN** a reported violation carries an offending `file`
- **THEN** the text report shows that file as the repair location; and **WHEN** a violation carries no file, **THEN** the report shows no file element rather than a fabricated one

#### Scenario: Violations are grouped by boundary

- **WHEN** `check` reports multiple violations spanning more than one boundary as text
- **THEN** the report orders them by `(target, rule)` so all findings under one boundary appear consecutively

#### Scenario: The JSON projection is unchanged

- **WHEN** the same outcome is emitted under `--format json`
- **THEN** the JSON content and field order are exactly as before this change — the foregrounding, file-surfacing, and grouping are presentation of the text report only
