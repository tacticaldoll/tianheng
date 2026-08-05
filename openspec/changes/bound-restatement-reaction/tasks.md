## 1. React

- [x] 1.1 Add the direction: a pinning test cited by declared bounds in more than one capability fails,
      naming every declaring capability and the shared test
- [x] 1.2 Confirm it does not fire on a bound citing two tests, nor on one capability citing one test twice
- [x] 1.3 Add both fixtures — the restatement failing, and the two non-restatement shapes passing

## 2. Repair

- [x] 2.1 Keep the declaration in `semantic-signature-coupling` and replace the parallel declarations in
      `semantic-forbidden-marker` and `semantic-trait-impl-locality` with references
- [x] 2.2 Regenerate the projection and confirm the bound count falls by four

## 3. Record

- [x] 3.1 Modify `observation-bound-register`'s shared-bound requirement, stating that the earlier
      per-capability rule is superseded and why
- [x] 3.2 Close the `READY-PATCH` entry with its resolution, moving its reproduction record per the backlog
      governance rule
- [ ] 3.3 Run the full Definition of Done, sync, archive with the dated copy pruned, and land one squash PR
