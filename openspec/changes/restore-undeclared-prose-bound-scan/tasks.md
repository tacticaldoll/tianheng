## 1. Prose-trigger and negation matching

- [x] 1.1 Add `states_a_bound_in_prose(line: &str) -> bool` to `bound_register_parse.rs`: hand-written,
      whitespace-tokenized port of the shell era's `(stated|documented)( [A-Za-z-]+)? bounds?`, word-boundary
      aware (reuse or mirror `contains_words`'s boundary logic).
- [x] 1.2 Add `negates_bound_in_prose(line: &str) -> bool`: port of `(rather than|not|never) an?( [A-Za-z-]+)?
      bounds?`, the negation-directly-on-the-noun exclusion.
- [x] 1.3 Unit tests for both, using the shell era's own historical examples as fixtures: the three sentences
      that a wider "anywhere nearby" negation rule was measured to hide (`"type aliases are not expanded (a
      stated bound)"`, `"the invocation is not transparent, so its body stays a stated coverage bound"`, `"a
      production probe must not live behind a non-production cfg — a stated bound"` — all three must trigger
      as declarations, not negations), plus the one-interposed-word tolerance case (`"a stated coverage
      bound"`) and a bare-adjacent case (`"a stated bound"`, `"a documented bound"`).

## 2. Requirement/scenario state-machine walk

- [x] 2.1 Add `undeclared_prose_offences(spec: &str, text: &str, capabilities: &BTreeSet<String>) ->
      Vec<String>`: single-pass line walk tracking the current `### Requirement:` heading (and whether its own
      wording names bounds) and the current `####` block (and whether `marks_a_bound` accepts it), mirroring
      the shell script's `req`/`req_is_bounds`/`req_stated`/`open` state exactly. (`capability` param dropped —
      unused, offences cite `spec` directly.)
- [x] 2.2 Wire in `bare_references` so a triggering line carrying a resolvable reference is cleared rather
      than reported.
- [x] 2.3 Wire in the bounds-named-requirement exemption: a triggering line under such a requirement does not
      fail directly; instead the requirement itself must declare at least one bound scenario, checked once at
      the end of (or exit from) that requirement's section.
- [x] 2.4 Tests for the four spec scenarios directly, using synthetic multi-line spec text (matching this
      file's own existing style for `bounds_in`/`marks_a_bound` tests, not the live corpus):
  - [x] 2.4.1 Prose stating a bound with no wrapping scenario and no reference fails, naming the occurrence.
  - [x] 2.4.2 The same statement inside a declared bound scenario does not fail.
  - [x] 2.4.3 Prose under a bounds-named requirement heading is exempt, but that requirement then fails if it
        declares no bound scenario of its own — and passes if it declares at least one.
  - [x] 2.4.4 A synthetic case with a resolvable `<capability>/<slug>` reference is cleared even though it
        states a bound and is not inside a scenario. (Plus a fifth: a negated bound in prose is not an offence.)

## 3. Wiring into the ordinary suite

- [x] 3.1 Call the new scan from `bound_register.rs` (`every_bound_stated_in_prose_is_declared_as_a_scenario`),
      iterating `tracked_specs(&root)`.
- [x] 3.2 Ran it against the live tree — one real offence found:
      `openspec/specs/runtime-origin-assertion/spec.md`'s "doubly-nested cfg_attr(path) is a stated, undetected
      bound" claim. Adversarially measured (a fixture with the nested target holding the only real probe) and
      found **false** — the scanner's linear "path=" search does not care about nesting depth, so it already
      resolves the doubly-nested case, and apparently has for a while (the claim itself was stale). Fixed: the
      requirement now states this as a SHALL with a new scenario (delta at
      `specs/runtime-origin-assertion/spec.md` in this change), the matching stale comment in
      `crates/louke/src/audit/scan/lexer.rs` corrected, and a new regression test
      (`a_doubly_nested_cfg_attr_path_is_followed_the_same_as_a_single_nesting`) added to
      `crates/louke/src/audit/tests.rs` pinning the now-confirmed-correct behavior.
- [x] 3.3 Negative-run check: appended a real undeclared bound-prose sentence to the end of
      `runtime-origin-assertion/spec.md` — confirmed `every_bound_stated_in_prose_is_declared_as_a_scenario`
      fails naming the exact line, then reverted and confirmed clean.

## 4. Documentation and closeout

- [x] 4.1 Confirmed: `docs/observation-bounds.md` already described these exact three residuals in prose
      (lines 13-28) even before the reaction existed — all three remain accurate descriptions of the new
      scan's actual limitations (unrecognized wording, line-oriented, reference-clears-more-than-it-names).
      No content change needed; `the_register_projection_is_generated_and_fresh` confirms the projection is
      still fresh (no new declared bound was added — the new `runtime-origin-assertion` scenario is a plain
      scenario, not marked a bound).
- [x] 4.2 `cargo clippy -p kanhe -p louke --all-targets --all-features -- -D warnings`, `cargo fmt --all
      --check` — clean.
- [x] 4.3 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test whitespace_hygiene --test reference_integrity
      --test census --test dod_coherence` — all green (first pass tripped on three literal references to the
      deleted shell script's path and a missing trailing newline picked up mid-work; both fixed).
- [x] 4.4 CHANGELOG.md entry (`### Self-governance`) added and Contract #6 checked off in
      `docs/audit/0.5.0-static-review-remediation.md`.
- [ ] 4.5 Sync the delta specs (`observation-bound-register` confirms text parity;
      `runtime-origin-assertion` merges the corrected requirement) and archive the change.
