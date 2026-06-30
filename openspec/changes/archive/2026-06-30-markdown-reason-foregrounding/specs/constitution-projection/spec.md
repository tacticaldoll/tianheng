## MODIFIED Requirements

### Requirement: List honors the format flag

The `list` command SHALL honor `--format`: human-readable text by default, a JSON document under `--format json` (and `--format=json`), or a Markdown document under `--format markdown` (and `--format=markdown`). The JSON SHALL faithfully project the constitution — its name and a `boundaries` array whose entries carry the `kind`, `target`, `severity`, `reason`, and the rule with its parameters — so a tool or agent can read the declared law. The `target` SHALL follow the existing report convention: the crate name for a crate boundary and the module path for a module boundary.

The Markdown SHALL project the **same declared law as the text and JSON projections, across every dimension** — the static constitution and each non-empty semantic capability (signature-coupling, trait-impl-locality, visibility, forbidden-marker) and the runtime boundaries — so it never carries less than the JSON document. For each declared boundary the Markdown SHALL render its `target`, what it forbids or restricts (its rule), and its `reason` (the declared intent), so an agent can read the architectural law and its prohibitions before proposing a change. The Markdown SHALL **foreground the `reason`**: within each boundary block the reason SHALL be rendered before the rule and before the kind/severity classification, so the agent-loaded idiom leads with the principle (the repair/imitation hint) rather than burying it under mechanical metadata; the boundary's `target` SHALL remain the block heading, and a boundary with no declared reason SHALL render no reason element. This foregrounding is an **ordering invariant only** — it does NOT pin the exact Markdown layout (the blockquote choice, wording, spacing, or added fields), which under Contract B (capability `self-law-projection`) remains free to evolve; that capability's standing prohibition on pinning the exact layout as a machine contract is unchanged and remains the binding guard, of which this ordering invariant is a deliberate, narrow exception. A dimension with no declared boundaries SHALL add no section, consistent with the text and JSON projections. The Markdown SHALL be a pure projection carrying no information absent from the JSON, and like `list` as a whole SHALL NOT react. An unrecognized `--format` value SHALL be a usage error that exits `2`, never a silent fallback, consistent with the `check` command.

#### Scenario: List emits a JSON projection

- **WHEN** the runner is invoked as `list --format json`
- **THEN** standard output is a JSON document carrying the constitution name and a `boundaries` array, each entry with its `kind` (`crate` or `module`), `severity`, `reason`, and rule parameters

#### Scenario: List emits a Markdown projection

- **WHEN** the runner is invoked as `list --format markdown`
- **THEN** standard output is a Markdown document carrying the constitution name and, for each declared boundary, its target, its rule (what it forbids or restricts), and its declared reason

#### Scenario: Markdown projection covers every non-empty dimension

- **WHEN** the runner is invoked as `list --format markdown` against a constitution declaring static, semantic, and runtime boundaries
- **THEN** the Markdown document includes the static boundaries and a section for each non-empty semantic capability and the runtime boundaries, so it carries no less than the JSON projection

#### Scenario: Markdown foregrounds the reason before the rule and classification

- **WHEN** the runner is invoked as `list --format markdown` against a boundary with a declared reason
- **THEN** within that boundary's block the reason appears before the rule, and the rule appears before the kind/severity classification (the reason leads the block, after the target heading)

#### Scenario: A boundary without a reason foregrounds nothing

- **WHEN** a boundary has no declared reason
- **THEN** its Markdown block renders no reason element and no orphan separator, and the rule and classification render as before

#### Scenario: Foregrounding is an ordering invariant, not a layout freeze

- **WHEN** the foregrounding requirement is enforced by a test
- **THEN** the test asserts only the foregrounding order (reason before rule before classification) and never a byte-for-byte snapshot of the Markdown, so the layout stays free to evolve under Contract B
