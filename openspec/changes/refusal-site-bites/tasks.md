## 1. One refusal vocabulary

- [ ] 1.1 Create `crates/tianheng/tests/support/refusal.rs` holding `Kind`, `Refusal`, `violation`,
      `cannot_judge`, and the named out-of-reach forms carrying a slug
- [ ] 1.2 Give the constructors `#[track_caller]` and read `Location::caller()` **in their own body**, passing
      the location down as a value (D2)
- [ ] 1.3 Add the injection point: `TIANHENG_REFUSAL_MUTANT=<file>:<line>:<kind|message>` swaps the kind or
      replaces the message only at the matching site; `ALL:<mode>` matches every site
- [ ] 1.4 Add recording: `TIANHENG_REFUSAL_RECORD=<path>` appends one `file:line` per construction, `O_APPEND`,
      read deduplicated
- [ ] 1.5 Delete the twin `Kind`/`Refusal`/`violation`/`cannot_judge` from `support/publish_source_gate.rs` and
      `support/release_coherence_gate.rs`; import the shared ones
- [ ] 1.6 Declare `#[path = "support/refusal.rs"] mod refusal;` in `publish_source.rs`,
      `publish_source_integrity.rs` and `release_coherence.rs` — flat, not nested `#[path]` (D6)
- [ ] 1.7 `cargo test -p tianheng --test publish_source --test publish_source_integrity --test release_coherence`
      green with every message and kind unchanged

## 2. The reaction's own falsifiers, written before its verdict

- [ ] 2.1 A direction building two refusals on two different lines of a fixture and requiring the recorded
      locations to **differ** — the `#[track_caller]` chain proof (D2)
- [ ] 2.2 A control requiring `ALL:kind` to make a target run **fail** — the injection is wired
- [ ] 2.3 A control requiring a selector naming no site to leave a target run **green** — the injection is not
      spurious
- [ ] 2.4 A **pair** over fixture refusals: one no assertion distinguishes, required to classify as
      undefended, and one a direction does distinguish, required to classify as defended — a classifier that
      always answers the same way fails one of the two
- [ ] 2.5 Run **each** guard against its own defect, one at a time, and record each failure:
      2.2 against a disabled injection point; 2.3 against a site-match condition forced true; 2.4 against its
      own pair; 2.1 against `#[track_caller]` removed from a fixture constructor. One blanket perturbation is
      not acceptable — disabling the injection leaves 2.3 passing vacuously (a poison that never fires does
      not fire where it was not aimed) and can leave 2.4 passing
- [ ] 2.6 Recording integrity: a failed write and an unparseable line each fail the reaction, seen to fail;
      appends within a process serialised, and one record file per target run

## 3. Enumeration, totality, and reach

- [ ] 3.1 Create `crates/tianheng/tests/refusal_bites.rs`, gated behind `TIANHENG_REFUSAL_BITES`, following
      `pin_bites`' skip-and-say-so shape
- [ ] 3.2 Build the corpus from **dep-info, not from text**: `cargo test --all-features --no-run
      --message-format=json` enumerates every integration-test target and its executable; `<executable>.d` is
      the source list rustc actually read. Derive each site's observing targets from which targets' dep-info
      names the site's *file*. Extract as `support/refusal_sites.rs` so `census.rs` shares one enumerator (5.4)
- [ ] 3.2a Do **not** reimplement module resolution. The rejected draft resolved `#[path]` textually, which
      misses `mod foo;`, `include!` and `#[cfg_attr(…, path)]` and admits `cfg`-excluded files. Verified against
      a real build before designing around it: the `.d` for `release_coherence` names its `#[path]` support file
- [ ] 3.3 Refuse a source line carrying two constructions, and refuse any `Refusal` struct literal outside the
      constructors — the totality the enumeration rests on
- [ ] 3.3a Refuse a file in the corpus that `git ls-files` does not name — a judged target compiling content
      no review can see
- [ ] 3.3b Refuse the **exact** shared vocabulary (the refusal type's name, its cannot-judge variant, the
      constructor names) defined in the corpus outside `support/refusal.rs`; fixture carrying one, seen to fail.
      Claim nothing wider: a renamed vocabulary is a declared bound (5.2b), not coverage
- [ ] 3.4 Refuse an enumeration that yields no site, and one that yields no reached site — the vacuity
      directions
- [ ] 3.5 Run the recording pass and produce the reached set; cross-check it against the static enumeration.
      Add a direction over a worktree carrying an uncommitted, unreachable site and require it to be named
- [ ] 3.6 Implement the five-way classification (D5) as a function returning a verdict per site, so the
      reaction's judgement is testable rather than an `assert!` over a clean tree

## 4. The perturbation sweep

- [ ] 4.1 Unpoisoned control run per target set, required green before any site is poisoned
- [ ] 4.2 For each reached site, run its target set twice — `kind` then `message` — requiring some direction to
      fail under each, and requiring the run to have executed at least one test
- [ ] 4.3 Report the residual the reaction measures: sites declared out of reach, and the count, in the shape
      `pin_bites` already uses for its uncovered remainder
- [ ] 4.4 **Measure and record the undefended population.** 24 is the floor under "either perturbation"; the
      "both" figure comes from this run and nothing is written against an estimate

## 5. Close the measured population

- [ ] 5.1 For each undefended site, either write the direction that distinguishes it, or — only where the
      precondition is genuinely unconstructible — convert it to the named out-of-reach form with a slug
- [ ] 5.2 Declare the bound covering the exempt class in the **delta** spec and in
      `crates/tianheng/src/bounds.rs`, extent `OutOfReach`, pinned by
      `a_site_declared_out_of_reach_is_only_observed_to_be_unreached`. **Do not widen `Extent`** — it is a
      shipped public type reached through `pub observation_bounds()`. The main spec is not edited during
      apply; the delta reaches it at sync (7.5)
- [ ] 5.2a Add the repository-local exemption registry in test support: `Exemption { slug, bound, because }`,
      joined against the produced site enumeration and the live `observation_bounds()` set
- [ ] 5.2b Declare the two residual bounds this reaction admits: a refusal vocabulary under other names
      (`a_refusal_vocabulary_under_other_names_is_not_observed`), and a file outside the perturbed feature set
      (`the_corpus_is_the_feature_set_that_is_perturbed`)
- [ ] 5.3 Refuse, each seen to fail: a slug carried by two sites; a slug no registry entry covers; a registry
      entry naming a slug no site carries; a registry entry naming a `BoundId` the live bound set does not
      contain; **an empty registry while the exemption-class bound is declared** (the biconditional — one
      direction alone leaves the bound as permanent residue after the last exemption goes); and a
      declared-out-of-reach site observed being reached (the stale exemption)
- [ ] 5.4 Declare the exempt census in `crates/tianheng/tests/census.rs`:
      **phrase** `"{} of {} refusal sites are declared out of reach"` — the sweep's own declaration guards
      compute and assert the length and single-line constraints, so no figure about the phrase is typed here;
      **figures** produced by `support/refusal_sites.rs`, the same enumerator `refusal_bites.rs` uses, so the
      census and the reaction cannot disagree — mirroring how `support/bound_register_parse.rs` is shared;
      **carrier during apply**: the sentence goes in the delta spec only
      (`openspec/changes/refusal-site-bites/specs/rust-self-governance-gates/spec.md`), on one line. It reaches
      `openspec/specs/` at sync (7.5), never by editing the main spec during apply
- [ ] 5.5 `TIANHENG_REFUSAL_BITES=1 … cargo test -p tianheng --test refusal_bites` green

## 6. Where the run is decided, and where the injection is scrubbed

- [ ] 6.1 `scripts/publish.sh`: invoke the publish-source gate under
      `env -u TIANHENG_REFUSAL_MUTANT -u TIANHENG_REFUSAL_RECORD -u TIANHENG_REFUSAL_BITES` — all three are
      instrumentation, including `RECORD`, which writes a file from inside the gate (D8)
- [ ] 6.2 Name the reaction on its own line in the `AGENTS.md` Definition of Done, with the cost and the reason
      it is gated
- [ ] 6.3 Add it to `.github/workflows/ci.yml` on its own line, beside the mutation and examples suites
- [ ] 6.4 Verify 6.1 actually scrubs: with `TIANHENG_REFUSAL_MUTANT` exported, the publish gate still refuses
      the shape it should

## 7. Records and closure

- [ ] 7.1 `CHANGELOG.md` `[Unreleased]` under `### Self-governance` — no adopter-facing vocabulary, no version
      bump anywhere
- [ ] 7.2 Retire the `WATCH: a refusal site is defended only if…` entry from `BACKLOG.md`, stating what closed
      it and what the residual now is
- [ ] 7.3 Regenerate `docs/observation-bounds.md` and any projection whose figures moved; check no prose figure
      about this set was typed
- [ ] 7.4 Full Definition of Done including MSRV 1.85 (**no let-chains**) and both gated suites
- [ ] 7.5 `openspec validate refusal-site-bites`, sync the delta into
      `openspec/specs/rust-self-governance-gates/spec.md`, archive the change
- [ ] 7.6 One squash PR from `change/refusal-site-bites` into `release/0.5.0`, curated subject and body, no AI
      attribution
