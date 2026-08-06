# gate-shape-contract (delta)

## MODIFIED Requirements

### Requirement: The contract SHALL be projected into a generated, staleness-checked document

The reaction SHALL emit a projection of the surface and its conformance, blessed by an environment variable
and diffed on every run, exactly as `AGENTS.self-law.md` and `docs/observation-bounds.md` are. A
hand-maintained table of this shape is the drift class this repository has closed twice; the projection is
what stops the capability's own description of the surface from rotting.

The projection SHALL state what it does not claim, in its own header rather than only in the reaction's
comments. A projection implying completeness would mislead exactly where it is most trusted.

That disclosure SHALL be **derived from the specification, not typed into the generator**, and held to it in both
directions: a declared bound with no note is one the document does not mention, and a note whose bound was retired
discloses a bound that no longer exists. Its figure SHALL be the derived list's length. A literal in a generator's
template is the one place a projection cannot self-correct — the freshness check compares the generator's own text
with itself — and measured, this projection typed both a figure and the list of bounds and silently omitted one
declared in the same window.

#### Scenario: The projection is stale

- **WHEN** the surface or a gate's conformance changes and the projection is not regenerated
- **THEN** the reaction fails and names the blessing command, so the document cannot drift from what was
  measured

#### Scenario: The projection names the properties it does not check

- **WHEN** a reader opens the projection
- **THEN** its header enumerates the semantic properties declared as bounds below, so a reader can see what
  conformance in this document does and does not mean

#### Scenario: A bound is declared with no note in the projection

- **WHEN** the specification declares a bound the generator holds no note for
- **THEN** the reaction fails, because the generated document would not mention it and the freshness check cannot
  see the omission

#### Scenario: The projection notes a bound the specification no longer declares

- **WHEN** the generator holds a note whose bound has been retired
- **THEN** the reaction fails, so a disclosure cannot outlive the bound it describes
