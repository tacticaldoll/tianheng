## 1. Foreground the reason in the violation text block

- [x] 1.1 In `crates/tianheng/src/runner.rs` `report_violations`, reorder each violation's block to **Reason → Boundary → Rule → Found → (File) → Reaction**: emit `Reason:\n  <reason>` right after the `{header}` line, then the boundary target, rule, finding, then (file), then the reaction line.
- [x] 1.2 Keep the header/severity logic, the baselined-skip counting, and the trailing "pre-existing violation(s) suppressed" line unchanged.

## 2. Surface the offending file

- [x] 2.1 When a violation's `file` is `Some(path)`, emit `File:\n  <path>` in the block (placed before the Reaction line); when `None`, emit no file element — never a placeholder. The predicate is `Some` vs `None` (`if let Some(path) = &violation.file`), NOT emptiness — `file` is `Option<String>`, so do not add an `is_empty()` guard the model never intended.
- [x] 2.2 Confirm the accessor: read the file off the `Violation` the same way the JSON projection does (do not re-derive it).

## 3. Group violations by boundary (presentation layer only)

- [x] 3.1 In `report_violations`, iterate the violations in a stably-sorted order by `(target, rule)` (sort a local view/indices; do NOT mutate `Report`), so multiple findings under one boundary render consecutively.
- [x] 3.2 Confirm `report_json` / the JSON path is untouched — JSON keeps its existing order and content.

## 4. (E) constitution_markdown doctest

- [x] 4.1 Add a doc-test on `constitution_markdown` (in `runner.rs`) that builds a small `Constitution`, renders it, and asserts a stable property (e.g. the output contains the constitution name and a boundary's reason) — locking the README adopter recipe as a CI-run example. Assert a property, not a byte snapshot (Contract B).

## 5. Verify (Definition of Done)

- [x] 5.1 Add/adjust a unit test asserting the text report's ordering invariant (reason before target/rule/finding) and that a violation with a file shows `File:` while one without shows none; and that multi-boundary violations cluster by `(target, rule)`. Construct the violations explicitly (e.g. `Violation::new(...).with_file(Some(...))` and one without), not via a live scan; assert ordering/presence, not a literal block snapshot.
- [x] 5.2 Add/confirm a test that the JSON projection (`report_json`) is unchanged by this change (same fields/order as before).
- [x] 5.3 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --all-features` passes; `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --all-features` all clean (verify by exit code, not piped output).
- [x] 5.4 Confirm no JSON/reaction/exit-code/baseline change; no `repair_hint` or derived field; no `Cargo.toml` version change (bump deferred to the release commit).
