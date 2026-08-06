## 1. The model in 璇璣

- [ ] 1.1 `crates/xuanji/src/bound.rs`: `BoundId`, `BoundDecl`, `Extent`, `Reached`, `Owner`, and the bounded
      fact-granularity value. Every enum `#[non_exhaustive]`, matching the crate's existing convention.
- [ ] 1.2 Nesting, not flags: `Extent::OutOfReach` carries no reaction direction and no owner. Assert the
      absence structurally — a unit test that would have to name a field that does not exist is the wrong
      proof, so the proof is the doc comment plus the compile itself, recorded in the PR.
- [ ] 1.3 Granularity is carried only by the as-intended value. No other extent offers it.
- [ ] 1.4 `Owner` is carried only by the under-reacting value, and its inherited form names the layer.
- [ ] 1.5 Derive the demonstrated direction from the extent (`Extent::demonstrates`). No declared field beside
      it; the second copy is what could disagree.
- [ ] 1.6 Export from `crates/xuanji/src/lib.rs`, and state in the crate doc how this differs from
      `ScanDepth` — how far a scan walks versus where the measure stops. Both names live in this crate on
      purpose (design D1); the doc comments are what keep them apart.
- [ ] 1.7 Unit tests for the derivation and for `BoundId` construction. Rejected as untestable-by-construction
      and recorded rather than skipped silently: "an out-of-reach bound cannot claim an owner" has no test,
      because the code expressing it does not compile.

## 2. Each owning dimension declares its bounds

- [ ] 2.1 `guibiao::observation_bounds()` — the static dimension's declared bounds, as a library item.
- [ ] 2.2 `hunyi::observation_bounds()` — the semantic dimension's, the largest set.
- [ ] 2.3 `louke::observation_bounds()` — the runtime dimension's.
- [ ] 2.4 No accessor in `xuanji`, `xingbiao`, or `tianheng`. They own no declared bound, and an empty accessor
      would be a name with nothing behind it.
- [ ] 2.5 Each declaration's id is the register's derived `<capability>/<scenario-slug>`, transcribed from the
      spec heading rather than invented. Verify by running the bijection reaction, not by eye.
- [ ] 2.6 Classify each bound from its WHEN/THEN, not from its old adjective. Two known traps, both measured:
      `cfg-blind` marks an over-reaction in one capability and an under-reaction in another; and the
      unrenderable-sub-node bound reacts correctly and is bounded only in granularity, so it is not an
      under-reaction.

## 3. The bijection reaction

- [ ] 3.1 New test module under `crates/tianheng/tests/` — the only crate that sees all three dimensions —
      under the `TIANHENG_WORKSPACE_TESTS` discipline: return outside a checkout, loud failure when the marker
      is set and the layout is absent.
- [ ] 3.2 Parse bound ids from `openspec/specs/*/spec.md` by the same derivation the register uses — lowercase,
      runs of non-alphanumerics to one hyphen, ends trimmed. Fail loudly if the enumeration reads zero specs or
      zero bounds — the vacuity direction this repository has re-opened six times in one window.
- [ ] 3.2a Assert the derived id set **equals** the id set in `docs/observation-bounds.md`. This is the only
      guard against a second slug implementation drifting from the shell's, and it catches a stale projection in
      the same assertion. Do not skip it on the grounds that the rule is three lines: the same file records a
      review round lost to two matchers whose character classes differed (design D9).
- [ ] 3.3 Assert ids duplicate-free on **both** sides before comparing, since two declarations collapsing onto
      one id would satisfy set equality while leaving a bound unclassified.
- [ ] 3.4 Assert set **equality**, and name every id on either side with no counterpart. Not a subset in either
      direction: a spec bound with no declaration is unclassified, a declaration with no spec bound is a
      classification no reader can find.
- [ ] 3.5 Assert every under-reacting declaration carries an owner — enforced by the type, so this asserts the
      *set* is non-empty where the specs say it should be, catching a sweep that classified a false negative as
      something milder.

## 4. The projection

- [ ] 4.1 Generalize the bless-and-diff rule once: `GovernanceTest::assert_projection_fresh` renders the
      constitution specifically, so add an additive method taking already-rendered content and have the existing
      one delegate to it. Do **not** reimplement bless in the new test module — the tree already carries a shell
      and a Rust implementation, and a third would be the drift this mechanism exists to prevent (design D10).
- [ ] 4.2 Generate a projection grouping every declared bound by extent, through that generalized method.
- [ ] 4.3 Lead with the count of declared false negatives and their owners. Print the figures; write none of
      them into prose anywhere.
- [ ] 4.4 Its header states what it does not claim: that the classification is authored, that no reaction
      verifies a rationale against its extent, and the three modelling bounds this capability declares.
- [ ] 4.5 Add a pointer from `docs/observation-bounds.md`'s header to it, and state the division of subjects —
      that one projects what the specs declare and the other what kind of stop each is.
- [ ] 4.6 Observe the staleness direction: edit the projection by hand, watch the reaction fail and name the
      blessing command.

## 5. Close the qualifier slot

- [ ] 5.1 Sweep every bound heading carrying a qualifier down to the bare marker. The meaning the qualifier
      carried moves into the declaration's rationale, not into the heading. Leave `stated` / `documented`
      alone — noise, not a false taxonomy, and each change costs an id (design D8).
- [ ] 5.2 Tighten **`BOUND_HEADING` only**, and add an explicit refusal that names a qualified heading and its
      repair. A non-match would fall through to the undeclared-prose direction and report the wrong thing.
- [ ] 5.3 **Leave `BOUND_PROSE` permissive.** It is the register's detection floor, and narrowing it would stop
      it reporting a bound stated in prose with a qualifier — a false negative in the direction that stops the
      register being completed by declaring only the convenient bounds. The first draft of this change narrowed
      both; recorded here so the asymmetry is not "fixed" later by someone making them consistent.
- [ ] 5.4 Update every in-tree `(bound: …)` reference whose id the sweep changed — the id is the heading's
      slug, so a swept heading is a renamed bound. The register's reference-resolution reaction catches a
      missed one; run it before believing the sweep is complete.
- [ ] 5.5 Extend `scripts/test_bound_register.sh` with two directions, each asserting the exit **code** per that
      matrix's existing form: a qualified heading refused, and prose with a qualifier still detected.
- [ ] 5.6 Regenerate `docs/observation-bounds.md` and expect **ids to change** for every swept bound — not an
      identical projection. Confirm the diff is exactly the swept ids and their statements, with no bound
      appearing or disappearing.

## 6. This capability's own bounds

- [ ] 6.1 `a_rationale_that_contradicts_its_extent_is_a_stated_bound` — the model does not read the rationale.
- [ ] 6.2 `an_entry_dependent_bound_is_declared_as_under_reacting` — the one entry-dependent bound is expressed
      through an existing value rather than earning its own.
- [ ] 6.3 `granularity_is_carried_only_by_the_as_intended_extent` — the unexpressible pair.
- [ ] 6.4 Each is a bound-marked scenario with the single marker and one `PINNED-BY`, and each resolves under
      `bash scripts/check_bound_register.sh` after sync. The projection's unpinned figure must still read zero.

## 7. Coherence

- [ ] 7.1 `CHANGELOG.md`: an `### Added` entry stating the new published surface **and that adopters migrate
      nothing**. "New public API" reads as work; here it is opt-in to read, and saying so is the entry's point.
- [ ] 7.2 `AGENTS.md`: no Definition of Done change — the reaction rides the existing `cargo test` line. Add
      the new projection to the *Self-governance* paragraph beside `AGENTS.self-law.md` and
      `docs/observation-bounds.md`.
- [ ] 7.3 `PROJECT.md`: this is the audit-cycle decision's second instance (enumerate, react, audit against the
      enumeration). Record it as an instance; do not restate the decision.
- [ ] 7.4 `BACKLOG.md`: file the follow-on `observer-protocol` change with its forced dependency stated — its
      `bounds()` has nothing to delegate to without this one. Also note that `gate-shape-contract`'s parked
      proposal must be revised before it lands: it mints two qualifier phrasings, and its membership exemption
      is a policy exemption rather than an observation bound, so it does not belong in this register at all.
- [ ] 7.5 Sweep for prose this change invalidates in the same window — three sites have been the pattern (spec,
      `CHANGELOG.md`, doc comment). Here the candidates are the register spec's own description of its
      recognizer and any comment in `check_bound_register.sh` describing the qualifier slot.

## 8. Verification — a guard is not a guard until it has been seen to fail

- [ ] 8.1 For every assertion in tasks 3–6, record the failure observed **without** the change: the offending
      state, the message, the exit status. In the pull request's `## Verification`, not only in a commit body.
- [ ] 8.2 Point the bijection reaction at each direction separately: a spec bound with no declaration, a
      declaration with no spec bound, duplicate ids on each side, and a zero-bound enumeration. Confirm each
      failure names the right id and the right direction.
- [ ] 8.3 Confirm the passing direction on the real tree, and that the projection matches after task 2's
      classification.
- [ ] 8.4 Run the full Definition of Done. Then run it again from a clean clone, since the bijection reads
      tracked content and a local untracked spec has changed a gate's answer in this repository before.
- [ ] 8.5 Confirm the additive claim mechanically: no existing public item's signature changed. `cargo doc` and
      the packaged-tarball self-test both pass, and the CHANGELOG's no-migration claim is stated against that
      check rather than against confidence.
