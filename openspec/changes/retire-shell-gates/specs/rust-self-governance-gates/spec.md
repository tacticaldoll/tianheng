## ADDED Requirements

### Requirement: Rust-native self-governance test suite
The repository self-governance reactions SHALL be executed as Rust integration tests under `crates/tianheng/tests/` rather than bash shell check scripts.

#### Scenario: Verification of workspace self-governance via cargo test
- **WHEN** developer or CI runs `cargo test -p tianheng`
- **THEN** all workspace self-governance reactions execute and report clean exit (0) or violation exit (1)

### Requirement: Distinction of projections from reactions and product code
Contract projections and censuses SHALL be formally classified as derived text views, explicitly documented as NOT reactions, NOT governance, and NOT shipped product code.

#### Scenario: Freshness check of projected documents
- **WHEN** developer or CI runs `BLESS=1 cargo test -p tianheng`
- **THEN** projected document contents are generated from Rust sources of truth and verified without adding runtime code to shipped crates
