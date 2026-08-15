## 1. Prose-trigger and negation matching

- [ ] 1.1 Add `states_a_bound_in_prose(line: &str) -> bool` to `bound_register_parse.rs`: hand-written,
      whitespace-tokenized port of the shell era's `(stated|documented)( [A-Za-z-]+)? bounds?`, word-boundary
      aware (reuse or mirror `contains_words`'s boundary logic).
- [ ] 1.2 Add `negates_bound(line: &str) -> bool`: port of `(rather than|not|never) an?( [A-Za-z-]+)? bounds?`,
      the negation-directly-on-the-noun exclusion.
- [ ] 1.3 Unit tests for both, using the shell era's own historical examples as fixtures: the three sentences
      that a wider "anywhere nearby" negation rule was measured to hide (`"type aliases are not expanded (a
      stated bound)"`, `"the invocation is not transparent, so its body stays a stated coverage bound"`, `"a
      production probe must not live behind a non-production cfg — a stated bound"` — all three must trigger
      as declarations, not negations), plus the one-interposed-word tolerance case (`"a stated coverage
      bound"`) and a bare-adjacent case (`"a stated bound"`, `"a documented bound"`).

## 2. Requirement/scenario state-machine walk

- [ ] 2.1 Add `undeclared_prose_offences(capability: &str, spec: &str, text: &str, capabilities:
      &BTreeSet<String>) -> Vec<String>` (or equivalent): single-pass line walk tracking the current `###
      Requirement:` heading (and whether its own wording names bounds) and the current `####` block (and
      whether `marks_a_bound` accepts it), mirroring the shell script's `req`/`req_is_bounds`/`req_stated`/
      `open` state exactly.
- [ ] 2.2 Wire in `bare_references` so a triggering line carrying a resolvable reference is cleared rather
      than reported.
- [ ] 2.3 Wire in the bounds-named-requirement exemption: a triggering line under such a requirement does not
      fail directly; instead the requirement itself must declare at least one bound scenario, checked once at
      the end of (or exit from) that requirement's section.
- [ ] 2.4 Tests for the four spec scenarios directly, using synthetic multi-line spec text (matching this
      file's own existing style for `bounds_in`/`marks_a_bound` tests, not the live corpus):
  - [ ] 2.4.1 Prose stating a bound with no wrapping scenario and no reference fails, naming the occurrence.
  - [ ] 2.4.2 The same statement inside a declared bound scenario does not fail.
  - [ ] 2.4.3 Prose under a bounds-named requirement heading is exempt, but that requirement then fails if it
        declares no bound scenario of its own — and passes if it declares at least one.
  - [ ] 2.4.4 A synthetic case with a resolvable `<capability>/<slug>` reference is cleared even though it
        states a bound and is not inside a scenario.

## 3. Wiring into the ordinary suite

- [ ] 3.1 Call the new scan from `bound_register.rs` (a new `#[test]`, e.g.
      `every_bound_stated_in_prose_is_declared_as_a_scenario`), iterating `tracked_specs(&root)` the same way
      `every_declared_bound_carries_exactly_one_citation` does.
- [ ] 3.2 Run it against the live tree. If it reports any real offence, fix that spec's prose in this same
      change (declare the bound as a proper scenario, or add a `<capability>/<slug>` reference) rather than
      suppressing or deferring the finding.
- [ ] 3.3 Negative-run check: temporarily add a real undeclared bound-prose sentence to a scratch spec
      fixture (or, if the corpus-integrated test makes that awkward, to the synthetic test's own input) and
      confirm the new test fails before the fix, then reverts clean.

## 4. Documentation and closeout

- [ ] 4.1 Confirm `docs/observation-bounds.md`'s projection header still accurately states the three residuals
      now that a real reaction exists (no projection content change expected, per design.md's non-goals, but
      confirm rather than assume).
- [ ] 4.2 `cargo clippy -p kanhe --all-targets -- -D warnings`, `cargo fmt --all --check`.
- [ ] 4.3 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test whitespace_hygiene --test reference_integrity
      --test census --test dod_coherence`.
- [ ] 4.4 CHANGELOG.md entry (`### Self-governance`) and check off Contract #6 in
      `docs/audit/0.5.0-static-review-remediation.md`.
- [ ] 4.5 Sync the delta spec (confirms text parity — no wording change expected) and archive the change.
