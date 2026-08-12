## 1. The example list

- [ ] 1.1 Hold `EXAMPLES` against tracked `examples/` directories, both directions.
- [ ] 1.2 Negative runs: an undeclared tracked example fails; a declared entry with no directory fails.

## 2. The type list

- [ ] 2.1 Parse the backticked type run from `AGENTS.md`'s *narrowest honest type* clause and hold it against
      `TYPES`, both directions; refuse loudly if the anchor is absent.
- [ ] 2.2 Negative runs: a type in one side and not the other fails, naming which side.

## 3. The arrival matrix

- [ ] 3.1 Enumerate the flags `scripts/publish.sh` forwards and assert the arrival matrix covers all of them,
      each with a legal value; fail loud on an unparseable arm.
- [ ] 3.2 Negative run: dropping a flag from the matrix fails.

## 4. Record and lifecycle

- [ ] 4.1 `CHANGELOG.md` under `### Self-governance`; confirm no version surface moves.
- [ ] 4.2 Full Definition of Done; sync both deltas; prune the dated copy; PR; squash-merge through the wrapper.
