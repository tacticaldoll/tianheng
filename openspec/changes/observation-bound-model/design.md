## Context

A declared observation bound is this family's most consequential sentence: it reads as **permission**, telling
a future auditor that a shape which looks like an escape is governed policy. `observation-bound-register`
governs that every bound is enumerable, carries a citation, and is projected. It has never governed **what kind
of stop** a bound describes.

The place that information has been living is a one-word capture in the recognizer:

```
BOUND_HEADING='^#### Scenario: .*(stated|documented)( [A-Za-z-]+)? bounds?'
```

Every figure below was measured on 2026-08-06 at `c5174a6`, and the classification behind the model came from
reading each declared bound's WHEN/THEN rather than its adjective. The projection this change adds replaces
these figures; they appear here because they decided the design.

Sixteen distinct phrasings, and three results that made this a defect rather than untidiness:

- **`stated` and `documented` are a synonym pair.** Two specs use both bare forms internally; the same
  qualifier appears under both words. Roughly half the declared bounds use no qualifier at all.
- **One qualifier spans both sides of the false-negative line.** `cfg-blind` in
  `external-crate-confinement` marks a `#[cfg]`-dead import observed as live — reacting **more** than the
  truth. `cfg-blind` in `runtime-origin-assertion` marks a probe behind `#[cfg(test)]` counted as coverage, so
  a seam whose only production probe lives there reads as **probed** — reacting **less** than the truth, a
  false negative. Same word, opposite consequence, and the direction is the whole content.
- **A misclassification already cost a wrong urgency call.** The `#[cfg_attr(pred, path=…)]` entry predicted a
  false negative; reproduction found a constitution error (exit 2) — fail-loud, never silent. That entry's own
  lesson is "the risk class is what decides urgency".

## Goals / Non-Goals

**Goals**

- Move the classification from a free adjective into a type whose illegal states cannot be written.
- Derive the value set from what the declared bounds exhibit, and cite an instance for each value.
- Make a declared false negative name who owns closing it.
- Hold the specs' declarations and the code's in a checked bijection, keyed on the id the register already
  derives.
- Leave adopters with nothing to migrate.

**Non-Goals**

- The execution protocol. `Observer`, the runner's observer set, and the shell's `dyn` seam are the change that
  follows this one; this change exists so that its `bounds()` has something to delegate to.
- Verifying that a bound's rationale prose describes the real cause. Declared as a bound of this capability.
- Binding bounds to `Rule` instances so an adopter could query a boundary's bounds programmatically. That is a
  larger claim — bounds are declared per capability today, not per rule — and it is not made here.
- Moving the register's own projection generation out of shell. Rejected below.

## Decisions

### D1 — The model lives in 璇璣, and the reason is each crate's stated job

`星表`'s self-description: "the shared **declared-workspace-data substrate** … reads `cargo metadata` … the
tabulated catalog every observation dimension references **before it observes**". Its entire public surface is
eleven functions and zero types; it spawns `cargo` and returns `Result<Value, String>`.

`璇璣`'s: "the shared **reaction model** … the dimension-agnostic **vocabulary** … it holds the *measure*,
never the react itself." Its public surface is nothing but types.

A bound is a claim about where a reaction's **measure** stops, rendering no verdict. That is 璇璣's stated job
in 璇璣's own words, and the constraint 璇璣 declares for itself — measure, never verdict — is satisfied rather
than strained. 星表 is where the corpus is catalogued; the extent of measurement belongs to the instrument, not
to the catalogue.

**On the name collision, deliberately co-located.** `ScanDepth` already lives in `xuanji::model` and means how
far a scan *walks* — an adopter's knob on a boundary. This model means where the measure *stops*. Putting them
in the same crate makes the distinction impossible to ignore and forces both doc comments to draw it; hiding
one in another crate is how the confusion would survive.

### D2 — Nesting, not flags: the contradiction is a compile error

`Extent` is `OutOfReach | Reached(Reached)`. A shape the observation source never saw has **nowhere** to record
over- or under-reaction. A flat enum with a direction field admits "never observed it, and it over-reacts" —
which is precisely the shape of the `#[cfg_attr]` misclassification, written as a type.

Granularity is carried **only** by the as-intended value, not as a field on every extent. No declared bound is
both out of reach and granularity-limited; a model offering the pair on every value would invite a combination
nothing exhibits, and would dilute the nesting that makes the contradiction above unwritable.

**Rejected — a typestate builder.** It was the first design and it is closed in the wrong dimension. A chain of
inherent methods on a concrete type can be *called* from outside but cannot be *implemented* from outside, so
an adopter's own bound could never join the model. Nested algebraic data types deliver the same
unrepresentability while remaining a value any implementor can return. The following change's `Observer` trait
depends on exactly that.

**Rejected — an enum plus a hand-maintained `ALL` array**, on the `SeamKind` pattern. It is the right shape for
its own problem and the wrong one here, and the difference is *which side breaks*: adding a variant breaks the
consumer's `match`, while every existing declaration keeps its old classification and nobody re-examines it.
The enforcement would sit on the reader instead of the declarer. What this model needs — and what the following
change's trait supplies — is that adding a *question* breaks every declaration, while adding a new *answer* to
an existing question correctly breaks nothing. `#[non_exhaustive]` on these enums is that second half, and it
matches the crate's existing convention (`Severity`, `BoundaryKind`, `ScanDepth`, `Polarity`, `Outcome`).

### D3 — Seven values, read out of the declarations, and three that a tidier model would have merged

The set came out of the declarations, not out of a design — and classifying all of them **falsified this
design's first draft**, which is the strongest evidence that the derivation was real rather than decorative.
Two corrections, both now in the types:

**A seventh value was missing.** Three bounds are reached, correctly silent, and bounded in nothing at all:
`as _` binds no nameable path a consumer can reach, and a `mod` or a plain item in a function body is
unreachable as `crate::…`. They exist only so a reader does not misread the silence as an escape. The
granularity value could not hold them because it requires a bounded part, and folding them into *out of reach*
would have been the exact confusion this model exists to end — the reaction saw them and was right.

**One value has no live instance.** No declared bound refuses to judge; the only fail-loud-adjacent declaration
is the one *declining* to refuse. It is kept and the absence is stated, because the motivating
misclassification was precisely a confusion between refusing to judge and being out of reach, and a direction
that cannot be named cannot be predicted with.

Three are worth defending because merging them is the obvious simplification:

**Refusing to judge vs deliberately not refusing.** `semantic-trait-impl-locality` declares that a cfg-gated
module with an absent file "is skipped … rather than failing the gate with a scan error (exit 2)". That is a
declaration about *not* erroring, and its adopter consequence is the opposite of a fail-loud: one refuses to
give a verdict, the other gives one while stepping over something. A single "verdict-affecting" value would
make them read alike.

**Under-reacting is its own value and carries an owner.** It is the one direction this family treats as a
defect, so a declaration of one with nobody responsible is how it outlives its reason. The declarations already
distinguish three owners in prose no reaction can read: "inherited from the module scanner", "shared with the
semantic dimension", and "a false negative the adopter owns by narrowing". The owner is carried *only* here —
nothing is owed for a shape nothing observes by design, and an owner field everywhere would be decorative
wherever it is not load-bearing.

### D4 — The demonstrated direction is derived, never declared

An extent already determines what its pinning test must show: out-of-reach and under-reacting are defended by
a test showing the reaction does not fire; over-reacting by one showing it fires on a harmless shape; refusing
by one showing exit 2. A declared direction beside the extent would be a second copy of one fact, and two
copies can disagree. So it is a function of the extent.

This turns the classification from decoration into a **prediction about evidence**, which is what earns it a
place under the drift law: the type does not merely name a kind, it constrains what the defence must look like.

### D5 — Declarations are library items, not test items

Measured: no crate in this workspace has any dev-dependency, and every dimension's tests are `#[cfg(test)]`
modules inside `src/`. A `#[cfg(test)]` item is compiled only when its own crate is under test, so a
declaration living there is invisible to every other crate — and the bijection needs one reaction that sees all
three dimensions at once. Only the composed shell does.

This is also what the following change requires: an observer cannot declare its bounds as part of joining a run
if its declarations exist only under test.

The three crates owning declared bounds are the static, semantic, and runtime dimensions. `xuanji`,
`xingbiao`, and `tianheng` own none and gain no accessor — an empty one would be a name with nothing behind it.

### D6 — The bijection reaction is Rust, in the shell; the register stays shell

The reaction reads the specs and the code and asserts set equality. It lives under
`crates/tianheng/tests/` — the only crate that sees all three dimensions — following the
`TIANHENG_WORKSPACE_TESTS` discipline six crates already use, and it holds its projection fresh through the
`GovernanceTest` machinery that already blesses `AGENTS.self-law.md`.

**Rejected — moving `check_bound_register.sh` to Rust wholesale.** It is 1010 lines plus a 1120-line matrix,
with a stateful `awk` parser and `git ls-files -z` quoting handling, and it decides whether a cited test is
*registered* by running `cargo test -p <member> -- --list`. A Rust reaction doing that would invoke `cargo test`
from inside `cargo test` and contend for the build lock. That is an obstacle, not a preference, and it is
separable from this change.

So the register keeps its three existing constraints and loses only the qualifier slot from its recognizer;
this capability owns the new obligation. One responsibility each, no duplicated vocabulary — the failure mode
this change would otherwise reproduce is a second declaration of the value set in shell.

### D7 — Two projections, with distinct subjects

`docs/observation-bounds.md` projects what the specs *declare* — statement and citation per bound. The new
projection groups by **extent** and leads with the count of declared false negatives and their owners, for the
same reason the register's leads with its unpinned count: a number in a footnote is not read.

One document generated by two mechanisms would be worse than two documents with distinct subjects, and merging
them requires D6's rejected rewrite.

### D8 — The sweep closes the harmful half of the slot, and the asymmetry with detection is deliberate

Exactly half the declared bounds carry a qualifier and half carry the bare marker. Only the qualifier did
damage: it read as a classification, and one of them spanned both sides of the false-negative line. `stated`
versus `documented` carries no information either, but it misleads nobody, so it stays.

The limit is not tidiness. **A heading's slug is the bound's derived id**, so every removal changes an id, and
every in-tree `(bound: …)` reference to it must move — measured, every reference in the tree contains the marker
phrase, because the marker is part of the slug. Sweeping the harmful half halves that churn. The register's
existing reference-resolution reaction is what catches a missed reference, so the sweep cannot be partially
completed and read as done; what it cannot catch is a reference in a merged pull request body, and those were
always snapshots.

**The heading requirement tightens and the prose recognizer must not.** They look like one change and are
opposite acts. `BOUND_HEADING` states what an author must write, and requiring a form the spec owns is the same
legitimacy the register already claims for the heading convention. `BOUND_PROSE` is *detection* — the floor that
catches a bound stated in prose and never declared, and the gate's own header names it as what "stops the
register being completed by declaring only the convenient bounds". Narrowing it in step would stop it seeing any
qualified phrasing, which is a false negative in the register's own floor. The first draft of this design did
narrow both.

Refusing a qualified heading is also an **explicit** refusal rather than a non-match. A heading the recognizer
simply fails to match is not read as a bound at all, so it would fall through to the prose direction and be
reported as an undeclared bound — a true failure with a misleading message and the wrong repair.

### D9 — One derivation of the id, held by an equality assertion rather than by care

The id is the heading's slug: lowercased, runs of non-alphanumerics collapsed to one hyphen, ends trimmed. A
Rust reaction needs that id, and writing the rule a second time is the divergence this file has already paid
for — `check_bound_register.sh`'s own comment records that two matchers decided one question until "a shell
`grep -qE` whose whitespace class differed" cost this window a review round.

So the reaction derives the id **and** asserts its derived set equals the id set in the shell-generated
projection. One assertion covers both failures at once: a slug rule that has drifted from the shell's, and a
projection that is stale. Reading the projection alone was considered and rejected — `cargo test` runs before
the register gate in the Definition of Done, so a stale projection would let the bijection pass while the specs
and the code disagreed, and the eventual failure would name staleness rather than the bijection.

### D10 — The projection-freshness machinery is generalized, not copied

`GovernanceTest::assert_projection_fresh` renders `constitution_markdown(&self.constitution)`: it is
constitution-specific, not a general bless-and-diff. Measured by reading it, after this design's first draft
claimed the new projection could ride it unchanged.

The bless rule is therefore **generalized once** — an additive method taking already-rendered content, with the
existing method delegating to it — rather than reimplemented in the new test module. The tree already carries
two bless implementations, one shell and one Rust; a third would be the same duplication D9 refuses, in the
mechanism whose whole purpose is to stop documents drifting.

## Risks / Trade-offs

**A classification is still authored, and an author can classify wrongly.** The type stops contradictions and
the derived direction constrains the evidence, but nothing verifies that a bound labelled over-reacting really
over-reacts. That residue is declared as a bound of this capability rather than implied away. What the model
does buy is that the *wrong kinds of wrongness* — a contradiction, a missing owner, a direction that disagrees
with its extent — stop being possible.

**Sixteen headings must be swept.** Mechanical, and the register's tightened recognizer fails on any that is
missed, so the sweep cannot be partially completed and read as done.

**The model is derived from today's declared set, so a genuinely new stop needs a variant.** Accepted, and
`#[non_exhaustive]` makes adding one a minor rather than a break. The alternative — a general escape value —
would restore the free-text slot this change exists to close.

**The public surface grows on a crate whose minimalism is deliberate.** Mitigated by scope: types and one
accessor per owning dimension, no trait, no constructor DSL, no rendering. The trait arrives with the following
change, where it earns its place by enforcing declaration on third-party observers.
