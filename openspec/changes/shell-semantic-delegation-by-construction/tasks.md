## 1. Make the delegation structural

- [x] 1.1 Route the shell's semantic arm through `SemanticObserver::new(...).observe(...)`.
- [x] 1.2 Confirm no verdict moves: the full workspace suite passes with the arm swapped and nothing else
  changed.

## 2. Say which dimensions are constructed and which are measured

- [x] 2.1 Move semantic into the construction-held list in the requirement, naming static as the one still
  independently implemented on both sides.
- [x] 2.2 State the obligation that keeps a construction-held dimension honest: the reaction still observes that
  the fixture's boundary for it reacts at all.
- [x] 2.3 Update the reaction's module doc to match, and de-number its "two properties" claim.

## 3. Retire the bound

- [x] 3.1 Remove the bound scenario from `observer-protocol`.
- [x] 3.2 Remove its typed declaration from `crates/tianheng/src/bounds.rs`.
- [x] 3.3 Regenerate both projections; confirm the register's census reaction catches the figures it makes
  stale.
- [x] 3.4 Close the `BACKLOG.md` READY-PATCH entry, keeping the record the disposition rests on.

## 4. Definition of Done

- [x] 4.1 Full `AGENTS.md` Definition of Done in order.
- [ ] 4.2 Sync the delta into `openspec/specs/observer-protocol/spec.md` and prune the change directory.
