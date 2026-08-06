## 1. Close the shape gaps first, so the reaction lands green

Ordered first deliberately: a reaction that lands with known-failing units teaches that its exemptions are
negotiable (design D4). Each of these is a real defect in the twin, not paperwork.

- [ ] 1.1 Add the empty-stderr assertion to `scripts/test_bound_register.sh`, and record the stderr observed
      on a clean run before the assertion existed.
- [ ] 1.2 Add it to `scripts/test_publish_source.sh`, same record.
- [ ] 1.3 Add it to `scripts/test_reference_integrity.sh`, same record.
- [ ] 1.4 Add it to `scripts/test_release_coherence.sh`, same record.
- [ ] 1.5 Add the unchanged-repository assertion to `scripts/test_reference_integrity.sh` — the one twin with
      no read-only direction. Observe it failing by making the gate write a file.
- [ ] 1.6 Re-measure all nine properties across the six gates and six twins. Every column must read 6 of 6
      before task 2 begins, or the reaction's first run is not a measurement of the contract but of the
      leftovers.

## 2. The reaction

- [ ] 2.1 New self-governance test module under `crates/tianheng/tests/`, following the
      `TIANHENG_WORKSPACE_TESTS` discipline: return outside a checkout, loud failure when the marker is set
      and the layout is absent (spec: *The reaction SHALL refuse to skip silently in CI*).
- [ ] 2.2 Enumerate the surface from `git ls-files 'scripts/check_*.sh'`, pairing each gate with its twin by
      basename substitution. Tracked content, never a filesystem walk.
- [ ] 2.3 Fail loudly on an empty enumeration, naming the emptiness. Verified by pointing the reaction at a
      fixture directory with no gate — the vacuity direction, six recurrences in this window.
- [ ] 2.4 Assert properties 1–3 per gate: backstop sourced **and invoked**; header declares a three-way
      contract ending in cannot-judge with the verdict words free (design D5 — a literal-sentence probe reads
      3 of 6 and would invent three violations); a target directory argument.
- [ ] 2.5 Assert properties 4–8 per twin: existence, `expected_status`, both a passing and a refusing
      direction, an unchanged-repository assertion, an empty-stderr assertion.
- [ ] 2.6 Assert property 9 against `AGENTS.md`'s Definition of Done block, locating the block by heading and
      fence, and failing loudly if that shape changes rather than parsing to zero commands and passing
      vacuously (the shape `check_dod_coherence.sh` already refuses for its own list).
- [ ] 2.7 Every failure names the offending file and the property, one message per offence. A reaction that
      reports "the gate surface is non-conformant" has moved the search cost onto the reader.

## 3. The two exemptions, checked live

- [ ] 3.1 Exempt `scripts/check_publish_source.sh` from property 9's gate half, and assert its twin's
      membership regardless.
- [ ] 3.2 Fail when that exemption stops applying — the publish-time gate appearing in the Definition of Done
      means the exemption is stale and must be retired, not silently kept. Observe the failure by adding the
      line to a fixture `AGENTS.md`.
- [ ] 3.3 Exclude every unit outside the gate-and-twin pairing, and assert **no excluded unit installs
      `exit_contract_backstop`** — the library defining it excepted. The exclusion is by naming, so this is
      what stops it becoming a place a gate can hide: a `verify_*.sh` carrying the contract would otherwise
      leave the surface by rename rather than by a spec change.
- [ ] 3.4 Do **not** assert that every excluded unit is a library or a matrix over one. Written that way
      first, and it is false: `scripts/test_examples.sh` is the example runner and `scripts/publish.sh` is a
      tool, neither of which fits. A classification nobody can state is not a check; the backstop test above
      is the property that actually matters.

## 4. The bounds

Each is a `#### Scenario:` whose heading marks it a bound, with exactly one `PINNED-BY`. The register resolves
these at sync, so the tests must exist and be harness-registered before task 6.

- [ ] 4.1 `a_missing_vacuity_guard_is_a_stated_semantic_bound` — a fixture gate iterating an enumeration with
      no zero-iteration guard passes the shape reaction, demonstrating the bound rather than asserting it in
      prose.
- [ ] 4.2 `an_unchecked_read_status_is_a_stated_semantic_bound` — same form, for a read whose status is never
      inspected in the parent.
- [ ] 4.3 `a_wrong_one_versus_two_assignment_is_a_stated_semantic_bound` — same form, for a gate whose twin
      asserts codes while the gate assigns the wrong one (the `return`-instead-of-`exit` inversion).
- [ ] 4.4 `the_publish_time_gate_is_exempt_from_dod_membership` and
      `units_outside_the_gate_pairing_are_outside_the_surface`.
- [ ] 4.5 Confirm each name resolves under `bash scripts/check_bound_register.sh` **after** sync, and that the
      projection's leading figure still reads 0 unpinned.

## 5. The projection

- [ ] 5.1 Emit the projection under `docs/`, blessed by an environment variable and diffed on every run,
      through the existing `GovernanceTest` projection-freshness machinery rather than a new mechanism.
- [ ] 5.2 Its header states what conformance in it does **not** mean: the three semantic bounds, the coverage
      bound, and that form-conformance is not substance (design *Risks*).
- [ ] 5.3 Print the measured columns rather than writing them into prose. No **surviving** document states a
      per-property count outside the generated projection — the spec delta carries none, and `proposal.md` /
      `design.md` record dated measurements against a named revision and dissolve at sync. Stated this
      precisely on the second pass: the first draft of this task forbade what `design.md` was doing three
      files away, which is the same defect one level up.
- [ ] 5.4 Observe the staleness direction: edit the projection by hand, watch the reaction fail and name the
      blessing command.

## 6. Coherence, and what this change does not touch

- [ ] 6.1 `AGENTS.md` — no Definition of Done change (the reaction rides the existing `cargo test` line). Add
      the new projection to the *Self-governance* paragraph beside `AGENTS.self-law.md` and
      `docs/observation-bounds.md`.
- [ ] 6.2 **No `CHANGELOG.md` entry.** That file is the adopter-facing projection; the publish gate and the
      bound register earned entries because an adopter verifies a tarball against the one and reads the other
      before reporting a defect. Nothing an adopter does touches this capability. Recorded here as a decision
      so it can be argued with rather than noticed as an omission.
- [ ] 6.3 `BACKLOG.md` at sync: close the entry, and correct its claim that the per-gate table "is uniform
      today" — measured, properties 7 and 8 read 5 of 6 and 2 of 6. Keep the lesson: the entry was right that
      nothing enforced the shape and wrong that the shape held, and it was wrong because it counted the
      properties it had just finished creating.
- [ ] 6.4 Sweep for prose this change invalidates in the same window — three sites have been the pattern
      (spec, `CHANGELOG.md`, doc comment); here the candidates are `PROJECT.md`'s audit-cycle decision, which
      gains a second instance of the enumerate-react-audit shape, and the WATCH entry on shell-gate
      capabilities, whose trigger this change deliberately does not fire (design D1).

## 7. Verification — a guard is not a guard until it has been seen to fail

- [ ] 7.1 For every assertion added in tasks 1–5, record the failure observed **without** the change: the
      offending state, the message, and the exit status. This goes in the pull request's `## Verification`
      section, not only in a commit body.
- [ ] 7.2 Point the reaction at a fixture repository per property, one gate missing one property at a time,
      and confirm each failure names that property and that file. A reaction that fails for the right reason
      only in aggregate cannot be trusted to have nine reasons.
- [ ] 7.3 Confirm the passing direction on the real tree: after task 1, the reaction reports every gate
      conformant, and its projection matches.
- [ ] 7.4 Run the full Definition of Done. Then run it again from a clean clone, since two properties are
      judged against tracked content and a local untracked file has changed a gate's answer before.
