## 1. The refusal names what was supplied

- [ ] 1.1 In `dispatch_list`, replace the `||` over five conditions with a collected list of the flags actually
      supplied, ordered by the check rather than by the command line (design D2).
- [ ] 1.2 The message names them and keeps the existing shape of the sentence around them, so the diagnostic reads
      as an extension of what was there rather than as a new format.
- [ ] 1.3 No exit code moves, and the `usage:` banner is untouched.

## 2. The guard, per flag

- [ ] 2.1 In `crates/tianheng/tests/baseline_cli.rs`, drive each of the five check-only flags individually and
      assert the message names it. One test asserting "some flag is named" would pass while four went unnamed
      (design D4).
- [ ] 2.2 One case supplying two of them at once, asserting both are named.
- [ ] 2.3 Observe each failing against the code **without** the change: the message before this is one sentence
      naming none, so all five and the pair fail. Record the message.
- [ ] 2.4 Confirm `list --format sarif` still gets its own precise refusal and is not swept into the new one — it
      is a *value* refusal, and the two must stay distinguishable.

## 3. Coherence

- [ ] 3.1 `CHANGELOG.md`: a **Fixed** entry under `[Unreleased]`, recording it as a diagnostic correction. No
      invocation's verdict moves, so it is not a behaviour change.
- [ ] 3.2 `BACKLOG.md`: close the sweep entry with what the enumeration measured — the twelve write-path operations
      and their tests, the five adversarial shapes probed live, the twenty-five CLI cells — the two defects found,
      and the honest note that the enumeration is a hand-made snapshot with a trigger for making it live.
- [ ] 3.3 Sweep for prose this invalidates: the `check`-internal conflict requirement's claim to be extending "the
      same rule" is now true, so nothing there needs changing — confirm that by reading it rather than assuming.

## 4. Verification

- [ ] 4.1 Every observation from task 2 in the pull request's `## Verification`.
- [ ] 4.2 Full Definition of Done, then again from a clean clone.
- [ ] 4.3 `openspec validate` strict before and after sync.

## 5. Sync

- [ ] 5.1 Archive, then prune the dated directory — only `archive/.gitkeep` is tracked.
- [ ] 5.2 Confirm the modified requirement landed with all three scenarios, and that the bound register's figures
      do not move (this change declares no bound).
