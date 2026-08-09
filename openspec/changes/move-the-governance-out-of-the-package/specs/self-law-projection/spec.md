## MODIFIED Requirements

### Requirement: Self-law projection is generated from the enforced self-constitution

The constitution this projection is generated from SHALL be **library code**, not a function inside a test
file. It is written with the product's own declaration API — the capability applied to its own author — and a
declaration living among a dozen `#[test]` functions reads as a test rather than as the repository's law, which
is how a governance document came to describe the reactions beside it as something they are not.

Authored text SHALL NOT restate a declared dependency allowlist, and the reaction holding that SHALL read
**every** declared allowlist against **every** tracked governance document, not one crate's line comments
against one dimension's. Measured before this change, the reaction read only Rust line comments under the
`tianheng` shell and only the shell's allowlist, while `PROJECT.md` named every member of `guibiao`'s live
allowlist — the same second source of truth, in a file class nothing scanned. A rule enforced at one site and
not its neighbour is a rule about the site.

What a declaration cannot carry SHALL stay prose: why a boundary exists, what it protects, the narrative of
the family. What it can carry — the membership — belongs to the declaration and its projection, and the repair
for a restatement is a pointer to `AGENTS.self-law.md`.

#### Scenario: A governance document names every member of a declared allowlist

- **WHEN** a tracked governance document names every member of any live `restrict_dependencies_to` allowlist
- **THEN** the reaction fails, naming the document, the crate whose allowlist was copied, and the members —
  and directs the repair to the projection rather than to a rewording

#### Scenario: The law is reached as a library

- **WHEN** the projection is generated and the reaction runs the constitution
- **THEN** both read the same exported declaration, so a projection cannot be generated from one definition
  while the reaction evaluates another
