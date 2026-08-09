## 1. What `clean` means

- [x] 1.1 Spec delta: ignored by **tracked** repository content is clean; hidden by this clone or this machine
      is not. Written first, so every control below asserts a requirement rather than a preference

## 2. The publish gate's own git

- [ ] 2.1 Route the judgement's `git()` through `hermetic()` **and** `-c core.excludesFile=/dev/null`
- [ ] 2.2 A direction: repo-local `.git/config` setting `core.excludesFile` to hide an untracked file — clean
      before, a violation after
- [ ] 2.3 Classify what no configuration neutralises: the difference between an unexcluded and an excluded
      listing, each path's source read with `check-ignore -v --no-index`, a source legitimate only if **tracked**
- [ ] 2.4 Three directions, landing together: `.git/info/exclude` refuses; a **tracked** `.gitignore` is
      accepted; an **untracked** `.gitignore` refuses
- [ ] 2.5 Every new refusal site reached by a direction, or declared — `refusal_bites` green

## 3. The release-coherence enumerations

- [ ] 3.1 An example manifest that exists and cannot be read is a cannot-judge naming the path; a directory with
      no `Cargo.toml` is still skipped
- [ ] 3.2 Propagate a failed directory entry in both enumerations
- [ ] 3.3 Attempt a fixture producing an iteration error; if none exists, declare each site with **its own slug**
- [ ] 3.4 `refusal_bites` green and the exempt census moved if anything was declared

## 4. The citation answered twice

- [ ] 4.1 Repeated `UNPINNED` becomes an invalid citation state naming the bound
- [ ] 4.2 A direction for it, and a control keeping two `PINNED-BY` accepted

## 5. Records and closure

- [ ] 5.1 `CHANGELOG.md` `[Unreleased]`, no version bump
- [ ] 5.2 Full Definition of Done including MSRV 1.85 and all four gated suites
- [ ] 5.3 Sync the three deltas, archive, one squash PR merged through `scripts/merge-pr.sh`
