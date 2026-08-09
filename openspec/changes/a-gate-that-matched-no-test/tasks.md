## 1. See the defect

- [x] 1.1 Record the three measurements the design rests on: `--exact <unknown>` exits 0 with 0 passed;
  `--exact <ignored>` exits 0 with `0 passed; 1 ignored`; `--list` includes the ignored test. They decide the
  division of labour and must be measured, not reasoned.
- [x] 1.2 With a wrapper's `--exact` argument temporarily pointed at a name no test carries, observe the
  script reach its irreversible command (stop before any network act). Revert with the Edit tool.

## 2. The wrapper assertion

- [x] 2.1 `scripts/publish.sh`: capture the gate run's combined output, require exactly one passing test, and
  otherwise print what it saw and exit 1. Keep the `env -u …` scrub and the invocation on one logical line.
- [x] 2.2 Same in `scripts/merge-pr.sh`, keeping its `TIANHENG_MERGE_*` environment.
- [x] 2.3 Re-run 1.2 against both patched scripts: the unknown name must now stop before the irreversible
  command. Revert the temporary edits with the Edit tool.
- [x] 2.4 Observe the passing direction too — a wrapper that always refuses is not a working one.

## 3. The pinning reaction

- [x] 3.1 `crates/kanhe/src/gate_identity.rs`: join physical lines ending in `\`; extract every
  `--exact <ident>` with that logical line's `--test <target>` and `-p <pkg>`; return the shared kinded
  refusal for each failure direction.
- [x] 3.2 Resolve each identifier through `cargo test -p <pkg> --test <target> -- --list`, taking the last
  `::` segment. Exactly one match required. Do not map the target to a source path.
- [x] 3.3 Each direction with its own message: registered zero times (violation); more than once (violation);
  `--exact` with no `--test` in its logical line (cannot-judge); script enumeration fails (cannot-judge);
  the `--list` run fails (cannot-judge).
- [x] 3.4 `crates/kanhe/tests/gate_identity.rs`: the reaction over `git ls-files scripts/`, plus the failure
  matrix over fixture script text.
- [x] 3.5 See it red before green: temporarily rename one cited test and observe the reaction name the script,
  the identifier, and the target. Revert with the Edit tool.

## 4. The new sites join the sweep, and the change lands

- [x] 4.1 `refusal_bites`: every new construction site defended, and the census figure updated where it is
  declared.
- [x] 4.2 No line needed: the reaction runs in `cargo test --workspace`, which the Definition of Done and CI
  already carry. Named here rather than left implicit, because a gated reaction absent from both is the shape
  this family refuses.
- [x] 4.3 `CHANGELOG.md` under `[Unreleased]`. No version bump.
- [ ] 4.4 Full Definition of Done, including every gated suite and `openspec validate --specs --strict`.
- [ ] 4.5 Sync both deltas, archive the change, and land the branch as one squash PR through
  `scripts/merge-pr.sh` — which now asserts its own gate ran.
