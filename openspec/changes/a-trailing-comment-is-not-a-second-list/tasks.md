# Tasks

- [x] 1 The delegation recognizer strips a trailing comment before comparing, following the convention already in
      `gate_shape_contract.rs`. **Verified**: the comment made the reaction report an offence before, and is
      accepted after.
- [x] 2 `fn bounds(` is located by line position. **Verified**: with the old locator, a doc comment reading
      ``See `fn bounds(` below {`` fails the reaction; with the new one it passes.
- [x] 3 The spec states both rules, so a reader is not left to infer them from the code.
- [x] 4 The three file counts become the claim without the number, and the changelog's ordinal becomes the fact.
- [ ] 5 Sync the delta and prune the dated archive copy.
- [x] 6 Full Definition of Done clean; no version bump.
