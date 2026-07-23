# Tasks: Docs Denoise History and Specs

## Tasks

- [x] 1. **Extract Backlog History**: Create `docs/history/0.3.0-built-ledger.md` and move historical `BUILT / HISTORY` entries from `BACKLOG.md`.
- [x] 2. **Prune BACKLOG.md**: Retain only Backlog Governance rules, Live Decision Index, and active `READY-PATCH` / `DESIGN-BREAKING` / `WATCH` / `ACCEPTED DEBT` / `DECLINED` items in `BACKLOG.md`.
- [x] 3. **Sweep OpenSpec Specs**: Audit prose in `openspec/specs/*` for 0.3.0 terminology alignment (`RuleKey`, `StructuredFactIdentity`).
- [x] 4. **Sweep Code Docs**: Audit module and item rustdoc comments in `crates/*/src/` for 0.3.0 consistency.
- [x] 5. **Update Release Coherence**: Add adopter-facing maintenance entry under `[Unreleased]` in `CHANGELOG.md`.
- [x] 6. **Run Pre-flight DoD Gates**: Execute workspace build, 3 clippy passes, rustfmt, workspace tests (`TIANHENG_WORKSPACE_TESTS=1`), rustdoc check, cargo deny, release coherence scripts, and example suite script.
