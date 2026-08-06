## 1. The tenth property

- [ ] 1.1 Add `backstop_label_of` to `crates/tianheng/tests/gate_shape_contract.rs`: the first argument of the
      `exit_contract_backstop` invocation, trimmed, with one surrounding pair of `'` or `"` removed. A gate with
      no invocation has no label and fails both properties — two offences, each a real absence, the precedent the
      absent twin already set (design D2a).
- [ ] 1.2 Add `name_from_basename`: `check_` and `.sh` removed, `_` read as space. One function, used by the
      recognizer and named in the failure — never a kept table (design D1).
- [ ] 1.3 Add the entry to `PROPERTIES`, subject `Gate`, immediately after the backstop entry — beside the
      property it refines, since both are read off the same line. (Not "in the gate's text order": the header
      comes first in every gate, so that justification was wrong.)
- [ ] 1.4 The remedy names **both** labels — the one written and the one the basename asks for (design D3).
      Verify the message on a real failure rather than by reading the format string.
- [ ] 1.5 Confirm the per-property fixture test covers it with no change to that test: it iterates `PROPERTIES`,
      so the tenth must enter the fixture builder's withholding by label. If it needs a change, the array is
      not yet the single declaration site it claims to be.

## 2. Observed failing, in both directions

- [ ] 2.1 The wrong label: perturb one gate's invocation to a sibling's name and confirm the reaction fails
      naming the gate, the written label and the derived one. This is the copy-paste shape itself.
- [ ] 2.2 The unreadable label: perturb one gate to `exit_contract_backstop "$(basename "$0" .sh)"` and confirm
      the reaction refuses **saying the label is not a literal** — not reporting a mismatch against a label the
      gate never wrote (design D2). Deliberately the *better* implementation as the fixture, so the message is
      read in the case where it is most tempting to be wrong.
- [ ] 2.3 The fixture direction: withhold the property in the per-property test's fixture and confirm exactly one
      offence, against the gate, named by this property — and exactly two when the backstop itself is withheld.
- [ ] 2.4 Confirm the recognizer discriminates by forcing it to hold and watching the per-property test fail,
      the check that caught a construction-passing test in the change that added the nine.

## 3. The projection

- [ ] 3.1 Re-bless `docs/gate-shape-contract.md`; the table gains a column and the printed figure moves from
      nine to ten by measurement, not by edit.
- [ ] 3.2 Observe the staleness direction once more, since the row shape changed.

## 4. The counts leave prose (design D4)

- [ ] 4.1 `crates/tianheng/tests/gate_shape_contract.rs`: the array's own doc comment says "three per gate, five
      per twin, one over `AGENTS.md`" directly above the array that answers it. Remove the census, keep the
      structure it was trying to convey.
- [ ] 4.2 The same file's other mentions — "the nine properties", "nine reasons", "judges on none of the nine
      properties". Reword to name the set, not its size.
- [ ] 4.3 `AGENTS.md`: "holds each to the nine properties of the family's exit contract" → point at the
      projection.
- [ ] 4.4 `BACKLOG.md`'s closed entry: "asserts the nine checkable properties" → same.
- [ ] 4.5 Grep the whole tree for `nine` and for `9 properties` afterwards, including the two documents this
      change does not otherwise touch. A count that survives in one place is the drift this task exists to end.

## 5. Verification

- [ ] 5.1 Every observation from task 2 recorded in the pull request's `## Verification`, with the message and
      the exit status.
- [ ] 5.2 Full Definition of Done, then again from a clean clone.
- [ ] 5.3 `openspec validate backstop-label-names-its-gate --strict` before sync; `openspec validate --specs
      --strict` after.

## 6. Sync

- [ ] 6.1 `openspec archive`, then prune the dated archive directory — only `archive/.gitkeep` is tracked.
- [ ] 6.2 Confirm the modified requirement landed in `openspec/specs/gate-shape-contract/spec.md` with all four
      scenarios, and that the register still reads 0 unpinned (this change declares no bound, so the figure must
      not move).
