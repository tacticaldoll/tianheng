## ADDED Requirements

### Requirement: The Three-Layer Agent Law structure is formalized and documented

The repository documentation SHALL define and teach the **Three-Layer Agent Law** structure: Layer 1 (Universal Preamble), Layer 2 (Generated Projection Body from `constitution_markdown()`), and Layer 3 (Rust Law Source governed by `CODEOWNERS`). The preamble discipline SHALL be explicitly documented in [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md), restricting preambles to universal meta-instructions and vocabulary while forbidding crate-specific architectural claims.

#### Scenario: COOKBOOK documents the Three-Layer Agent Law recipe

- **WHEN** an adopter reads [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md)
- **THEN** it contains a dedicated recipe teaching how to assemble the preamble, generate the projection body, gate staleness with `GovernanceTest`, and update the law with `BLESS=1`
