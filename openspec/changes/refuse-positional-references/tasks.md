## 1. The reaction

- [ ] 1.1 In `crates/kanhe/tests/reference_integrity.rs`, refuse a backticked `<tracked-path>:<line>` in every
      tracked format, with the left side required to be a path the existing enumeration produces.
- [ ] 1.2 Keep the vacuity guard on the **corpus**, not on the match count, so an empty result means "nothing
      matched" rather than "nothing was read".

## 2. The negative run

- [ ] 2.1 Run the direction **before** repairing the two citations and record it naming both verbatim.
- [ ] 2.2 Confirm a positional *phrase* in Markdown is not matched, and that a non-path `word:number` is not.

## 3. The repairs

- [ ] 3.1 `BACKLOG.md` — replace both coordinates with the entries' own names.
- [ ] 3.2 `CHANGELOG.md` — narrow *tracked content* to *tracked source* and say what that scope left.

## 4. Record and lifecycle

- [ ] 4.1 `CHANGELOG.md` entry under `### Self-governance`.
- [ ] 4.2 Confirm the workspace version does not move.
- [ ] 4.3 Full Definition of Done.
- [ ] 4.4 Sync the delta, prune the dated archive copy, open the pull request, squash-merge through the wrapper.
