## MODIFIED Requirements

### Requirement: List honors the format flag

The `list` command SHALL honor `--format`: human-readable text by default, a JSON document under `--format json` (and `--format=json`), or a Markdown document under `--format markdown` (and `--format=markdown`). The JSON SHALL faithfully project the constitution — its name and a `boundaries` array whose entries carry the `kind`, `target`, `severity`, `reason`, and the rule with its parameters — so a tool or agent can read the declared law. The `target` SHALL follow the existing report convention: the crate name for a crate boundary and the module path for a module boundary.

The Markdown SHALL project the **same declared law as the text and JSON projections, across every dimension** — the static constitution and each non-empty semantic capability (signature-coupling, trait-impl-locality, visibility, forbidden-marker) and the runtime boundaries — so it never carries less than the JSON document. For each declared boundary the Markdown SHALL render its `target`, what it forbids or restricts (its rule), and its `reason` (the declared intent), so an agent can read the architectural law and its prohibitions before proposing a change. A dimension with no declared boundaries SHALL add no section, consistent with the text and JSON projections. The Markdown SHALL be a pure projection carrying no information absent from the JSON, and like `list` as a whole SHALL NOT react. An unrecognized `--format` value SHALL be a usage error that exits `2`, never a silent fallback, consistent with the `check` command.

#### Scenario: List emits a JSON projection

- **WHEN** the runner is invoked as `list --format json`
- **THEN** standard output is a JSON document carrying the constitution name and a `boundaries` array, each entry with its `kind` (`crate` or `module`), `severity`, `reason`, and rule parameters

#### Scenario: List emits a Markdown projection

- **WHEN** the runner is invoked as `list --format markdown`
- **THEN** standard output is a Markdown document carrying the constitution name and, for each declared boundary, its target, its rule (what it forbids or restricts), and its declared reason

#### Scenario: Markdown projection covers every non-empty dimension

- **WHEN** the runner is invoked as `list --format markdown` against a constitution declaring static, semantic, and runtime boundaries
- **THEN** the Markdown document includes the static boundaries and a section for each non-empty semantic capability and the runtime boundaries, so it carries no less than the JSON projection

#### Scenario: Markdown projection still does not react

- **WHEN** the runner is invoked as `list --format markdown` against any constitution
- **THEN** it exits `0` and prints the projection, never evaluating the workspace or producing a violation

#### Scenario: An unknown format to list is a usage error

- **WHEN** the runner is invoked as `list --format` with a value other than `text`, `json`, or `markdown`
- **THEN** it prints usage guidance and exits `2`
