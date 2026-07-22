## 1. louke modularization

- [x] 1.1 Create `crates/louke/src/tracked.rs` — move `Tracked` trait + blanket impl, `FoldHasher` struct + `Hasher` impl, and `TidMap` type alias; keep all as `pub(crate)` except `Tracked` (which stays `pub`).
- [x] 1.2 Create `crates/louke/src/dsl.rs` — move `Posture` enum + impl, `RuntimeBoundary` struct + impl, `RuntimeSeamDraft` struct + impl, `RuntimeBoundaryDraft` struct + impl, `OriginEntry` struct + impl; all retain their existing visibility.
- [x] 1.3 Create `crates/louke/src/registry.rs` — move `Seam` struct, `OriginInfo` struct, `Registry` struct + impl, `install` fn, `check_crossing` fn, `__react` fn (`pub`), `set_sink` fn (`pub`), `emit` fn; verify `__react` and any other macro-referenced symbols are included.
- [x] 1.4 Update `crates/louke/src/lib.rs`: add `mod tracked; mod dsl; mod registry;`, add `pub use` re-exports covering every previously-`pub` item, keep `RUNTIME_SEAM_RULE` const, `runtime_seam_rule_line` fn, `assert_boundary!` macro, `register_origin!` macro, and the `#[cfg(feature = "audit")]` gate lines.
- [x] 1.5 Verify all `$crate::` paths inside `assert_boundary!` and `register_origin!` resolve correctly after the move (each referenced symbol must appear in the `lib.rs` re-export list or remain in `lib.rs` directly).

## 2. tianheng text projection skeleton

- [x] 2.1 In `crates/tianheng/src/runner/projection/text.rs`, define the private `ModuleBlockSpec<'a>` struct with fields: `severity: &'a str`, `target: String`, `rule_line: String`, `reason: &'a str`, `anchor: Option<&'a str>`.
- [x] 2.2 Add private `fn render_section(title: &str, blocks: &[ModuleBlockSpec<'_>]) -> String` — contains the `if-empty → text_section → for → format!(block) → anchor_line` skeleton (replacing the duplicated pattern); use the same format string as the existing `module_block` helper.
- [x] 2.3 Rewrite each of the eight `*_text` functions to map its boundary slice into `Vec<ModuleBlockSpec>` and delegate to `render_section`. Each function sets `target` to the appropriate prefix ("module X in Y", "trait X in Y", "subtree X in Y", "crate X") and `rule_line` to the existing rule-string logic.
- [x] 2.4 Remove the now-unused private `module_block` helper.
- [x] 2.5 Leave `runtime_text` untouched (extra `posture:` field makes it a permanent exception).

## 3. tianheng document projection dedup

- [x] 3.1 In `crates/tianheng/src/runner/projection/document.rs`, extract a private `shape_operand_boundary_json(boundary: &impl …, rule: &str) -> Value` helper (or equivalent — see design note on trait vs closure) shared by `dyn_trait_boundary_json` and `impl_trait_boundary_json`.
- [x] 3.2 Rewrite `dyn_trait_boundary_json` and `impl_trait_boundary_json` to call the shared helper, differing only in the rule constant passed in.

## 4. Verification (Definition of Done)

- [x] 4.1 `cargo build --workspace`
- [x] 4.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 4.3 `cargo clippy --workspace -- -D warnings`
- [x] 4.4 `cargo clippy -p louke -- -D warnings`
- [x] 4.5 `cargo fmt --all --check`
- [x] 4.6 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [x] 4.7 `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- [x] 4.8 `cargo deny check`
- [x] 4.9 `bash scripts/test_release_coherence.sh && bash scripts/check_release_coherence.sh`
- [x] 4.10 `bash scripts/test_examples.sh`
- [x] 4.11 Verify self-governance test passes without weakening any boundary in `self_governance.rs`.

## 5. OpenSpec sync

- [x] 5.1 Update `openspec/specs/internal-refactoring/spec.md` with the delta from this change's `specs/internal-refactoring/spec.md`.
- [ ] 5.2 Commit and open PR into `release/0.2.3`.
