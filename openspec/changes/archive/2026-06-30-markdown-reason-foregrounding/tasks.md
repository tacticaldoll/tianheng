## 1. Renderer: foreground the reason (shape B2)

- [x] 1.1 In `crates/tianheng/src/runner.rs`, rewrite `boundary_markdown()` to the B2 order: heading = target; if `reason` is non-empty, emit it as a leading `> <reason>` blockquote followed by a blank line; then `- **rule**: …`; then `- **kind**: … · **severity**: …` with `· **crate**: …` appended only when the boundary carries a `crate` key (module boundaries only — crate boundaries have none). Keep the existing `if !reason.is_empty()` guard: a reason-less boundary emits no blockquote **and no orphan blank line**.
- [x] 1.2 Confirm `list_document` / `list_markdown` / `constitution_markdown` are structurally unchanged (only the per-boundary block layout moved); CLI dispatch, `list`/`check`, JSON, and exit codes untouched.

## 2. Foregrounding invariant test (NOT a format freeze)

- [x] 2.1 Add a unit test in `runner.rs` rendering a boundary with a known reason and asserting byte-index order: `idx(reason) < idx(rule) < idx(kind)` within that boundary's block. Assert ordering only — never `assert_eq!` against a literal Markdown blob. Add a sibling case: a reason-less boundary renders no blockquote and no orphan blank line (heading immediately followed by the rule bullet).
- [x] 2.2 Confirm the existing `list_markdown_covers_every_dimension_with_target_rule_and_reason` (uses `.contains()`, order-agnostic) and `constitution_markdown_equals_the_cli_projection_byte_for_byte` (same renderer both sides) still pass.

## 3. Docs (Contract B reaffirmed)

- [x] 3.1 Update doc-comments on `boundary_markdown` (and as needed `list_markdown` / `constitution_markdown`) to state the Markdown foregrounds the declared reason because it is the agent's repair/imitation hint, and to reaffirm Contract B (layout may evolve; JSON is the machine contract).
- [x] 3.2 (D — ride-along) Add a one-line adopter recipe near the README's "published binary is a demo" note (or crate-level docs): `let md = tianheng::constitution_markdown(&constitution()); std::fs::write("AGENTS.<project>-law.md", md)?;`. Library primitive only — no generator, no CLI command.

## 4. Regenerate the self-law artifact

- [x] 4.1 `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh` to regenerate `AGENTS.self-law.md` into the reason-foregrounded shape; commit the regenerated artifact. (The staleness test failing before regeneration is the expected, correct ripple.)

## 5. Verify (Definition of Done)

- [x] 5.1 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --all-features` passes (foregrounding test + freshness + byte-exact helper + governance).
- [x] 5.2 `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all --check` clean.
- [x] 5.3 `cargo doc --workspace --no-deps --all-features` clean.
- [x] 5.4 Sanity-read regenerated `AGENTS.self-law.md`: each boundary leads with its reason (blockquote), then rule, then kind/severity; target still the heading.
- [x] 5.5 Confirm no byte/golden snapshot test of the Markdown exists (only the ordering invariant + the helper-vs-CLI equality) — Contract B intact.
- [x] 5.6 Confirm no JSON/text-projection change, no reaction/law change, no `Cargo.toml` version change (bump deferred to the release commit).
