## 1. Reproduce each defect before repairing it

- [ ] 1.1 A fixture whose tracked `.gitignore` ignores a file git prints quoted: observe the gate refuse it,
  and record the quoted spelling and `check-ignore`'s exit 1 for that literal.
- [ ] 1.2 Observe `check-ignore`'s failure read as an empty classification.
- [ ] 1.3 Observe the package enumeration shorten rather than refuse.

## 2. The publish gate reads what it asks about

- [ ] 2.1 `ls-files -z`, `status -z`, `check-ignore -z -v --stdin` throughout `hidden_by_the_checkout`.
- [ ] 2.2 A classification that could not be produced is a cannot-judge naming the unclassified paths.
- [ ] 2.3 The fixture from 1.1 is now accepted, and a file hidden by this clone's own exclude is still
  refused — both, so the repair is not a widening.

## 3. The package enumeration reads tracked content

- [ ] 3.1 `git ls-files -- 'crates/*/Cargo.toml'`, refusing on a non-zero status.
- [ ] 3.2 A direction for the failed enumeration, and one for an untracked directory carrying a manifest.

## 4. The anchor and the last dropped entries

- [ ] 4.1 `audit_corpus_and_anchor`'s innermost fallback returns `Err`; the synthetic-metadata fallback stays.
- [ ] 4.2 `baseline_cli`'s sibling-file direction stops dropping the entries it counts.
- [ ] 4.3 The `filter_map(entry.ok())` class is empty across the tree.

## 5. Record and land

- [ ] 5.1 `refusal_bites`: new sites defended; census figures updated where declared.
- [ ] 5.2 `CHANGELOG.md` under `[Unreleased]`. No version bump.
- [ ] 5.3 Full Definition of Done.
- [ ] 5.4 Sync both deltas, archive, and land as one squash PR through `scripts/merge-pr.sh`.
