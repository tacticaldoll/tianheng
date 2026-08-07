## MODIFIED Requirements

### Requirement: Every generated document and every projection holder SHALL correspond in both directions

The register SHALL inspect a Rust holder with Rust line-comment semantics and a shell holder with shell
line-comment semantics. It SHALL NOT use one language-blind executed region for both mechanisms.

#### Scenario: Holder text begins with a language-specific marker

- **WHEN** a potential holder line begins with `//` in Rust or `#` in shell
- **THEN** the matching language region excludes the comment without excluding the other language's executable text
