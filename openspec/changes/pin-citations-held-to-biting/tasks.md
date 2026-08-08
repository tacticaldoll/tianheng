## 1. Observe the gap before closing it

- [x] 1.1 Confirm the register decides running and not biting: delete the assertions from one cited pinning
  test in a scratch tree and record that `check_bound_register.sh` still reports clean.
- [x] 1.2 Record the isolation measurement — a mutated scratch tree built against the repository's own
  `target/` reports the pin as biting in hundredths of a second — as the evidence the gate's own target
  directory is a requirement rather than a preference.

## 2. Declare the mutations

- [x] 2.1 Add `scripts/lib/pin_mutations.tsv` with the four fields, `\n` and `\t` escapes, and a header comment
  stating that `from` must occur exactly once.
- [x] 2.2 Seed it with mutations that are each **verified to kill** their citation — a mutation is not added
  until it has been run and seen to fail the pin it names.
- [x] 2.3 For every seeded mutation, record the observed failure for the PR's `## Verification`.

## 3. Build the gate

- [x] 3.1 `scripts/check_pin_bites.sh`: install the shared backstop from `scripts/lib/exit_contract.sh`, take
  the gate's own label, declare the three-way contract in the header, accept a target directory argument.
- [x] 3.2 Build the scratch tree from `git archive HEAD`, with a target directory the gate owns; warm it once.
- [x] 3.3 Per record: require the anchor to occur exactly once (else cannot judge), apply, run only the cited
  test, require failure, restore the file before the next record.
- [x] 3.4 Cross-check records against the register's citations in both directions.
- [x] 3.5 Refuse an empty record set rather than reporting every property of zero mutations satisfied.
- [x] 3.6 Print the uncovered count on a clean run.

## 4. Prove every refusal

- [x] 4.1 `scripts/test_pin_bites.sh`: assert expected exit **codes**, hold a passing direction, and cover each
  refusal — a surviving pin, an absent anchor, an ambiguous anchor, a mutation naming an uncited test, an empty
  record set, and a build that cannot be produced.
- [x] 4.2 Confirm the twin holds the five properties `gate-shape-contract` requires, and that
  `gate_shape_contract.rs` accepts the new pair with no edit to any list.

## 5. Join the enforced surface

- [x] 5.1 Add both scripts to `AGENTS.md`'s Definition of Done in order, with the comment saying what each
  buys.
- [x] 5.2 Add them to `.github/workflows/ci.yml` so `check_dod_coherence.sh` passes.
- [x] 5.3 `CHANGELOG.md` `[Unreleased]` entry; `BACKLOG.md` entry for the uncovered remainder, naming what
  closing it costs per bound.

## 6. Definition of Done

- [x] 6.1 Full `AGENTS.md` Definition of Done in order, including the two new gates.
- [ ] 6.2 Sync the delta into `openspec/specs/observation-bound-register/spec.md` and prune the change
  directory.
