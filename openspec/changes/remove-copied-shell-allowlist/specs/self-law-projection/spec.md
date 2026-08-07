## MODIFIED Requirements

### Requirement: Self-law projection is generated from the enforced self-constitution

The repository SHALL carry an agent-readable Markdown artifact projecting Tianheng's self-governance law. The projection SHALL be derived from the **same** constitution object the self-governance gate reacts against (`tianheng_constitution()`), never a hand-written restatement, so the projected law and the enforced law cannot diverge into two sources of truth. The projection SHALL cover every boundary the self-constitution declares, each with its target, rule, and declared `reason`. Boundary `because(...)` reasons SHALL be distilled into forward-looking shape declarations, avoiding historical debug text. Crate dependency allowlists SHALL be minimal and un-duplicated, and file-system path canonicalization SHALL be confined crate-wide across observation crates. The `tianheng` shell's normal-dependency allowlist SHALL name only `guibiao`, `hunyi`, `louke`, and `serde_json`; it SHALL NOT permit a direct `xingbiao` edge.

Authored Rust line comments under the `tianheng` shell SHALL NOT restate the shell dependency
declaration with `restrict_dependencies_to(`. A comment that explains the dependency-free
implementation SHALL point to the generated self-law projection instead of copying the declaration or
its current members.

The direct shell-to-metadata direction SHALL be defended by an in-repository fixture evaluated against the unique live `tianheng` dependency boundary selected from `tianheng_constitution()`. The test SHALL NOT redeclare that boundary's allowlist, and the fixture SHALL carry no other forbidden dependency that could satisfy the expected violation.

#### Scenario: The projection carries the enforced self-law

- **WHEN** the self-law projection is generated
- **THEN** it contains every boundary `tianheng_constitution()` declares — each crate boundary with its distilled forward-looking `reason`, with `guibiao`'s allowlist carrying the functional core ⊥ shell clause without a redundant denylist boundary, with `tianheng` restricted to direct normal dependencies on `guibiao`, `hunyi`, `louke`, and `serde_json`, and with `std::fs::canonicalize` boundaries enforced at crate-root subtree depth for `guibiao`, `hunyi`, and `louke`

#### Scenario: Authored shell comments do not copy the dependency declaration

- **WHEN** the repository self-governance tests enumerate Rust source under `crates/tianheng/src`
- **THEN** no line comment contains `restrict_dependencies_to(` and the ANSI dependency rationale points to `AGENTS.self-law.md`, so dependency membership remains owned by the constitution and its generated projection without forbidding executable DSL calls

#### Scenario: A direct shell-to-metadata dependency reacts

- **WHEN** the isolated `tianheng` fixture declares `xingbiao` as its only forbidden direct normal dependency
- **THEN** the live shell dependency boundary selected from `tianheng_constitution()` reports an enforced violation
