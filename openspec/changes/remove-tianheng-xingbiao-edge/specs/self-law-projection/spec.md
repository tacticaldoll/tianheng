## MODIFIED Requirements

### Requirement: Self-law projection is generated from the enforced self-constitution

The repository SHALL carry an agent-readable Markdown artifact projecting Tianheng's self-governance law. The projection SHALL be derived from the **same** constitution object the self-governance gate reacts against (`tianheng_constitution()`), never a hand-written restatement, so the projected law and the enforced law cannot diverge into two sources of truth. The projection SHALL cover every boundary the self-constitution declares, each with its target, rule, and declared `reason`. Boundary `because(...)` reasons SHALL be distilled into forward-looking shape declarations, avoiding historical debug text. Crate dependency allowlists SHALL be minimal and un-duplicated, and file-system path canonicalization SHALL be confined crate-wide across observation crates. The `tianheng` shell's normal-dependency allowlist SHALL name only `guibiao`, `hunyi`, `louke`, and `serde_json`; it SHALL NOT permit a direct `xingbiao` edge.

#### Scenario: The projection carries the enforced self-law

- **WHEN** the self-law projection is generated
- **THEN** it contains every boundary `tianheng_constitution()` declares — each crate boundary with its distilled forward-looking `reason`, with `guibiao`'s allowlist carrying the functional core ⊥ shell clause without a redundant denylist boundary, with `tianheng` restricted to direct normal dependencies on `guibiao`, `hunyi`, `louke`, and `serde_json`, and with `std::fs::canonicalize` boundaries enforced at crate-root subtree depth for `guibiao`, `hunyi`, and `louke`

#### Scenario: A direct shell-to-metadata dependency reacts

- **WHEN** the `tianheng` crate declares `xingbiao` as a direct normal dependency
- **THEN** the self-governance reaction reports an enforced violation against the shell dependency boundary
