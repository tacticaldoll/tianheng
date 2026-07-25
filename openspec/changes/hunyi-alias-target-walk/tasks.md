# tasks: Hunyi Non-Generic Type Alias Target Walk Implementation Plan

- [ ] Implement `alias_nominal_targets` helper in `crates/hunyi/src/resolve/mod.rs` to recursively collect nominal target paths from non-generic compound type constructors (`Type::Reference`, `Type::Tuple`, `Type::Slice`, `Type::Array`, `Type::Group`, `Type::Paren`). <!-- id: 0 -->
- [ ] Update `crates/hunyi/src/scan.rs` `Item::Type` handling to iterate through all extracted nominal targets and register resolved paths into `scan.aliases`. <!-- id: 1 -->
- [ ] Add unit tests in `crates/hunyi/src/tests.rs` verifying non-generic tuple, reference, and slice type alias target resolution and signature coupling reactions. <!-- id: 2 -->
- [ ] Verify full pre-flight DoD gates (`cargo build`, `clippy -D warnings`, `cargo fmt --check`, `cargo test --workspace --all-features`, `self_governance`). <!-- id: 3 -->
