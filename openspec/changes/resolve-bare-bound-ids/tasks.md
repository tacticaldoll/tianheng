## 1. The recognizer

- [ ] 1.1 In `crates/kanhe/src/bound_register_parse.rs`, add the bare-reference recognizer: maximal runs of
      path characters, kept when the run is exactly `<capability>/<slug>` with the capability drawn from an
      **enumerated** set and the slug kebab-case.
- [ ] 1.2 Resolve into the **same** produced id set the `(bound: …)` form uses, never a second derivation, so
      the two forms cannot disagree about what a valid id is.

## 2. The failure matrix

- [ ] 2.1 Rows in `crates/kanhe/src/tests/bound_register_parse.rs`: a bare id naming a declared bound resolves;
      one naming nothing is refused; a path containing a capability name is **not** a reference; a capability
      absent from the enumerated set yields no reference.

## 3. The repository direction

- [ ] 3.1 In `crates/kanhe/tests/bound_register.rs`, resolve every bare reference across tracked Rust and
      Markdown, failing with file, line and unresolved id.
- [ ] 3.2 Keep the corpus enumeration on tracked content (`git ls-files`), not the worktree.

## 4. The stale citations this makes observable

- [ ] 4.1 `crates/kanhe/tests/capability_subjects.rs` — correct to
      `repository-checks/files-no-capability-claims-a-stated-bound`.
- [ ] 4.2 `crates/xuanji/src/tests.rs`, both occurrences — correct to
      `external-crate-confinement/a-confined-crate-use-inside-a-string-or-macro-body-is-not-observed-a-stated-bound`.
- [ ] 4.3 `crates/kanhe/tests/observation_bound_model.rs` — the doc comment says *the family's four sets* over a
      chain of five. Remove the number; state the property instead.

## 5. The negative run

- [ ] 5.1 Before correcting them, run the new direction and record it naming all three verbatim. That is the
      evidence the direction is wired to the tree, and these are real defects rather than a planted probe.

## 6. Record and lifecycle

- [ ] 6.1 `CHANGELOG.md` entry under `### Self-governance`.
- [ ] 6.2 Confirm the workspace version does not move.
- [ ] 6.3 Full Definition of Done, including the env-gated lines.
- [ ] 6.4 Sync the delta into `openspec/specs/observation-bound-register/spec.md`, prune the dated archive copy.
- [ ] 6.5 Open the pull request into `release/0.5.0` and squash-merge through `bash scripts/merge-pr.sh`.
