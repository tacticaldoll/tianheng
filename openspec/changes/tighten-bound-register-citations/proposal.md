## Why

An adversarial review of `release/0.4.1` found the observation-bound register's reaction weaker than the
requirements it was written to enforce, in four independent directions. Two of them let a citation read as
coverage while defending nothing — the exact silent pass the register exists to end, now reachable inside
the register itself:

- `PINNED-BY` resolves against **any** Rust function definition, so a production helper of the right name
  satisfies a citation the spec says names "the test that pins it". All 36 citations happen to name real
  `#[test]` functions today; nothing makes that true tomorrow.
- `UNPINNED` accepts **any non-empty text** as a tracker, so `- **UNPINNED** no test exists` passes the very
  reaction whose requirement says a citation that merely asserts the absence of a test SHALL fail.

The other two are claim/mechanism mismatches rather than coverage holes:

- The shared-bound requirement claims one behaviour change cannot leave several specs stale. The reaction
  keys on a **shared PINNED-BY citation**, so two capabilities declaring one behaviour with two different
  tests pass. That shape exists in the tree today, and the historical `#[path]` restatement the requirement
  cites was actually caught by the undeclared-prose direction, not this one. The claim is broader than the
  direction beneath it, which is the stale-declaration failure the register was built to prevent — here in
  the register's own spec.
- `BLESS=1` writes the projection and exits 0 before evaluating offenses, so regeneration reports success
  over a register it just printed offenses for, against the script's own declared 0-clean/1-violation
  contract. CI runs the non-blessing line, so this misleads a local run rather than passing a violation.

And the capability's `Purpose` is still the archive-generated `TBD` placeholder — the only one of 30 specs
that is, so it is this change's omission rather than a repository convention.

## What Changes

- The pinning-citation reaction requires the resolved definition to be a **test**: an attribute run above
  the definition containing `#[test]`, scanning upward past interleaved attributes (`#[should_panic]` sits
  between attribute and `fn` in three places in this tree today). A same-named ordinary function fails.
- The unpinned-citation reaction requires the tracker to **name a path git tracks**. Text naming no tracked
  file fails; a path that does not exist fails.
- The shared-bound requirement is **narrowed to the shape its reaction observes** — one citation claimed by
  two capabilities — and the residual is stated as a floor in the spec and in the projection's header,
  beside the undeclared-prose floor already stated there. The residual is **not** declared as a bound:
  distinguishing two declarations of one behaviour from two behaviours over sibling shapes is a semantic
  judgment no reaction can reach, and a declaration nothing observes is the name-without-a-reaction
  `PROJECT.md` forbids.
- `BLESS=1` writes the projection and then falls into the **same verdict logic** as any other run, so its
  exit code carries the family's contract. The cannot-judge condition moves ahead of writing, so a
  vacuous register cannot produce a projection at all.
- The capability's `Purpose` is written.

No crate behaviour changes: every edit is to a repository gate, its failure matrix, and its spec. Both
tightened directions are satisfied by all 41 citations in the tree today, so neither is a migration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-register`: the pinning citation must resolve to a test rather than to any function; the
  unpinned citation's tracker must name a tracked path; the shared-bound requirement is narrowed to the
  shared-citation shape with its residual stated as a floor; regeneration is bound by the same exit
  contract as judgment; the projection header states the second floor.

## Impact

- `scripts/check_bound_register.sh` — the two citation directions, the bless path, the projection header.
- `scripts/test_bound_register.sh` — new fixtures for each new refusal and for the passing direction of
  each; the fixture helper stops depending on bless returning 0 over an invalid register.
- `openspec/specs/observation-bound-register/spec.md` — Purpose, and the requirements above.
- `docs/observation-bounds.md` — regenerated (header text).
- No crate, no public API, no adopter action.
