## ADDED Requirements

### Requirement: Composed runtime audit preserves Cargo target roots

The composed check SHALL obtain every workspace member's exact library-or-binary target root from
the shared workspace-data substrate and pass those files to the runtime probe audit. It SHALL NOT
reduce roots to directories or guess conventional filenames. Root-resolution or reachable-source
errors SHALL retain the composed check's constitution-error precedence.

#### Scenario: Composed check rejects orphan-only coverage

- **WHEN** a workspace member's declared seam is probed only in an orphan `.rs` file outside the module graph
- **THEN** the composed check reports the seam unprobed and exits according to its declared severity

#### Scenario: Custom Cargo root is passed exactly

- **WHEN** a workspace member declares a custom library or binary source path
- **THEN** the composed runtime audit begins at Cargo's reported source file and observes its reachable probes
