## 1. The reaction

- [ ] 1.1 New module `crates/tianheng/tests/projection_register.rs`, following the `TIANHENG_WORKSPACE_TESTS`
      discipline as a pure function of (root, marker-set) so the loud-in-CI direction can be observed failing
      without a test mutating the process environment.
- [ ] 1.2 Enumerate tracked Markdown from `git ls-files`, `-z`, and select those containing the marker. Fail
      loudly on an empty enumeration, naming the emptiness.
- [ ] 1.3 Parse each document's named generator: the first repository-relative path in its header pointing at a
      tracked file. Verify against all four before trusting the parse — the four headers were written at four
      different times and only the marker is uniform (`AGENTS.self-law.md` puts its command on a following line;
      the other three put it after an em dash).
- [ ] 1.4 Enumerate the holders independently, both mechanisms: Rust call sites of the shared rule, and shell
      scripts writing a projection under `BLESS`. `crates/tianheng/src/testing.rs` defines the rule and holds
      nothing — confirm it is not counted as a holder, and that excluding it is by *defining the rule* rather
      than by name.
- [ ] 1.5 The bijection, both directions, each failure naming the document or the unit and what is missing.

## 2. Reachability and self-inclusion

- [ ] 2.1 Assert each document's path appears in `AGENTS.md`, with comment-only mentions cut before the search —
      the defect the previous change made once (design D6).
- [ ] 2.2 Assert the register's own projection is in its own table, with its own generator named (design D5).
- [ ] 2.3 Expect to bless **twice** on first creation: the first blessing changes the set the second measures.
      Record that it converges on the second run rather than treating the first mismatch as a defect (design D5).

## 3. The bounds

- [ ] 3.1 `a_regeneration_command_is_registered_and_never_run` — demonstrate rather than assert: a fixture
      document whose header names a command that cannot regenerate anything passes every property.
- [ ] 3.2 `a_third_generation_mechanism_is_not_recognized` — same form: a fixture holder using neither mechanism,
      with no marker on its document, leaves the bijection holding.
- [ ] 3.3 Typed declarations in `tianheng::observation_bounds()`: `OutOfReach` for the command, `UnderReacts`
      owned by `Owner::Engine` for the third mechanism (design D4 — recording the second as out-of-reach would be
      the misclassification the bound model exists to prevent).
- [ ] 3.4 Confirm both resolve under `bash scripts/check_bound_register.sh` after sync, and that the projection's
      leading figure still reads 0 unpinned. The declared-false-negative count in
      `docs/observation-bound-extents.md` moves up by one; take the figure from the run.

## 4. The projection

- [ ] 4.1 Emit it through `assert_projection_matches`, never a new mechanism.
- [ ] 4.2 Header states what registration does not mean: not freshness (each holder asserts that), and not that a
      stated command works.
- [ ] 4.3 Print the table; write no count into prose anywhere.
- [ ] 4.4 Observe the staleness direction.

## 5. Verification

- [ ] 5.1 Every direction observed failing first, and each recorded in the PR's `## Verification`: a marked
      document whose generator is absent; a holder no document names; a document `AGENTS.md` does not name; a
      commented-only mention **not** counting; an empty enumeration; the absent layout under the marker; the
      projection hand-edited.
- [ ] 5.2 Perturb one recognizer to always hold and confirm a test fails — the check that caught a
      construction-passing test twice in this window.
- [ ] 5.3 Full Definition of Done, then again from a clean clone.

## 6. Coherence

- [ ] 6.1 `AGENTS.md` — the *Self-governance* paragraph gains the new projection. State in the commit that this is
      the last time that edit is unchecked, which is the point of the change.
- [ ] 6.2 No `CHANGELOG.md` entry: nothing an adopter does touches this capability. Its bound census moves by two;
      correct it from the register's run, never by recounting.
- [ ] 6.3 `PROJECT.md`'s audit-cycle decision gains the fourth instance, and the first over the repository's own
      *documents* rather than its specs or its gates.
- [ ] 6.4 `BACKLOG.md` — record the residual D4 declares, and sweep for prose this change invalidates in the same
      window.
