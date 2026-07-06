# constitution-projection Specification

## Purpose

Render the declared constitution as a read-only projection of the Rust source of
truth, so the effective law is legible without reading code or triggering a
violation. Delivered as the runner's `list` command (through `tianheng::run`, so every
adopter gets it), it serves a steward reviewing an amendment, an operator reading a
CI log, and a tool or agent that wants the declared law. A projection is never a
reaction: `list` observes nothing, claims no target or drift type, and always
exits 0.
## Requirements
### Requirement: List command projects the declared constitution

The runner SHALL provide a `list` command that renders the caller-supplied constitution as a human-readable projection on standard output. For each boundary it SHALL show the severity, the kind, the target (a module boundary SHALL also show its crate), the rule together with its parameters (e.g. an allowlist of crate names, the forbidden crate names, or the forbidden module path), and the boundary's reason. The projection SHALL be derived only from the declared constitution; it MUST NOT invent a field for data the constitution does not hold.

#### Scenario: List renders each boundary

- **WHEN** the runner is invoked as `list` against a constitution holding a crate boundary and a module boundary
- **THEN** standard output names the constitution and, for each boundary, its severity, kind, target, rule with parameters, and reason

#### Scenario: A crate boundary's rule parameters are shown

- **WHEN** a deny-external boundary carries an allowlist, a forbid-dependency-on boundary names crates, or a restrict-to boundary names an allowlist
- **THEN** the projection shows those crate names alongside the rule

#### Scenario: An empty constitution lists cleanly

- **WHEN** the runner is invoked as `list` against a constitution with no boundaries
- **THEN** it prints the constitution name and an empty boundary set and exits `0`, never erroring

### Requirement: List is a projection, not a reaction

The `list` command SHALL observe nothing: it SHALL NOT read `cargo metadata`, SHALL NOT require `--manifest-path`, and SHALL NOT evaluate any boundary against a workspace. It SHALL always exit `0`, because a projection cannot be violated. The `list` command therefore claims no target type or drift type and performs no pass/fail judgment.

#### Scenario: List needs no manifest path

- **WHEN** the runner is invoked as `list` with no `--manifest-path`
- **THEN** it prints the constitution projection and exits `0`, never treating the absent manifest path as a usage error

#### Scenario: List always exits 0

- **WHEN** the runner is invoked as `list` for any valid constitution
- **THEN** it exits `0`, never `1` (it makes no reaction) and never `2` (it reads no workspace)

### Requirement: List honors the format flag

The `list` command SHALL honor `--format`: human-readable text by default, a JSON document under `--format json` (and `--format=json`), or a Markdown document under `--format markdown` (and `--format=markdown`). The JSON SHALL faithfully project the constitution — its name and a `boundaries` array whose entries carry the `kind`, `target`, `severity`, `reason`, and the rule with its parameters — so a tool or agent can read the declared law. The `target` SHALL follow the existing report convention: the crate name for a crate boundary and the module path for a module boundary.

The Markdown SHALL project the **same declared law as the text and JSON projections, across every dimension** — the static constitution and **each non-empty semantic capability** and the runtime boundaries — so it never carries less than the JSON document. This coverage SHALL NOT be a hand-maintained enumeration of capability names (which drifts as capabilities are added); the Markdown SHALL derive its sections from the same document the JSON projects, and a reaction (a test) SHALL assert that every dimension the JSON document emits has a corresponding Markdown section. For each declared boundary the Markdown SHALL render its `target`, what it forbids or restricts (its rule), and its `reason` (the declared intent), so an agent can read the architectural law and its prohibitions before proposing a change. The Markdown SHALL **foreground the `reason`**: within each boundary block the reason SHALL be rendered before the rule and before the kind/severity classification, so the agent-loaded idiom leads with the principle (the repair/imitation hint) rather than burying it under mechanical metadata; the boundary's `target` SHALL remain the block heading, and a boundary with no declared reason SHALL render no reason element. This foregrounding is an **ordering invariant only** — it does NOT pin the exact Markdown layout (the blockquote choice, wording, spacing, or added fields), which under Contract B (capability `self-law-projection`) remains free to evolve; that capability's standing prohibition on pinning the exact layout as a machine contract is unchanged and remains the binding guard, of which this ordering invariant is a deliberate, narrow exception. A dimension with no declared boundaries SHALL add no section, consistent with the text and JSON projections. The Markdown SHALL be a pure projection carrying no information absent from the JSON, and like `list` as a whole SHALL NOT react. An unrecognized `--format` value SHALL be a usage error that exits `2`, never a silent fallback, consistent with the `check` command.

The JSON and Markdown projections SHALL additionally surface a boundary's declared durable `anchor` — the stable governance pointer (e.g. `"ADR-014"`) distinct from the free-text `reason` — so the projected law carries the same durable coordinate an agent or steward reads. The anchor SHALL be surfaced **only when the boundary declares one**: a boundary with no anchor SHALL add no `anchor` field to its JSON entry and no anchor element to its Markdown block, so the projection of an anchor-less constitution is byte-identical to before this capability existed. This Some-only discipline is the same one the operand and `including_trait_impls` params already follow, and it preserves the byte-stability the self-law projection's staleness check depends on. The anchor SHALL remain a pure projection of the declared boundary, carrying no information absent from the constitution.

#### Scenario: List emits a JSON projection

- **WHEN** the runner is invoked as `list --format json`
- **THEN** standard output is a JSON document carrying the constitution name and a `boundaries` array, each entry with its `kind` (`crate` or `module`), `severity`, `reason`, and rule parameters

#### Scenario: List emits a Markdown projection

- **WHEN** the runner is invoked as `list --format markdown`
- **THEN** standard output is a Markdown document carrying the constitution name and, for each declared boundary, its target, its rule (what it forbids or restricts), and its declared reason

#### Scenario: Markdown projection covers every non-empty dimension

- **WHEN** the runner is invoked as `list --format markdown` against a constitution declaring static, semantic, and runtime boundaries
- **THEN** the Markdown document includes the static boundaries and a section for each non-empty semantic capability and the runtime boundaries, so it carries no less than the JSON projection

#### Scenario: Markdown coverage is pinned by a reaction, not an enumeration

- **WHEN** a constitution declares one boundary of every semantic capability the JSON document can emit
- **THEN** a test asserts the Markdown projection renders a section for every dimension key the JSON document carries, so a newly-added capability that omits its Markdown section fails CI rather than silently under-projecting

#### Scenario: Markdown foregrounds the reason before the rule and classification

- **WHEN** the runner is invoked as `list --format markdown` against a boundary with a declared reason
- **THEN** within that boundary's block the reason appears before the rule, and the rule appears before the kind/severity classification (the reason leads the block, after the target heading)

#### Scenario: A boundary without a reason foregrounds nothing

- **WHEN** a boundary has no declared reason
- **THEN** its Markdown block renders no reason element and no orphan separator, and the rule and classification render as before

#### Scenario: Foregrounding is an ordering invariant, not a layout freeze

- **WHEN** the foregrounding requirement is enforced by a test
- **THEN** the test asserts only the foregrounding order (reason before rule before classification) and never a byte-for-byte snapshot of the Markdown, so the layout stays free to evolve under Contract B

#### Scenario: Markdown projection still does not react

- **WHEN** the runner is invoked as `list --format markdown` against any constitution
- **THEN** it exits `0` and prints the projection, never evaluating the workspace or producing a violation

#### Scenario: An unknown format to list is a usage error

- **WHEN** the runner is invoked as `list --format` with a value other than `text`, `json`, or `markdown`
- **THEN** it prints usage guidance and exits `2`

#### Scenario: An anchored boundary surfaces its anchor in the JSON and Markdown

- **WHEN** the runner is invoked as `list --format json` (and `--format markdown`) against a constitution holding a boundary that declared `.with_anchor("ADR-014")`
- **THEN** that boundary's JSON entry carries `anchor` equal to `"ADR-014"` and its Markdown block renders the anchor

#### Scenario: An anchor-less boundary omits the anchor entirely

- **WHEN** the runner is invoked as `list` against a constitution holding a boundary that declared no anchor
- **THEN** that boundary's JSON entry carries no `anchor` key and its Markdown block renders no anchor element, so the projection is byte-identical to before the anchor capability existed

### Requirement: List projects runtime boundaries

The `list` command SHALL project the constitution's declared **runtime** boundaries alongside the
static and semantic ones, in both the human-readable and the JSON forms, following the same
projection contract: for each runtime boundary it SHALL show the severity, that it is a runtime
boundary, the seam (its target), the rule together with its allowed origins, and the reason. The
projection SHALL be derived only from the declared constitution and MUST NOT invent a field the
constitution does not hold. A constitution with no runtime boundaries SHALL project no runtime
section, leaving the existing static and semantic projection byte-identical. `list` SHALL remain a
projection, not a reaction: it observes no workspace and always exits `0`.

#### Scenario: List renders each runtime boundary

- **WHEN** the runner is invoked as `list` against a constitution holding a runtime boundary
- **THEN** standard output shows, for that boundary, its severity, that it is a runtime boundary, its seam, the allowed origins, and its reason

#### Scenario: A runtime boundary appears in the JSON projection

- **WHEN** the runner is invoked as `list --format json` against a constitution holding a runtime boundary
- **THEN** the JSON document carries the runtime boundary with its kind, target (the seam), severity, allowed origins, and reason

#### Scenario: No runtime boundaries leaves the projection unchanged

- **WHEN** the constitution declares no runtime boundaries
- **THEN** `list` emits no runtime section and the static and semantic projection is identical to before this capability

