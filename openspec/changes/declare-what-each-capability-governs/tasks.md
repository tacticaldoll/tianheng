## 1. See what is being claimed, before changing any of it

- [ ] 1.1 Record the rename surface: occurrences of the capability id, including the published bound ids in
  `crates/tianheng/src/bounds.rs`, and the projections that carry them.
- [ ] 1.2 Record which capability each currently-tracked path would be claimed by, so a subject written in
  task 3 can be compared against something rather than judged by eye.

## 2. The rename, as one atomic step

- [ ] 2.1 Move `openspec/specs/rust-self-governance-gates/` to `openspec/specs/rust-repository-reactions/`
  with its requirement set **verbatim**; the only edit in the file is its own title line.
- [ ] 2.2 Sweep the capability-id occurrences, including the published bound ids and their citations.
  Regenerate every projection rather than hand-editing it.
- [ ] 2.3 `bound_register` and `observation_bound_model` hold the derived-id join, so a missed occurrence is
  red rather than silent. Confirm that is what happens by leaving one unswept on purpose first.
- [ ] 2.4 `CHANGELOG.md` under `[Unreleased]`: four published bound ids changed value. No version bump.

## 3. Every capability says what it governs

- [ ] 3.1 Add `## Subject` to all 36 capability specs, each listing the tracked-path globs it governs. Derive
  each from what the capability's requirements actually talk about; where a subject is genuinely wide, say so
  in globs rather than narrowing it to look tidy.
- [ ] 3.2 The reaction: every capability declares a subject; every declared glob matches at least one tracked
  path; a failed `git ls-files` is a cannot-judge. Failure matrix over fixture spec text, each direction with
  its own message.
- [ ] 3.3 See it red before green: remove one `## Subject`, then point one glob at nothing.

## 4. The filing decision is joined to what the change touches

- [ ] 4.1 Resolve the base from the branch's upstream and the tracked release/main refs; unresolvable is a
  cannot-judge. No active change is clean.
- [ ] 4.2 The join: each touched file claimed by some capability's subject requires one claiming capability in
  the proposal's Capabilities section. Exclude the change's own directory from the touched set.
- [ ] 4.3 See it red against the real defect: point this change's own proposal at a capability list omitting a
  capability whose subject it touches, and observe the refusal name the file, the claiming capability, and
  what was listed. Revert with the Edit tool.
- [ ] 4.4 Reconstruct the parked defect as a direction: a change touching `scripts/publish.sh` naming only a
  capability whose subject is this repository's Rust reactions must be refused.
- [ ] 4.5 Declare the non-tiling bound with its scenario, and confirm the reaction reports its unclaimed files
  rather than implying it judged them.

## 5. The new sites join the sweep, and the change lands

- [ ] 5.1 `refusal_bites`: every new construction site defended, and the census compared against the figure
  this branch starts from.
- [ ] 5.2 Full Definition of Done, including every gated suite and `openspec validate --specs --strict`.
- [ ] 5.3 Sync the two deltas — including the verbatim directory move — archive the change, and land the
  branch as one squash PR through `scripts/merge-pr.sh`.
- [ ] 5.4 Rewrite the parked `a-gate-that-matched-no-test` artifacts onto the new names and subjects, and
  confirm the join would have refused their original filing.
