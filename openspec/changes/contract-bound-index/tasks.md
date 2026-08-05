## 1. Measure the scope before writing anything

- [x] 1.1 Enumerate every bound-declaring occurrence in `openspec/specs/*`, recording file, requirement,
      and the sentence, so the register's size is measured rather than assumed (the 43-occurrence figure
      is a grep count, not a count of distinct bounds)
- [x] 1.2 For each occurrence, identify whether a test pins it and by what name, recording the ones with
      no pinning test separately — that list is the register's opening unpinned set and the audit backlog
- [x] 1.3 Settle the prose pattern the floor scans for, derived from the occurrences actually found rather
      than guessed, and record which occurrences it matches and which it would miss
- [x] 1.4 Resolve the design's first open question against the enumerated set: whether a bound shared by
      two capabilities is registered once or twice

## 2. Build the reaction before registering anything

- [x] 2.1 Write `scripts/check_bound_register.sh`: parse `#### Bound:` blocks, require `statement` plus
      exactly one of `pinned-by` / `unpinned`, and exit `0` clean, `1` violation, `2` cannot judge
- [x] 2.2 Add the pinning-test resolution rule: exactly one `fn <name>(` definition under `crates/`, with
      zero and two both failing, and matching on the definition form so a mention cannot satisfy it
- [x] 2.3 Add the unregistered-prose floor over `openspec/specs/*`, counting only occurrences outside a
      `#### Bound:` block
- [x] 2.4 Add the vacuity guard: a run that parsed zero register blocks, or scanned zero spec files, fails
      `2` rather than reporting clean
- [x] 2.8 Add the reference form `(bound: <capability>/<slug>)`: a referencing prose line is cleared, and
      a reference resolving to zero or to more than one declared bound fails — the second direction being
      what checks the derived id's uniqueness
- [x] 2.9 Confirm a reference contributes nothing to the bound count and carries no citation of its own
- [x] 2.5 Write `scripts/test_bound_register.sh` proving every failure direction on its own fixture —
      missing test, duplicate test, mention-only citation, both elements, neither element, unregistered
      prose, untracked `unpinned`, dangling reference, ambiguous reference, stale projection — and the passing direction, asserting the exact exit
      code rather than merely non-zero
- [x] 2.6 Prove the gate is read-only: tree, `HEAD`, and projection unchanged after a run
- [x] 2.7 Run `test_bound_register.sh` against a stubbed always-passing gate and record the observed
      failure, so the companion test is shown to be a guard rather than a restatement

## 3. Register the bounds

- [ ] 3.1 Add a citation to every declared bound the gate reports without one, citing its pinning test
      where one exists
- [ ] 3.5 Triage the 22 prose occurrences the gate reports: declare each real bound as a bound-marked
      scenario, and give each genuine cross-reference a resolving `(bound: …)` reference — deciding per
      occurrence rather than applying one rule to all of them
- [ ] 3.2 Register each unpinned bound from 1.2 with its tracker reference, and open the `BACKLOG.md`
      entries those references point at
- [ ] 3.3 Register the register's own detection residual — that the floor covers recognizable wording
      only — as a bound of `observation-bound-register`, with its own pinning test
- [ ] 3.4 Confirm `openspec validate --specs --strict` still passes over every touched spec

## 4. Project the register

- [ ] 4.1 Generate `docs/observation-bounds.md` from the specs, grouped by capability, with the unpinned
      count as its headline figure and the floor's residual stated in its header
- [ ] 4.2 Add the staleness direction to the gate: a projection that no longer matches the specs fails,
      with an explicit regeneration path
- [ ] 4.3 Prove the staleness direction fails on a hand-edited projection and passes after regeneration

## 5. Wire it in

- [ ] 5.1 Add the gate and its companion test to `AGENTS.md`'s Definition of Done, and the identical lines
      to `.github/workflows/ci.yml`
- [ ] 5.2 Point `AGENTS.md` at the projection where it explains reading the law, so an auditor and an
      agent find the register without knowing it exists
- [ ] 5.3 Confirm `check_dod_coherence.sh`, `check_reference_integrity.sh`, and
      `check_whitespace_hygiene.sh` all pass over the new files
- [ ] 5.4 Add the `[Unreleased]` `CHANGELOG.md` entry, stating that no adopter action follows

## 6. Close the change

- [ ] 6.1 Run the full Definition of Done from the workspace root and record each command's result
- [ ] 6.2 Sync the delta into `openspec/specs/observation-bound-register/`
- [ ] 6.3 Archive the change and remove the dated archive copy, leaving `openspec/changes/archive/.gitkeep`
      as the only tracked file there
- [ ] 6.4 Open the pull request into the release branch with the curated squash subject and body, and
      squash-merge it
