## 1. Lift: `GnomonConstitution` → `Constitution`

- [x] 1.1 Add `impl From<GnomonConstitution> for Constitution` in `crates/tianheng/src/lib.rs` (`static_` = input, `semantic` = `SemanticBoundaries::default()`, `runtime` = `Vec::new()`); doc it as the projection-side lift.
- [x] 1.2 Confirm `tianheng_constitution()` and `tianheng_governs_itself()` in `self_governance.rs` are unchanged (the dogfood proof `check(&tianheng_constitution(), &manifest)` stays byte-for-byte).

## 2. Public Markdown helper (contract B)

- [x] 2.1 Add `pub fn constitution_markdown(constitution: &Constitution) -> String` in `crates/tianheng/src/runner.rs` returning `list_markdown(&list_document(constitution))` verbatim — it MUST add nothing (no preamble, no trailing newline), so it matches the CLI byte for byte. Keep `list_document`/`list_markdown` private.
- [x] 2.2 Write its doc contract: human/agent-readable, for display/review/LLM context, layout MAY evolve in any compatible release; machine consumers use the JSON projection.
- [x] 2.3 Re-export `constitution_markdown` from the crate root so the test reaches it as `tianheng::constitution_markdown` (it is an integration test and sees only `pub`).
- [x] 2.4 Unit test in `runner.rs`: assert **byte-exact** `constitution_markdown(&c) == list_markdown(&list_document(&c))` for a representative `c` (the helper adds nothing — guards against a parallel-projection or stray-newline divergence).

## 3. Self-law projection + staleness gate (contract A)

- [x] 3.1 Add `SELF_LAW_PREAMBLE` const in `self_governance.rs` — only how to read the projection + the reaction loop (declare in code / observe with source / react 0·1·2 / repair toward reason / never weaken / 三儀 measure, 三司 administer); NO crate-specific architectural claims.
- [x] 3.2 Add `render_self_law_doc() -> String` = `SELF_LAW_PREAMBLE` + join + `constitution_markdown(&Constitution::from(tianheng_constitution()))` (whole file generated). The seam newline between preamble and projection is owned here (the doc-builder), never smuggled into the helper.
- [x] 3.3 Add `workspace_root()` helper (`CARGO_MANIFEST_DIR/../..`): `Some(root)` when `Cargo.toml` exists; `None` otherwise, asserting loudly when `TIANHENG_WORKSPACE_TESTS` is set (same repo-only discipline as `workspace_manifest()`).
- [x] 3.4 Add `self_law_projection_is_fresh()` test: skip when `workspace_root()` is `None`; on `BLESS=1` overwrite `<root>/AGENTS.self-law.md` and return; else `assert_eq!` checked-in file == `render_self_law_doc()`, with a message naming the file and the regenerate command.
- [x] 3.5 Generate `AGENTS.self-law.md` at repo root via `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`; commit the generated file.

## 4. Entry-point doc

- [x] 4.1 Add one line to `AGENTS.md` pointing agents to `AGENTS.self-law.md` (generated from `self_governance.rs`, staleness-checked) for Tianheng's own enforced law.

## 5. Verify (Definition of Done)

- [x] 5.1 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --all-features` passes (governance + freshness + helper tests).
- [x] 5.2 `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all --check` clean.
- [x] 5.3 `cargo doc --workspace --no-deps --all-features` clean (the new `pub fn`'s doc contract intra-doc-links resolve).
- [x] 5.4 Confirm no public API beyond the two additive items (the `From` impl + `constitution_markdown`); `list`/`check` behavior, formats, and exit codes unchanged; `list_document`/`list_markdown` still private.
- [x] 5.5 Sanity-read `AGENTS.self-law.md`: preamble makes no crate-specific claim; every crate-specific claim sits in the generated projection with its boundary's `reason`.
- [x] 5.6 Review checkpoint (Contract B): `constitution_markdown`'s doc-comment carries the evolvability + "use JSON for a machine contract" clause, and no golden/snapshot test pins the helper's exact output — this is the only thing holding Contract B, so it is a named review gate, not an automated one.
- [x] 5.7 Confirm `tasks` add no `Cargo.toml` version change — the 0.1.1 bump lands in the `release: 0.1.1` commit, not in this change (consistent with the two prior 0.1.1 changes).
