## ADDED Requirements

### Requirement: Markdown treats durable anchors as structural metadata

The Markdown constitution projection SHALL render a declared durable `anchor` as its own boundary
element, distinct from the foregrounded `reason` and from the rule's inline parameters. The
parameter renderer MUST classify `anchor` as structural metadata and MUST NOT include it in the
parenthesized rule parameters. A boundary with no declared anchor SHALL render no anchor element
and SHALL retain its prior Markdown bytes.

#### Scenario: Declared anchor is separate from rule parameters

- **WHEN** a projected boundary declares `.with_anchor("ADR-014")` and also carries rule parameters
- **THEN** its Markdown block renders a standalone anchor element containing `ADR-014`, while the
  rule's parenthesized parameters contain no `anchor`

#### Scenario: Missing anchor changes nothing

- **WHEN** a projected boundary declares no durable anchor
- **THEN** its Markdown block contains no anchor element and remains byte-identical to the
  pre-anchor projection
