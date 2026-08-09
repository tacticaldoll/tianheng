## 1. Record what ships today

- [ ] 1.1 `cargo package --list -p tianheng` — the file count under `tests/` and the target names, before
  anything moves. This is the figure the change is measured against.
- [ ] 1.2 Record the pre-move census figures the moved reactions produce — `refusal_bites`' site counts, the
  bound register's totals, the projection register's rows — so the post-move run is compared against a
  recorded observation rather than a memory.

## 2. The member exists and is empty (N1)

- [ ] 2.1 `crates/shengmo/Cargo.toml`: `publish = false`, `license.workspace = true`, no license texts, and a
  dev-dependency on `tianheng` for the law. Add it to the workspace's explicit `members` list.
- [ ] 2.2 `crates/shengmo/src/lib.rs`: documentation and no code — what 繩墨 is, why it ships in no package,
  and that its tests are the repository's law and the reactions holding it.
- [ ] 2.3 `cargo build --workspace` and `cargo test -p shengmo` pass with no targets yet. Confirm the
  repository's own constitution does not react to a doc-only crate; if it does, that reaction is the design
  question to settle before any target moves.

## 3. The law moves first, alone (N2)

- [ ] 3.1 Move `self_governance.rs` and confirm `AGENTS.self-law.md` still regenerates from it identically —
  the projection is a record of the law, and a move that changes it changed the law.
- [ ] 3.2 Follow its invocations: `.github/CODEOWNERS`, `docs/projection-register.md`, `README.md`, `deny.toml`,
  `AGENTS.md`. Regenerate the projection register rather than hand-editing it.
- [ ] 3.3 The law alone, moved and green, before anything else moves. If this step is wrong, everything after
  it is wrong for the same reason.

## 4. The reactions follow, with their support modules (N3)

- [ ] 4.1 Move the nine support modules whose only users move: `bound_register_parse`, `census`,
  `merge_message_gate`, `publish_source_gate`, `refusal`, `refusal_exemptions`, `refusal_sites`, `region`,
  `release_coherence_gate`. `support/mod.rs` stays and drops `pub mod region;`.
- [ ] 4.2 Move the remaining 16 targets and their fixtures. Keep `#[path = "support/…"]` inclusion exactly as
  it is — the inclusion mechanism does not change in the same step as the location.
- [ ] 4.3 `refusal_sites` builds its corpus from dep-info: confirm it enumerates shengmo's targets and that
  `refusal_bites` reports the same site classification as 1.2 recorded. A different census here is a defect in
  the move, not a new fact about the tree.
- [ ] 4.4 `crates/tianheng/tests/` holds only the eight crate tests. `cargo test -p tianheng` passes without
  any workspace marker, and `cargo package --list -p tianheng` no longer carries a governance target.

## 5. Every invocation follows (N4)

- [ ] 5.1 `.github/workflows/ci.yml` — six jobs name moved targets; `packaged-selftest` and `license-files`
  iterate crate lists that gain a member.
- [ ] 5.2 `AGENTS.md`'s Definition of Done, `scripts/publish.sh`, `scripts/merge-pr.sh`.
- [ ] 5.3 `dod_coherence` and `reference_integrity` run **last**: they hold the DoD-to-CI correspondence and
  every path reference, so they are the reaction that catches a missed invocation. Observe each red against a
  deliberately unmigrated invocation first.

## 6. Record and land

- [ ] 6.1 `CHANGELOG.md` under `[Unreleased]`, `### Self-governance`: the tarball no longer carries 16 test
  targets an adopter could not run. No version bump.
- [ ] 6.2 Full Definition of Done, including every gated suite from its new package, plus
  `cargo package --list` for each published crate.
- [ ] 6.3 Sync the two deltas, archive the change, and land the branch as one squash PR through
  `scripts/merge-pr.sh`.
- [ ] 6.4 Rebuild `declare-what-each-capability-governs` on the new tree: its subject declarations name
  `crates/shengmo/**`, and its proposal join is what keeps a new reaction from landing back inside the
  package.
