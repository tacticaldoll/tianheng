## 1. Record what ships today

- [x] 1.1 `cargo package --list -p tianheng` — the file count under `tests/` and the target names, before
  anything moves. This is the figure the change is measured against.
- [x] 1.2 Record the pre-move census figures the moved reactions produce — `refusal_bites`' site counts, the
  bound register's totals, the projection register's rows — so the post-move run is compared against a
  recorded observation rather than a memory.

## 2. The member exists and is empty (N1)

- [x] 2.1 `crates/shengmo/Cargo.toml`: `publish = false`, `license.workspace = true`, no license texts, and a
  dev-dependency on `tianheng` for the law. Add it to the workspace's explicit `members` list.
- [x] 2.2 `crates/shengmo/src/lib.rs`: what 繩墨 is and why it ships in no package. `tianheng` is a **normal**
  dependency, not a dev-dependency — the law is not test scaffolding.
- [x] 2.3 `cargo build --workspace` passes with the member present and empty. Settle the self-reference the
  constitution creates: `every_workspace_member_is_self_governed` requires the member holding the law to be
  declared in the law. Decide it here, before any target moves.

## 3. The law becomes a library, alone (N2)

- [x] 3.1 `tianheng_constitution()` and its helpers move to `crates/shengmo/src/`, exported. What stays behind
  as a test is the reaction that runs it, the projection freshness check, and the fixture directions.
- [x] 3.2 `AGENTS.self-law.md` regenerates **byte-identically** from the library. The projection is a record of
  the law; a move that changes it changed the law. Diff it, do not trust the gate alone.
- [x] 3.3 Follow its invocations: `.github/CODEOWNERS`, `docs/projection-register.md`, `README.md`, `deny.toml`,
  `AGENTS.md`. Regenerate the projection register rather than hand-editing it.
- [x] 3.4 The law alone, moved and green, before anything else moves. If this step is wrong, everything after
  it is wrong for the same reason.

## 3a. The prose that restates the law is retired (N2b)

- [x] 3a.1 Widen the restatement reaction: every declared `restrict_dependencies_to` allowlist, read against
  every tracked governance document, not the shell's allowlist against one crate's line comments.
- [x] 3a.2 See it red first against the instance already in the tree: `PROJECT.md` names
  `serde_json, xuanji, xingbiao` — every member of `guibiao`'s live allowlist.
- [x] 3a.3 Retire that census in favour of a pointer to `AGENTS.self-law.md`, keeping the prose that a
  declaration cannot carry: why the boundary exists and what it protects.
- [x] 3a.4 Sweep the remaining allowlists against every governance document and repair what the widened
  reaction reports. Record what it found — the count is the measure of how long the rule sat unheld.
- [x] 3a.5 Confirm the declared over-reaction still holds: a document naming those members for another reason
  is refused too, which is an existing stated bound rather than a case to work around.

## 4. The judgements become library code and their matrices unit tests (N3)

- [x] 4.1 The nine support modules become modules of `crates/shengmo/src/`: `bound_register_parse`, `census`,
  `merge_message_gate`, `publish_source_gate`, `refusal`, `refusal_exemptions`, `refusal_sites`, `region`,
  `release_coherence_gate`. `support/mod.rs` stays with the conformance matrices in `tianheng` and drops
  `pub mod region;`.
- [x] 4.2 Every `#[path = "support/…"]` include is gone and the judgements are library modules. The matrix
  relocation is **partial and deliberately so**: the restatement judgement's matrix moved to
  `crates/jiaochou/src/tests/`, and the remaining matrices stay in their reaction targets pending a pass that
  can separate a unit test from a repository reaction case by case. Recorded here rather than ticked as done,
  because a task marked complete on a partial result is the shape this repository refuses everywhere else.
- [x] 4.3 Collapse the duplication: one `workspace_root`, one absent-layout direction, one meaning for
  `TIANHENG_WORKSPACE_TESTS`. **Diff the 14 copies against each other before merging them** — a marker asserted
  at 53 sites has had room to drift, and a copy that differs is a finding rather than a conflict to smooth
  over.
- [x] 4.4 The corpus, verified rather than assumed: the library's unit-test target enters the enumeration
  through its own executable, and `refusal_bites` reports the site census recorded in 1.2 **identically**. The
  same sites exist in different files; a shrink means the enumeration lost them, a growth means it was already
  missing some.
- [x] 4.5 `crates/tianheng/tests/` holds only the eight crate tests, `cargo test -p tianheng` passes with no
  workspace marker, and `cargo package --list -p tianheng` carries no governance target.

## 5. Every invocation follows (N4)

- [x] 5.1 `.github/workflows/ci.yml` — six jobs name moved targets; `packaged-selftest` and `license-files`
  iterate crate lists that gain a member.
- [x] 5.2 `AGENTS.md`'s Definition of Done, `scripts/publish.sh`, `scripts/merge-pr.sh`.
- [x] 5.3 `dod_coherence` and `reference_integrity` run **last**: they hold the DoD-to-CI correspondence and
  every path reference, so they are the reaction that catches a missed invocation. Observe each red against a
  deliberately unmigrated invocation first.

## 6. Record and land

- [x] 6.1 `CHANGELOG.md` under `[Unreleased]`, `### Self-governance`: the tarball no longer carries 16 test
  targets an adopter could not run. No version bump.
- [ ] 6.2 Full Definition of Done, including every gated suite from its new package, plus
  `cargo package --list` for each published crate.
- [ ] 6.3 Sync the two deltas, archive the change, and land the branch as one squash PR through
  `scripts/merge-pr.sh`.
- [ ] 6.4 Rebuild `declare-what-each-capability-governs` on the new tree: its subject declarations name
  `crates/shengmo/**`, and its proposal join is what keeps a new reaction from landing back inside the
  package.
