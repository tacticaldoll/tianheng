## 1. Transparent Control-Flow Macro Support (guibiao)

- [ ] 1.1 Add `is_transparent_macro_name` recognition for `cfg_if` in `crates/guibiao/src/module_scan/lexer.rs`
- [ ] 1.2 Update `strip_macro_bodies_tracked` to preserve `cfg_if!` body structural tokens while stripping non-transparent macro bodies
- [ ] 1.3 Add unit tests in `guibiao::module_scan::lexer` and `use_scan` verifying `cfg_if!`-wrapped `use` and `mod` declarations

## 2. Ancestor Glob Hazard Detection (guibiao)

- [ ] 2.1 Update `crates/guibiao/src/module_scan/use_scan.rs` and module boundary checkers to evaluate `path_within(forbidden_target, glob_base)` for ancestor glob imports
- [ ] 2.2 Emit an enforced Glob Hazard violation (exit 1) when an observed wildcard import base path is an ancestor of a forbidden module target
- [ ] 2.3 Add unit tests in `guibiao` verifying ancestor glob hazard fail-closed detection and plain ancestor import clean status

## 3. Dogfood Fixtures & Pre-Flight Verification

- [ ] 3.1 Create `cfg_if_violation` and `glob_hazard_violation` test fixtures under `crates/tianheng/tests/fixtures/`
- [ ] 3.2 Integrate fixture checks into `crates/tianheng/tests/self_governance.rs`
- [ ] 3.3 Execute full Definition of Done pre-flight checks (`cargo test`, `clippy`, `test_examples.sh`) to verify zero regressions
