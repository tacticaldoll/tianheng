## MODIFIED Requirements

### Requirement: Composition SHALL introduce no trait object

The lexical reaction SHALL inspect every Rust source file recursively below the Tianheng crate's `src` directory.
Directory nesting or module visibility SHALL NOT remove a file from the corpus, because a private nested module can
still expose an item through a public re-export. The existing one-line recognizer and its continuation-line stated
bound remain unchanged.

#### Scenario: A trait object appears in a nested source file

- **WHEN** a Rust source below a nested `src` directory contains a one-line public trait-object signature
- **THEN** the reaction reports it exactly as it reports the same signature in a top-level source file
