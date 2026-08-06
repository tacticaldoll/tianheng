## Context

Six `check_*` gates and their twins defend this repository's documents, release spine, and specs. Their
*subjects* are governed by specs; their own *shape* is governed by nothing. `AGENTS.md`'s Definition of Done
binds the gate **list** to CI — `check_dod_coherence.sh` reacts to that — and no reaction binds a gate's
structure to anything.

Every figure below was measured on 2026-08-06 at `c5174a6`, each with the command that produced it. They are
observations of a moment and appear here because they decide the design; the capability's own projection
replaces them, at which point nothing in this document is load-bearing.

Per-property conformance across the six gates and their six twins:

| # | property                                        | probe                              | conformance |
| - | ----------------------------------------------- | ---------------------------------- | ----------- |
| 1 | gate installs the shared backstop               | `exit_contract_backstop`           | 6 of 6      |
| 2 | gate's header declares the three-way contract   | three-way statement, any wording   | 6 of 6      |
| 3 | gate is fixture-addressable                     | `${1:-`                            | 6 of 6      |
| 4 | twin exists                                     | basename substitution              | 6 of 6      |
| 5 | twin asserts exit codes                         | `expected_status`                  | 6 of 6      |
| 6 | twin holds a passing and a refusing direction   | `expect_pass` / `expect_fail`      | 6 of 6      |
| 7 | twin asserts the gate is read-only              | unchanged-repository assertion     | **5 of 6**  |
| 8 | twin asserts a silent clean run                 | empty-stderr assertion             | **2 of 6**  |
| 9 | gate and twin are in the Definition of Done     | block membership                   | 6 of 6 †    |

† with the publish-time gate's declared exemption; the block held 15 `bash` lines against 16 units, and that
difference is exactly that gate.

At `v0.4.0` the same probes over the then-4 gates and 5 matrices read: 0, —, 1, —, 0, —, —, 0, —. Properties
1, 5 and 8 trace to a single commit each in this window (`git log -S… -- scripts/`). The shape is not a
convention several authors converged on; it is what one sweep imposed, three weeks before this proposal.

## Goals / Non-Goals

**Goals**

- Bind the gate surface's structure to a reaction, so the seventh gate inherits the shape by failing a test
  rather than by its author having read six others.
- Enumerate the surface from tracked content, so a new gate joins it without an edit anywhere.
- Say precisely what is not checked, as declared bounds with pinning tests, so nobody reads form-conformance
  as substance.
- Close properties 7 and 8's gaps inside this change, so the reaction lands green.

**Non-Goals**

- Judging whether a gate's *verdict* is right. Correct 1-versus-2 assignment is a declared bound.
- Governing shell units that are not gates. The sourced libraries under `scripts/lib/` and the four matrices
  over them are a declared coverage bound, not a silent omission.
- Defeating an author who wants to evade the contract. See *Risks*.
- Any adopter-visible effect. No public API, no `Constitution`, no baseline format.

## Decisions

### D1 — The reaction is a Rust test under `crates/`, not a seventh shell gate

The backlog entry assumed the `observation-bound-register` shape: a shell gate with a projection and a matrix.
That collides with this capability's own requirement to declare three semantic classes as **observation
bounds**. `PINNED-BY` resolves exactly one harness-registered Rust function under `crates/`, so a
shell-defended capability cannot pin its own bounds; they would land `UNPINNED`, and
`docs/observation-bounds.md` opens with "0 of 42 declared bounds have no pinning test" — deliberately, because
"a number in a footnote is not read". The shell shape's real price is turning that headline into 3 of 45.

The Rust route needs no new machinery, measured rather than assumed:

- Reading repository paths from a test is established in six crates under the `TIANHENG_WORKSPACE_TESTS`
  discipline; `crates/xuanji/src/tests.rs` already scans `crates/` for forbidden identifiers, which is the
  same shape one directory over.
- A blessed, staleness-checked projection is established: `self_law_projection_is_fresh` holds
  `AGENTS.self-law.md` fresh via `GovernanceTest::assert_projection_fresh_with_preamble`.
- The reaction rides the existing `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` line, so
  it adds no Definition of Done entry and no CI step. A shell gate would have added two of each — and would
  itself have entered the surface it judges, which is answerable but is complexity bought for nothing.

**Rejected — extend `PINNED-BY` to resolve a shell fixture.** This is `BACKLOG.md`'s WATCH entry's subject,
whose promotion trigger is "a second residual that is genuinely out of reach — one where the exact observation
source has been *measured* and found unaffordable". The Rust route is affordable and measured, so this
instance does not fire that trigger. Taking it anyway would spend a change on `observation-bound-register` to
escape a constraint this capability never hits.

**Rejected — a shell gate with three `UNPINNED` bounds.** Legal, and the projection would be honest. But a
capability whose entire subject is that gates hold their shape would open its register entry by declaring
three undefended bounds, in a register that has never carried one.

### D2 — A new capability, and its name

**Rejected — a requirement on `governance-dogfood`.** Its subject is the *published boundary families*, kept
exercised through self-governance and adopter-shaped examples; its inventory is the public `Constitution`
family set. The gate surface is neither a published family nor an adopter surface. Folding it in would make
one capability's subject "everything we check about ourselves", which is the shape that stops reacting to
anything in particular.

**Rejected — a requirement on `observation-bound-register`.** It supplies the shape, not the ownership. Its
subject is what the specs *declare*; this capability's subject is what the tree *is*. One staleness check over
two unrelated claim surfaces would make each one's failure ambiguous.

**Rejected — the name `gate-shape-register`.** The register form projects declarations. This projects measured
facts, and borrowing the word would borrow another mechanism's credibility to describe something different.

### D3 — The surface is the `check_*` gate and its twin, with both exemptions declared

`git ls-files 'scripts/check_*.sh'` is enumerable and, as a *definition of the surface*, incomplete — which is
worse than not being enumerable, because it looks complete. Several `test_*` matrices defend sourced function
libraries under `scripts/lib/` that carry no exit contract and no backstop, one is the example runner, and
`scripts/publish.sh` is a tool. Drawing the surface at `check_*` + twin is right, and stating so is the whole
point: a projection titled after the gate surface that silently omits matrices claims a completeness it does
not have.

Because the exclusion is by **naming**, it must not become a place a gate can hide. So the reaction also
refuses an excluded unit that installs `exit_contract_backstop` — the library defining it excepted — which is
the property that makes the exclusion safe. A weaker version was written first: "assert every excluded unit is
a library or a matrix over one". That is false of `scripts/test_examples.sh` and `scripts/publish.sh`, and a
classification nobody can state is not a check.

The second exemption is a **false positive the entry's ninth property would have produced on its first run**.
`check_publish_source.sh` is not in the Definition of Done and must not be — it runs from `scripts/publish.sh`
at publish time, because no development checkout is a release snapshot.

Both exemptions are declared bounds with pinning tests, and the membership exemption additionally carries a
**live-instance check**: were the publish-time gate ever added to the Definition of Done, an exemption that
only permits would go on permitting, and the next reader would inherit a licence with nothing behind it. So
the reaction fails when the exemption stops applying. This direction is cheap here and has been expensive
elsewhere; a proposed matcher that would have fired on a pair already in the tree was caught once in this
window by checking before adopting.

### D4 — Properties 7 and 8's gaps close in this change

Two ways to ship, given the measurement: record the five missing assertions as accepted exemptions, or close
them so the reaction lands green.

**Accepted: close them.** A gate-shape capability that ships with a baseline of shape violations is the
weakening-to-pass shape *Adversarial review stance* forbids, and it would establish that this register's
exemptions are negotiable — the one property it cannot afford. The cost is bounded and known: four twins gain
an empty-stderr assertion, one gains an unchanged-repository assertion.

### D5 — Property 2 is checked by shape, not by wording

The six gates declare their contract six ways: "Exit 0 clean, 1 violation, 2 cannot judge", "0 coherent, 1
incoherent", "0 publishable, 1 wrong source". Each names its own subject better than a shared sentence would.
A probe requiring one literal sentence measures three of six — so the reaction recognizes a three-way
statement whose third term is cannot-judge and leaves the verdict words free.

This is recorded as a decision rather than a detail because it is the invented-violation direction, and the
first draft of this design's own measurement table hit it: property 2 read 3 of 6 under a literal probe and
6 of 6 under the right one.

### D6 — Requiring the twins' helper form is legitimate, by ownership

Property 6 is checked through `expect_pass` / `expect_fail`, a naming convention. `observation-bound-register`
draws exactly this line: it may require a *scenario heading's* form, because the heading is authored in the
spec, and may not require a *pinning test's* name, because that name pre-exists the register and belongs to
its suite. These twins are authored in this repository for this purpose, so requiring their shape sits on the
authored side of that line. Measured: 6 of 6 already use it, so the requirement describes the tree rather than
migrating it.

## Risks / Trade-offs

**The reaction checks form, and form can be satisfied without substance.** `expect_pass` can appear in a
comment; the backstop can be sourced and never invoked; `${1:-` can sit in an unrelated position. The reaction
is aimed at an author who *forgets* the shape, not one who *evades* it — every recurrence this window was the
former. Stating this is not a hedge: the three semantic bounds are exactly where substance lives, and they are
declared rather than approximated.

**Freezing a three-week-old shape may ossify a poor choice.** Mitigated structurally rather than by
confidence: the shape lives in one reaction and one projection, so revising it is one edit and one bless,
where today the same revision is a six-file sweep that has twice left a sibling behind. The projection also
makes the shape *visible*, which is the precondition for arguing with it.

**A required set is harsher than an allowlist, deliberately.** A gate added without a twin fails immediately
rather than at review. That is the intended trade: an allowlist of exceptions rots silently; a required set
fails loudly the moment reality diverges. The cost is that a legitimately unusual gate must argue its case as
a spec change, which is where such an argument belongs.

**The capability's own reaction is not in its own surface.** It is a Rust test, and the surface is `check_*`
shell gates, so nothing checks *its* shape. That asymmetry is real and accepted: the alternative is a
self-including shell gate, which D1 rejects for reasons that outweigh it, and the Rust reaction sits inside the
self-governance suite whose own shape is governed by `governance-dogfood` and `self-law-projection`.
