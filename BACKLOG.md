# Backlog

Forward-looking work, deliberately deferred. Promote an item to an OpenSpec change when
you pick it up. Every future reaction obeys the drift law:

> **No drift type without an observation source. No target type or name without a
> reaction.**

Nothing here is "designed" yet — reaction *phases* with their observation sources named,
not APIs. A new observation dimension is **a crate, born when it is built** (never a
pre-created empty stub); the heavy dependency it needs is quarantined to that crate so the
`guibiao` core stays `serde_json`-only.

## Backlog governance — evidence before promotion

The live backlog is a decision surface, not a promise that every recorded idea will ship. Before a
live item is promoted, it must name: **class**, **observed pressure**, **observation source**,
**current reaction or bound**, **risk**, **promotion trigger**, **version class**, and **authority**
(the spec, project decision, or code/test evidence that owns the claim). Classify it as:

- **READY-PATCH** — supported pressure with a concrete source, and the correction preserves the
  published API and current baseline/report identity wire.
- **DESIGN-BREAKING** — a supported problem whose honest solution needs a public or wire migration.
- **WATCH** — plausible pressure without enough adopter, second-consumer, or correctness evidence.
- **ACCEPTED DEBT** — a known, bounded risk whose current reaction or documented coverage bound is
  intentionally sufficient.
- **DECLINED** — a considered direction rejected for a recorded reason.
- **BUILT / HISTORY** — shipped context retained only where it explains a live contract or trigger;
  requirements live in [`openspec/specs/*`](openspec/specs) and settled rationale in [`PROJECT.md`](PROJECT.md).
  Detailed historical ledgers for 0.1.x – 0.3.0 are archived in [`docs/history/0.1.0-0.3.0-built-ledger.md`](docs/history/0.1.0-0.3.0-built-ledger.md).

A **closed** item leaves the live class it was filed under; it does not stay there struck through. Its
reproduction record moves to *Closed — reproduction records* below, because a class heading is read as a
queue and an entry that carries a question and its answer at once is a reader trap.

## Open defect queue

No live queue currently. The `v0.2.3..release/0.3.1` adversarial sweep that populated this section
is fully closed: every finding reached a terminal state — 11 fixed (each its own `change/*` + PR,
cited in the affected code's history and `CHANGELOG.md`), 2 verified moot, 2 refuted, 6 promoted to
live decisions below, and its own prior two-round sweep's 6 refuted / 6 upheld-by-only-one-lens
findings absorbed into `DECLINED`/`WATCH` below (condensed deliberately rather than kept verbatim,
to avoid a future reader anchoring on a raw agent-verdict's specific phrasing instead of the settled
conclusion). The sweep's own working queue file is not retained once fully drained — its substance
now lives here and in the closing PRs, not in a file kept only because it once existed. A future
sweep gets its own dated `docs/audit/*.md` queue file and its own pointer here.

## Live decision index

### DESIGN-BREAKING

**Empty.** Every item that stood here when the 0.4.0 window opened is closed; each one's reproduction
record is kept under *Closed — reproduction records* below, out of this queue so the index cannot read as open
migrations. Nothing currently requires a public or wire migration.

The `xuanji` sink for shared run/projection vocabulary remains *classed* DESIGN-BREAKING while sitting
under WATCH — its promotion trigger (a second standalone instrument consumer demonstrating the same
projection/run need) has not fired, and acting on it speculatively would break the proven standalone 圭表
consumer for an undemonstrated deduplication.

### READY-PATCH

- **The bounds-method reader anchors on a whole-line occurrence that is not the definition.** *Class:*
  READY-PATCH. *Observed pressure:* the reader requires the signature to occur exactly once and at a line
  start, and knows nothing of comments or literals. So where the definition has moved out of the inspected
  file, any surviving **whole-line** copy anchors — reproduced with a block-comment copy, and again with a
  copy inside a `&str` constant, both giving
  `every_observer_declares_exactly_its_dimension_s_bounds ... ok`. *Observation source:* those two
  perturbations, run during the closing review of the 0.5.0 window.
  *Current reaction or bound:* the declared bound
  `observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound`.
  *Risk, measured rather than assumed:* **narrower than it first reads.** A *divergent* second list does not
  pass — `observation-bound-model` reads every dimension through `Observer::bounds` and holds a bijection with
  the specs, so a difference in membership or content fails `the_extent_projection_is_fresh` and the
  classification test one capability over. What passes is a second, hand-maintained path that **agrees today**
  and is maintained by hand from now on: re-run with a list rebuilt element by element from
  `observation_bounds()`, the whole workspace suite is green. *Promotion trigger:* fired; both perturbations
  are tree artefacts rather than reports. *Version class:* patch; a `tests/` reaction of this repository,
  shipping in no crate. *Authority:* `observer-protocol`.

  *Shape, with the corpus measured rather than borrowed:* comment stripping does **not** close this — a string
  literal is not a comment — so the register's rejection of comment-delimiter lexing is not the reason here,
  and citing it was wrong. This reader's corpus is the three files `DIMENSIONS` names, 35, 48 and 43 lines,
  none carrying a string literal with a comment delimiter, so the register's measurement does not transfer in
  either direction. Two candidate closures, neither adopted: require the anchor to be preceded by an
  `impl Observer for` line, which refuses both reproduced perturbations, needs no lexing, and declines strictly
  more — this reader's declared error direction; or stop reading a file the dimension table names and read the
  definition the compiler resolves, which is a different instrument than any textual condition tried here. The
  first is a further narrowing, and every textual narrowing of this recognizer's retired sibling was defeated,
  which is why it is recorded for the next author to weigh rather than applied inside a closing review.

- **The construction-held list is hand-maintained prose.** *Class:* READY-PATCH. *Observed pressure:*
  `observer-protocol` requires the spec to say which dimensions' equality holds by construction, and nothing
  observes that the list is correct. The 0.5.0 window is the evidence: the list named runtime alone, the shell's
  semantic arm changed under it in the same window, and the list was repaired **by hand**. Falsifying it — say,
  claiming static is construction-held and runtime observed — passes the whole workspace suite and every gate.
  *Observation source:* the final sweep of that window, which ran exactly that perturbation.
  *Current reaction or bound:* the declared bound
  `observer-protocol/whether-the-stated-construction-held-list-matches-the-composition-path-is-not-observed-a-stated-bound`.
  *Risk:* a reader takes a constructed equality for a measured one — the failure the requirement's own sentence
  exists to prevent, in the sentence that prevents it. *Promotion trigger:* fired; the list went stale inside
  the window that wrote it. *Version class:* patch; repository-internal, shipping in no crate. *Authority:*
  `observer-protocol`. *Shape:* not a text reader — that route is retired and its defeat is recorded one
  requirement over. The discriminator is behavioural and needs a **perturbed build**: empty a dimension's
  observer and see which assertion fails — the equality assert for an independently-implemented dimension, only
  the reacts-at-all assert for a construction-held one. `crates/kanhe/tests/pin_bites.rs` already builds and runs a
  mutated checkout, so the machinery exists; what it does not yet do is carry a declaration whose subject is a
  spec sentence rather than a pinning citation.

- **`observation-bound-model`'s projection discloses its own bounds by a typed list; its sibling requires a
  derived one.** *Class:* READY-PATCH. *Observed pressure:* `gate-shape-contract` hit this and wrote the
  requirement — *"That disclosure SHALL be **derived from the specification, not typed into the generator**,
  and held to it in both directions"* — with `the_projection_discloses_every_declared_bound` holding it and a
  vacuity guard. `observation-bound-model` has the same shape and only the weaker requirement (*"The projection
  SHALL state what it does not claim, in its own header"*), so its "what this document does not claim"
  paragraph enumerated members as a literal in the generator's template, where the freshness check compares
  that text with itself. *Observation source:* the final sweep of the 0.5.0 window, which found the paragraph
  naming two limits while the capability declared three; the enumeration is removed from the template as an
  immediate stop, so what remains is the missing requirement rather than a live falsehood. *Current reaction or
  bound:* none — the projection now points at `docs/observation-bound-extents.md` instead of listing.
  *Risk:* the same class the sibling already paid for, in the one place a freshness check structurally cannot
  see. *Promotion trigger:* fired — the sibling's requirement exists and this one's absence was measured.
  *Version class:* patch; repository-internal, shipping in no crate. *Authority:* `gate-shape-contract`, whose
  requirement is the shape to copy, and `observation-bound-model`, which would carry it.
  *Shape:* lift the derived-disclosure requirement into `observation-bound-model`, and give it a holder
  mirroring `the_projection_discloses_every_declared_bound` — declared headings read from the spec, held
  set-equal to what the projection discloses, both directions, with the empty-enumeration guard.

- **Most pinning citations have never been seen to fail.** *Class:* READY-PATCH. *Observed pressure:* the
  register decides a citation names a test that RUNS and cannot decide that it BITES; gutting a cited pin's body
  in a worktree left the suite green and the register clean. `crates/kanhe/tests/pin_bites.rs` closes that for the citations
  that declare a mutation, and it prints how many do not on every clean run — the figure is produced there, not
  typed here. *Observation source:* that gutting, and the anchor-counting rule in `observer_protocol.rs` losing
  its only assertions during the composition-body retirement, found by a reviewer reading the diff.
  *Current reaction or bound:* `crates/kanhe/tests/pin_bites.rs` over the declared mutations; nothing over the rest.
  *Risk:* a defence that has stopped defending is indistinguishable from one that has not, which is the failure
  the register was built to end one level down. *Promotion trigger:* fired — the gate exists; what remains is
  coverage, which grows one considered record at a time. That last claim was false while the tree under test
  was an export of tracked content: a pin reading the repository through git failed its own control run, so no
  record could ever exercise it — `units_outside_the_gate_pairing_are_outside_the_surface` was one. The tree is
  a detached worktree now and the claim holds. One citation is still outside it for a different reason:
  `a_cfg_gated_module_with_no_file_is_skipped_not_errored` is defined in two files under `crates/`, so the
  target to run it in cannot be derived from a set and any record naming it refuses. Both episodes are kept
  because the entry's economics rest on the claim. *What closing it costs, measured while seeding:* a
  mutation must genuinely perturb the pinned point, and authoring one is per-bound expert work. One attempt
  during this change did not — masking a brace inside a block comment left the exact one-statement comparison
  refusing the body anyway, so the pin held and the record reported a biting pin as a dead one. That direction
  is safe, and it is why coverage cannot be swept. (Two further failures that looked the same were gate defects,
  not authoring cost: a lib test registering under its module path, and the cargo target derived from the
  mutated file rather than from the test's definition. Both are fixed and neither recurs.) *Version class:* not release-affecting; a
  repository gate over this repository's own governance tests. *Authority:* `observation-bound-register`, whose
  added requirement states the obligation and the arrangements that make it observable.

- **The shell's semantic delegation, held by construction.** **Still open**, and one attempt at it is recorded
  here because the attempt's own reasoning was wrong. The shell's semantic arm now invokes `SemanticObserver`
  rather than calling 渾儀's composed entry point beside it, which was this entry's named shape — and review
  measured that it closes nothing here: writing the bound's exact WHEN into the arm, a guard deciding
  emptiness itself before delegating, leaves the whole suite and every gate green. What it *did* close is the
  two paths' **equality** for this dimension, which is a different property and is now construction-held. The
  runtime precedent does not transfer: that arm held a second implementation of the corpus derivation, the
  audit call and the `cannot read workspace` message, and delegating collapsed its two copies into one, whereas
  the semantic arm always had one implementation with two callers. Closing this one needs the shell's semantic
  outcome to be *unreachable* except through the observer — a shape where the guard stops compiling — which is
  a design step, not a call-site swap. *Class:* READY-PATCH. *Observed pressure:* the
  source-shape reaction that claimed to observe it is retired in this window after four review rounds each
  defeated the narrowing before it — by name resolution, by the parameter's binding site, by which definition is
  the subject, by the caller frame, and by execution, which no reading of text reaches. `observer-protocol`
  declares the resulting gap as an unpinned bound owned by the engine, and it is the tracker for this entry.
  *Observation source:* that bound and the retired reaction's history on `change/refuse-ambiguous-delegation-extent`.
  *Current reaction or bound:* the declared bound; no reaction. *Risk:* the shell grows a second semantic
  behaviour owner and nothing says so — the drift a seam exists to end. The dimension's *equality* is
  construction-held since this window; its *delegation* is the seam that is not, and the two are one word
  apart. *Promotion trigger:* fired; the bound is declared unpinned, which the register leads with.
  *Version class:* patch if the composition is restructured without moving a public signature; minor if the
  shell's entry point changes shape. *Authority:* `observer-protocol`, whose spec states both the obligation and
  the retirement. *Shape:* **not** the runtime route, which this window
  tried and measured wrong: invoking the observer makes the *equality* construction-held and leaves an
  independent shell decision as writable as before, because the runtime arm had a second implementation to
  collapse and this one never did. What would close it is the shell's semantic outcome being unreachable except
  through the observer — a shape in which the guard stops compiling rather than one in which it merely has
  nowhere tidy to sit.

- **Every normative SHALL either has a reaction or is a declared bound.** *Observed pressure:* **ten** found
  and closed through re-review #3 of the 0.5.0 window. The first seven were the family's declarations staying
  literal, the equality fixture reacting in every dimension, an observer declaring exactly its dimension's bounds
  (a comparison that was `f() == f()`), the protocol's required `bounds` method having no consumer at all, *an
  audit finding carries no repair polarity*, *an observer's bounds method cannot be found where the reaction
  looks*, and *joining a run would require no new export*. Re-review #3 added three more: a requirement prescribed
  wording for the projected shell reason that no reaction observed, a reason-only correction claimed verdict
  stability through an impossible-to-fail scenario, and the shell's semantic delegation scenario could not detect a
  duplicated local guard. Every one was found by hand. *Observation source:* the window's closing review, rounds
  1–7 and re-review #3; the first seven are recorded in `CHANGELOG.md`'s `[Unreleased]` with the perturbation that
  proved them.

  **Promoted from WATCH because its own trigger fired, six times.** That trigger was *a normative SHALL found
  un-reacted **after** this window's sweep* — the sweep being the control, so the four found before it could not
  stand as evidence for themselves. Rounds 6 and 7 then found three more, and **all three were requirements this
  window had just written**: a SHALL added in one change and left unreacted, in the window whose whole subject was
  closing that class. That is stronger evidence than the original four, because it shows the class reproducing under
  authors actively watching for it. Re-review #3 then found the fourth through sixth post-sweep recurrences, again
  in requirements or scenarios written in the same window; two were removed as inert, and semantic delegation
  gained a source-shape reaction.

  *Current reaction or bound:* none. Only a **bound** carries a `PINNED-BY`; an ordinary requirement is bound to
  nothing, so no gate can tell a SHALL with a reaction from one without. *Risk:* the class recurring and being found
  by hand or not at all — a normative rule nothing enforces is indistinguishable from one that is enforced, which is
  the failure the bound register was built to end one level down. *Measured before promotion, not estimated:* the
  specs held
  **1048** `SHALL` occurrences across **310** requirements and **1177** scenarios. The register, by contrast,
  currently holds **78 bounds across 23 capabilities** — a live figure rather than part of the measurement
  above, written in that exact form because it is the one phrasing
  `crates/kanhe/tests/bound_register.rs` reacts to, and a census in any other wording is what that gate's own policy says must
  not exist in prose. A citation per SHALL would add on the order of a thousand hand-maintained pointers, which is
  the drift class this family already refuses. *First step, and why it is not simply "add a gate":* the binding must
  be **derived**, not declared — which test defends a requirement is nowhere written, so the honest first move is to
  find a derivation (a naming convention between requirement and reaction, or a reaction that enumerates what it
  covers) rather than to require an annotation. That is a capability to design, and designing it inside the closing
  review of a window would be the same haste this entry documents. *Version class:* not release-affecting; a new
  capability with its own gate, preserving every published API. *Authority:* `observation-bound-register`, which
  solves the same problem for bounds and is the shape any answer here would have to generalize.

  *Interim discipline:* `AGENTS.md` now requires a scenario entering main specs to name an existing reaction in
  the same change or arrive with a new guard and its negative run; a construction-guaranteed property stays in
  requirement prose instead. This does not close the entry — review convention cannot derive the missing binding —
  but it prevents sync from knowingly admitting another un-reacted scenario while the derived capability is designed.

- ~~**`crates/kanhe/tests/release_coherence.rs`'s subshell reads have not been audited for a swallowed status.**~~
  **CLOSED** in the open window. Audited read by read rather than swept: four of the five consumers already
  carried a vacuity guard (the release spine, the crate-manifest set, and both example-pin counters), which
  is why the entry's MEDIUM risk did not materialise as a false clean. The fifth — the internal-pin loop —
  had none, so a reformatted `[workspace.dependencies]` table iterated zero times and the direction passed
  having asserted nothing about any pin; it now refuses with `2`.

  The audit found two things the entry did not predict, both worth keeping. The gate collapsed **violation**
  into **cannot judge**'s absence: every refusal was `1`, including a shallow clone, an absent manifest, and a
  moved crate layout, which the family contract forbids and this gate's own header (written one change
  earlier) already claimed it did not do. And the exit-contract backstop had **inverted** it — `fail` was a
  `return 1` relying on `set -e`, so the `ERR` trap turned every genuine incoherence into `2`. Neither was
  visible to CI, because this matrix asserted a non-zero status rather than a code. Every gate matrix now
  assert the code, which is the property that would have caught it.

- ~~**Two gates have no companion failure matrix.**~~ **CLOSED** in the open window. Both now have one, and
  both were worth building rather than recording. `crates/kanhe/tests/whitespace_hygiene.rs` is the only fixture that pins
  the exit-contract backstop's subshell misfire — removing that guard fails it and no other matrix.
  `crates/kanhe/tests/dod_coherence.rs` closes the gate whose subject is a claim `AGENTS.md` makes about **itself**, so a
  reaction nobody had watched refuse was all that stood behind it; its zero-commands direction is the one that
  matters most, since without that guard the gate reports `ok: every local Definition of Done command (0
  parsed) is run by CI` and exits 0. Each gate gained the target-directory argument that makes a fixture
  possible, the same argument the register and reference-integrity gates already took.

  Every `check_*` gate now has a `test_*` twin, and every one of those matrices asserts the expected exit
  **code** rather than a non-zero status — the property whose absence let a 1-into-2 collapse ride green
  through CI in the release-coherence gate. The count is deliberately not written here: it is printed by
  the retired gate-shape projection, whose reaction enumerates the pairing.

- ~~**An observer cannot be made to declare what it does not observe.**~~ **CLOSED** in the open window, and
  worth recording for what it did **not** buy as much as for what it did.

  `xuanji::Observer` has two methods and no default body on either, so a participant cannot be composed into a
  run without declaring its limits. The promise stops being a convention this family keeps about itself and
  becomes a property of the type — including for a third party this family never reviews. `tianheng::Run` folds
  eagerly, and 圭表, 渾儀 and 漏刻 each implement it, so it is dogfooded rather than offered.

  **What it does not buy, and this entry should not be read as claiming it:** the obligation is to *declare*,
  never to declare *completely*. A participant may answer with a partial list, or an empty one, and no reaction
  can enumerate the limits of a reaction it did not write. The fold likewise composes verdicts and does not
  adjudicate them. Both are declared bounds of `observer-protocol` rather than gaps left to be discovered.

  Two design decisions were **reversed by measurement**, both recorded because each looked principled. A third
  method — "identify your boundary kind" — was dropped because nothing reacts to it: a `Violation` already
  carries its kind, so restating it would be a second copy of one fact. And the heterogeneous set was to be a
  declared `dyn` exposure in the shell; measured, no module of `tianheng` is governed by a semantic boundary and
  the `dyn`-trait DSL has no allow-except form, so the declaration would have been a name with no reaction. The
  eager fold removes the exposure instead, and a grep-based assertion keeps it removed because 渾儀 is not
  watching that crate. **The lesson: a boundary declaration must be checked against the DSL that would have to
  carry it, not only against the architecture that motivates it.**

  A side effect worth keeping: the corpus-and-anchor derivation the shell computed for 漏刻 now lives in 星表,
  the single reader of truth, because a runtime observer would otherwise have derived it a second time — and it
  is **baseline identity**, which is precisely the twin drift that crate exists to prevent.

- ~~**The list of generated documents is prose, and nothing checks it.**~~ **CLOSED** in the open window as the
  `projection-register` capability, and filed here in the same change that closed it because the trigger was that
  change's own last step: closing `gate-shape-contract` added a fourth projection **and** a hand edit to
  `AGENTS.md` to mention it, with no reaction behind the edit.
  `crates/kanhe/tests/projection_register.rs` enumerates the generated documents from the marker each carries,
  holds a two-way correspondence with the reactions blessing them, requires each to be named in `AGENTS.md`'s prose
  rather than inside a fence, and includes itself. Measured before proposing: 4 of 4 documents already carried the
  marker and named their generator, and the holders already agreed 4 for 4, so it described the tree.

  **Residual, declared rather than solved.** Two mechanisms are recognized — the shared Rust rule, and a `check_*`
  gate writing under `BLESS`. A document generated a third way whose author also omitted the marker is absent from
  both sides of the correspondence, which then holds over a surface missing a member. That is a declared false
  negative owned by the engine, not a limit of the corpus: the third mechanism's source sits in the tree the
  reaction already reads. Promotion trigger for closing it: a second generating mechanism actually existing. There
  is one today. The other residual is that a stated regeneration command is registered and never run, because
  running it means re-entering the harness already running or letting the reaction write into the tree it judges —
  `OutOfReach`, and not closable without giving up one of those two properties.

  **The lesson worth keeping is about reactions whose subject is text.** This apply hit self-reference three times:
  the specification quoting the marker it requires, the reaction's own source naming the signature it excludes, and
  the register having to be blessed twice to see itself. Every one was found by running it, none by reading it, and
  the fix was the same each time — recognize by position or shape, never by the bare string.

- ~~**A gate's own shape is convention, so every new gate re-learns it by breaking.**~~ **CLOSED** in the open
  window as the `gate-shape-contract` capability, whose reaction enumerated the
  tracked shell units under `scripts/` whose basename began with `check_` — on the basename rather than by a
  `check_*` pathspec, since git matches pathspec wildcards without `FNM_PATHNAME` and the glob would be
  describing something other than what it says — pairs each gate with the twin its basename names, asserts every
  checkable property, declares the three semantic classes and the coverage limit as observation bounds with
  pinning tests, and projects the retired gate-shape projection. A Rust reaction rather than a seventh shell gate,
  because `PINNED-BY` resolves only a harness-registered Rust function: a shell-defended capability would have
  landed those bounds `UNPINNED` and moved the register projection's leading figure off zero. It rides the
  existing `cargo test` line, so it added no Definition of Done entry and no CI step.

  **This entry's own claim was wrong, and worth recording as such.** It said the per-gate property table "is
  uniform **today** and enforced **nowhere**"; the second half held and the first did not. Measured while
  applying: the silent-clean-run assertion held in **2 of 6** twins — four grepped a clean run's stderr for the
  backstop's own diagnostic, which catches the one line it names — and the unchanged-repository assertion held
  in 5 of 6 by name and, once observed, in **none** of them, because every form captured `before` from a
  repository the gate had already run over, so a gate writing the same file on every run left it in `before` too.
  A stray write injected into a gate passed that direction unnoticed. The entry was wrong because it counted the
  properties it had just finished creating: the shape it described as settled was three weeks old — probed at
  `v0.4.0`, the backstop was installed in 0 of 4 gates and fixture-addressability in 1 of 4 — so the entry was
  reading its own recent work as convention. **The lesson: a promotion trigger written from inside the code that
  lacks the observation describes building the property, not finding it; measure the exact observation source
  before writing a figure into a trigger.**

  Both revisions the parked proposal needed were applied before it landed: the two qualifier phrasings left the
  heading slot, since what kind of stop a bound describes now belongs to `xuanji::Extent`; and the publish-time
  membership exemption left the bound mechanism entirely, because a bound says a reaction stops at a shape while
  an exemption says one named instance is excused from a requirement. **Residual, declared rather than solved:**
  nothing enumerates policy exemptions. The register enumerates bounds; this exemption is checked live by the
  reaction and named in the projection, and that is all. Trigger for giving exemptions their own register: a
  second instance. This is the first.

- ~~**A swallowed subshell status was repaired nine times and could be written a tenth.**~~ **CLOSED** in the open
  window, and closed as a *shape* rather than as a tenth repair. `gate-shape-contract` gained an eleventh property:
  a gate may not consume an observation source through `< <(producer)` whose producer can fail.

  **Both failure directions are measured, which is what made the class worth a property rather than a habit.** A
  `git ls-files --eol` truncated after one clean row made `crates/kanhe/tests/whitespace_hygiene.rs` report
  `whitespace hygiene ok (1 tracked text files)` at **exit 0** over a repository it had read one file of — the count
  fell from two to one in its own output and nothing reacted to it. A `git log` truncated the same way made
  `crates/kanhe/tests/release_coherence.rs` conclude snapshot state and report `[Unreleased] must be empty` at **exit 1**, a
  violation invented from a partial read. A vacuity guard reaches neither: it was built for zero rows, and a partial
  read gives one or more. **The guard and the capture answer different questions, which is why they now sit side by
  side rather than one replacing the other.**

  Eight sites migrated to the shell era's shared capture library, and **the property found two more on its first run — one of them
  written by the migration itself** (`< <(sort …)`), the other a per-file scan never migrated at all. A property that
  catches its own author's fresh mistake within a minute of existing is the argument for having it.

  **The lesson worth keeping is about the helper, not the sites.** Its first version turned `grep`'s exit 1 — a clean
  miss, the ordinary case — into cannot-judge, and the release gate's own vacuity direction failed immediately. So it
  takes `--ordinary-empty <status>` per call site: the rule is about the producer's *contract*, not about its name,
  and the shell era's shared exit-contract backstop draws the same distinction for the same reason. A shared helper that decides a
  producer's contract for its callers would have been a new class of its own.

  **Residual, and the bound was narrowed rather than left standing.** A status swallowed by a command substitution,
  or by a pipeline's non-final stage, is still unobserved — detecting either would mean modelling whether the caller
  reads `$?` afterwards, which is control flow rather than text. The declared bound now says exactly that, with its
  heading untouched because the slug is its id. Also unreproduced: the `git ls-files | grep -q` SIGPIPE shape, which
  needs enough data to fill a pipe buffer; migrated with the class and recorded as migrated without a demonstrated
  negative run. Trigger for revisiting: a swallowed status found in a shape the property does not reach.

- ~~**The pre-publish gate had no specification, and its stated bound had the cause backwards.**~~ **CLOSED** in
  the open window as the `publish-source-integrity` capability. Found in the 0.5.0 pre-release review, and worth
  keeping for the shape rather than the fix.

  All 34 specifications were searched: **none** stated that a publish must come from a signed annotated tag at
  the tip of `main`. The gate standing before the one irreversible act carried its contract in its own header
  comment, while `gate-shape-contract` exempted it from Definition-of-Done membership *by name* — the one place a
  reader was told it is special. A reaction with no requirement is the mirror of the class this window kept
  closing, and it has a consequence: there was nowhere to declare a bound.

  **The bound's stated cause was wrong, and that is why the defect survived.** The header said the signature could
  not be verified because verification needs an allowed-signers configuration CI lacks. Measured with no such file
  anywhere, `ssh-keygen -Y check-novalidate` verifies validity; only **attribution** needs the file. So the gate
  matched a shape — and accepted an unsigned tag whose message quoted a signature block. **The lesson: a bound's
  cause is what the next author reasons from. This one said "you cannot check this", so nobody tried.** A bound
  with a wrong cause is worse than one with no cause at all.

  This is the second live instance in one day of `observation-bound-model`'s own declared bound that *a
  declaration's stated cause is the real cause is not observed*. Two instances is the number worth naming:
  the extent is typed and checkable while the rationale is prose the model never reads, and both instances were
  found by a human-style review rather than by any reaction. Trigger for revisiting that bound: a **third**
  instance, or a proposal for a rationale check that is not a keyword heuristic.

  Also recorded: `PINNED-BY` resolving only a Rust function meant this shell gate's bound could be defended by a
  twin direction and cited by nothing. Rather than accept `UNPINNED` — which would have reported a defended bound
  as undefended, buying a true figure with a false fact — the release-repository fixture builder was extracted to
  `crates/kanhe/src/publish_source_gate.rs` and a Rust test pins the bound through it. **Measured, not estimated**: one
  shared builder, no second construction. So the WATCH entry on shell-defended capabilities stays WATCH; its
  trigger asks for a residual measured and found *unaffordable*, and this one was affordable.

- ~~**The 天衡 shell's baseline-writing and CLI surface has never been swept.**~~ **CLOSED** in the open window,
  swept against an enumeration built first as the entry required, and it found **two defects — both in the
  requirements rather than in the code.**

  **The enumeration, and what it measured.** The baseline write path's twelve filesystem operations, read out of
  the code rather than guessed: `canonicalize` on the target, the mode read, the `O_EXCL` temp create, the
  temp-plant loop, `fchmod` on the *descriptor* rather than the path, `sync_all` after the mode and before the
  rename, the rename, the parent-directory flush whose error is deliberately discarded, the guard's cleanup on
  drop, and on the create path `create_new`, the `symlink_metadata` dangling-symlink diagnosis and `read_link`.
  Every one already has a test in `crates/tianheng/tests/baseline_cli.rs`. Five further adversarial shapes were
  probed live — absent parent directory, target is a directory, read-only parent, unreadable target, symlink to a
  directory — and all five exit 2 naming the path and the OS cause, with no silent success and no misdiagnosis.
  The create-versus-overwrite decision takes the create path on `NotFound` **only**, so an unreadable existing
  baseline cannot be misreported as a creation race. On the CLI side, twenty-five cells: four value-taking flags ×
  {missing value, empty, flag-shaped, empty in the equals form, repeated}, plus the boolean flags, the unknown
  flag and positional, and the `--format` value. **Swept and defended, except for one column.**

  **What it found.** `list`'s refusal named no flag — one sentence for all five check-only flags — while the
  requirement covering the same conflict inside `check` cites `list`'s rule as the one it extends *and* requires
  the flag to be named. Each implementation satisfied its own requirement, which is why no test caught it. And
  `list`'s requirement enumerated four check-only flags while the runner rejected five: `--disallow-stale` was
  added to the code and not to the prose. Corrected by tightening the requirement and deriving the set, so a sixth
  flag is covered the moment it exists.

  **Two of the sweep's own measurements were wrong before they were right**, recorded because the correction is the
  transferable part. The first CLI probe grepped the whole output for the flag name — and the `usage:` banner lists
  every flag by construction, so every cell measured as naming its flag, including a nonexistent one. The second
  read the `error:` line only. Separately, `list --format sarif` looked like it shared the generic message until
  the probe stopped passing `--manifest-path` alongside it, which was tripping the check-only guard first. **A
  probe that carries an unrelated flag is measuring that flag.**

  **Residual: the enumeration is hand-made and will rot.** It is a snapshot taken against a named revision, in a
  proposal that dissolves at sync, and nothing enumerates the CLI surface as a reaction — which is how this
  requirement's own list of four went stale in the first place. Not solved here: a register over six flags would be
  ceremony this finding does not justify. Promotion trigger: a **second** defect found in this surface, or a flag
  added to the runner without a test that names it.

- ~~**The 渾儀 seam-identity and owner-qualification surface has not been swept against the bound index.**~~
  **CLOSED** in the open window, swept and found defended — by a structurally enforced enumeration rather
  than by diligence, which is why the result is worth recording instead of just noting "no findings".

  `every_public_seam_shape_is_named_and_identity_injective` carries the enumeration and forces it to stay
  complete: `seam_kind` matches all eleven `PublicSeam` variants with no wildcard, so a new variant cannot
  compile without a representative; `SeamKind::ALL` is asserted duplicate-free; the observed shape set must
  **equal** it; the shape-to-label mapping is a bijection checked in **both** directions, so a new variant
  folded into an existing shape cannot read as covered; each seam's field schema is asserted exactly; and
  `keys.len() == seams.len()` asserts the injectivity the name claims. Thirty-eight further
  distinctness tests cover the owner-qualification half across the family.

  **ACCEPTED DEBT, stated by that test itself**: the *content* of each representative stays hand-maintained.
  A representative whose field values do not actually differ from a sibling's in the distinguishing field
  would satisfy the set equality and the bijection while proving nothing about that field. The test names
  this as a judgment no structure can force, and it is accepted rather than closed because the alternative —
  generating representatives — would need a model of which field distinguishes which shape, which is the
  judgment itself. It is not a bound and does not belong in the register: it limits a test's completeness,
  not an observation.

  Lesson kept, because it recurred four times in this window: a gap suspected from a partial view dissolves
  on the full one. The injectivity assertion here was read as absent from a fifteen-line excerpt and sits on
  the sixteenth. Read the whole construct before reporting it, and diff what a change actually moves rather
  than reasoning about what it should.


- ~~**An inherited observation bound is RESTATED by each capability that inherits it, so one behaviour
  change leaves several specs stale at once.**~~ **CLOSED** in the open window. Closed by a reaction plus
  the repair it forced, not by choosing one of the three candidates the entry listed: they were framed as
  alternatives and are not — a reaction detects the restatement and the repair resolves it, and only the
  reaction stops the next one accumulating silently.

  The register made it measurable on its first projection: two behaviours were declared as bounds in three
  capabilities each, all six declarations citing one test. `crates/kanhe/tests/bound_register.rs` now fails when a test
  is cited by declared bounds in more than one capability, keyed on the shared citation rather than on
  statement text — two declarations of one behaviour do not have identical prose, so text similarity would
  be a heuristic where a shared citation is a fact. Repetition within one capability is not a restatement
  and does not fire it.

  The repair sank the declaration to `semantic-signature-coupling`, which its own spec already identifies as
  the owner by stating the anchor-and-item property on every single-module-anchored capability's behalf, and
  the other two now reference it. **Raising a shared surface into its own capability was rejected**:
  ownership already existed, so a new capability would have added a name without adding an observation.

  A settled decision was reversed deliberately and the reversal recorded: the register originally declared a
  shared bound once per capability, because declaring it once would leave the other specs silent about a
  bound they have. The reference form, which did not exist when that was settled, keeps the bound visible
  everywhere it applies while leaving one declaration to maintain, so the property the old rule protected is
  no longer bought at the price of restatement.


### WATCH / ACCEPTED / DECLINED / BUILT

- **WATCH: `PROJECT.md` restates facts a generated projection already holds, and states others nothing
  holds.** *Observed pressure:* three consecutive attempts at one paragraph were withdrawn, all failing the
  same way — asserting a location or an absence without sweeping for it. Classifying that file's claims
  against the generated projections splits them in two. The **architectural** ones are already carried:
  which crates exist, what each may depend on, that no dimension names a sibling — all projected from
  `tianheng_constitution()` into `AGENTS.self-law.md` and staleness-checked, and since
  `mutual-independence-reacts-to-membership` the last of those is asserted rather than merely projected.
  Two classes are carried by nothing: a **location** claim, the class that falsified the second attempt, and
  a **count**. *Observation source:* that classification, run against the projections and the file on
  2026-08-08. Not a census — the named instances are examples found by sampling claim-shaped lines, not an
  enumeration. *Current reaction or bound:* half of one. `crates/kanhe/tests/reference_integrity.rs` holds that a cited
  path **exists and is tracked**; nothing holds that the thing described lives there, which is the half the
  withdrawn attempt got wrong. *Risk:* the class that produced three withdrawn attempts, in the document
  `AGENTS.md` names as the contract — and it **grew** when the paragraph finally landed:
  `three-offices-are-vocabulary-not-crates` added five location claims to that file
  (`crates/guibiao/src/projection.rs`, `crates/tianheng/src/runner/render.rs`,
  `crates/xuanji/src/baseline.rs`, `.github/CODEOWNERS`, `crates/tianheng/src/constitution.rs`), each
  path-checked and none content-checked. Filing this entry as a smaller problem than before would have been
  the comfortable reading and the false one. *Promotion trigger:* a claim in `PROJECT.md` about the tree
  found false **after** this entry — the three found before it are the control and cannot stand as evidence
  for themselves. **Not fired.** *Version class:* not release-affecting. *Authority:* `projection-register`,
  which already enumerates the documents a claim could cite, and `self-law-projection`, which owns the one
  carrying the architecture. *Shape, if it fires:* not a detector over prose — that instrument was measured
  three times and rejected. The reachable direction is to make more of what the file asserts **citable**, so
  that restating is the strictly worse-looking option at the moment of writing.

- **WATCH: four limits of the mutual-independence reaction, each measured and each declared.** *Observed
  pressure:* closing the membership half of `三儀 ⊥ 三儀` exposed four more, all reproduced by writing them into
  the tree rather than argued about. **Wording, over-reacting:** paraphrasing `guibiao`'s clause makes the
  reaction fire — it refuses a reason that genuinely states the law. **Wording, under-reacting:** a `because`
  carrying the literal clause while *negating* it passes, and `AGENTS.self-law.md` then teaches the negation to
  every agent that loads it. **Enumeration, the dimension list:** `DIMENSIONS` is a hand-kept literal beside an
  enumerable set, and the set-coverage assertion cannot notice an omission because `found` is produced by
  filtering on `expected` — removing `guibiao` from the literal leaves a `guibiao` allowlist naming `hunyi`
  green. **Enumeration, the rule variant:** the filter admits only `RestrictDependenciesTo`, so a second
  boundary using `restrict_workspace_dependencies_to` — the more natural rule for this law — is never examined.
  *Observation source:* those four perturbations, run during review of
  `change/mutual-independence-reacts-to-membership`. *Current reaction or bound:* none of the four; the
  reaction's doc comment and `self-law-projection` state them where a reader meets them. *Risk:* the second is
  the serious one — the agent-facing projection can teach the negation of the law it quotes. *Promotion
  trigger:* fired for the second; the others are recorded with it because they are one reaction's limits and
  closing them separately would re-open the same file four times. *Version class:* patch; a `tests/` reaction of
  this repository. *Authority:* `self-law-projection`. *Shape:* **pinning** any of them needs the reaction run over a
  supplied declaration rather than its predicate over a string, which means factoring the assertion loop to take
  a `Constitution` — that is what this entry owns. **Declaring** them needed none of that, and an earlier draft
  of this entry said it did: it read the pin requirement as a declaration requirement and withheld all four,
  which kept a measured false negative out of the register a reader is told to consult before calling a
  behaviour a defect. All four are declared unpinned against this entry now.

- **ACCEPTED: capabilities whose subject is this repository are indistinguishable from those describing what
  adopters get, and nothing can tell them apart.** *Observed pressure:* the census that motivated
  `### Self-governance` found the same mispricing at a second surface. `gate-shape-contract`,
  `observation-bound-register`, `projection-register`, `self-law-projection` — and `release-coherence`, which
  review found to be a fifth — sit in the same directory, under the same lifecycle and the same review bar as
  the capabilities describing the product. Measured once, at `change/adopter-narrative-names-no-self-machinery`
  and over the **four originally identified**: 1,291 of the 8,027 non-blank lines under `openspec/specs`,
  about a sixth. Recorded as an observation of that moment rather than a live figure, since every one of those
  files changes — and since the fifth arrived while the entry was being written. *Observation source:* a keyword heuristic — per
  capability, mentions of this repository's own artifacts against mentions of adopters — and **the heuristic
  is part of the finding**. Run at `release/0.5.0` it named exactly those four. Run at this change it names
  six: `observer-protocol` sat one mention below the line either way, and `release-coherence` crossed it
  because this change added twenty-nine self-governance mentions to that spec. `release-coherence` was
  always a fifth by a plain reading of its Purpose — "the read-only repository reaction that keeps
  Tianheng's release commit spine coherent" — and the keyword count simply missed it. A set that a prose
  heuristic cannot hold steady is the instrument this repository measured three times and rejected, which is
  precisely why the set needs a marker a reaction can read. *Current reaction or bound:* none. Nothing
  distinguishes a capability whose subject is this repository from one whose subject is what ships, so
  nothing can. *Risk:* low and slow — it costs review attention and makes `openspec/specs` read as a larger
  product surface than it is; it cannot mislead an adopter, who never reads it. *Promotion trigger:* a fifth
  capability of this kind. **FIRED at filing**, by this entry's own measurement: `release-coherence` is the
  fifth, and `observer-protocol` is a sixth under one reading and not under another. Recording it as unfired
  would have been the more comfortable sentence and the false one. *Version class:* patch; specification layout of this repository. *Authority:* undecided — marking
  them is itself a governance decision about `openspec/specs`, and the change that found this deliberately
  left it alone so that change could close. *Shape:* the cheap form is a marker the register can read, so the
  distinction is enumerable rather than a convention; the expensive form is a second directory, which would
  move files every gate resolves by path.

- ~~**WATCH: a refusal site is defended only if some direction dies when its kind is swapped or its message
  sentinelled — and twenty-four are not.**~~ **Closed at the time** by the now-retired refusal mutation sweep,
  which turned the review technique into a repository check and, in doing so, corrected the entry: the
  twenty-four merged two populations a hand-run sweep cannot separate, because a perturbation kills nothing
  both when no direction distinguishes a site and when no direction reaches it. At closure it measured:
  `58 enumerated, 52 defended, 6 declared out of reach, 0 undistinguished, 0 unreached and unclaimed, 0 stale`.
  The residual was closed in order — fourteen sites constructed, two deleted as second reads of something the
  judgement already held, six declared with a slug joined in both directions to a bound. The mechanism was
  later retired when constructor locations were reclassified as implementation coverage rather than
  repository governance identities; focused behavior matrices retain the operator-facing evidence.

- **WATCH: the self-governance residual is a judgement over an entry's subject.** *Observed pressure:*
  `CHANGELOG.md` is the adopter's document and offered no heading that was not an adopter's vocabulary, so
  twenty entries named that machinery, before the section was collapsed — eleven in `[Unreleased]`
  and nine in the released `[0.4.0]` — spread
  across `### Added`, `### Changed`, `### Fixed` and `### Documentation`. The rule that now refuses them reads an entry's
  **references** — a word equal to a path under `scripts/`, or to a basename `git ls-files scripts/` resolves
  — and an entry describing this repository's own governance while naming no such word stays invisible to it. *Observation source:* two live instances, not a hypothetical: after this
  window's move, `CHANGELOG.md:173` and `:337` both sit under adopter headings, both describe the bound
  register's own behaviour, and both name nothing the enumerator resolves. *Current reaction or bound:*
  declared unpinned as
  `release-coherence/an-entry-about-self-governance-that-names-no-machinery-a-stated-bound`; the five limits
  that *do* have a mechanical WHEN are pinned in `tests/release_coherence.rs`. *Risk:* low and
  one-directional — the adopter reads a paragraph about housekeeping, never a wrong claim about what they get.
  *Promotion trigger:* an entry of this shape carrying a claim an adopter could **act on** — a version, a
  migration step, a behaviour change — rather than a description of internals. That is a property of one
  entry and decidable by reading it, unlike a threshold on a population nothing counts. **Not fired**: every
  instance found so far describes this repository's own machinery and asks nothing of a reader. *Version class:* patch; a
  document and a `scripts/` reaction of this repository. *Authority:* `release-coherence`. *Shape:* closing it
  needs a judgement over the entry's **subject** rather than its references, which is the prose detector
  `AGENTS.md` records as designed, measured three times and rejected — so the honest shape of this entry is a
  standing decision not to build it, revisited only if the trigger fires. *Risk of the alternative:* widening
  the matcher toward the subject — heading keywords, phrase lists — trades a declared, bounded blindness for an
  undeclared false-positive surface, which is the wrong direction under the Core Contract.

- **WATCH: a rejected observation point is recorded only in prose, and prose reaches no agent at the moment of
  temptation.** *Observed pressure:* rejections of whole detectors and approaches are scattered across doc
  comments, `AGENTS.md`, `BACKLOG.md` and spec prose in several non-overlapping phrasings, with nothing
  enumerating them. **No count is given, and one should not be.** Three were attempted and each was wrong within
  hours: the first stated a figure from a wider pattern set than the method it published, the second was
  falsified by this entry's own correction commit — which added two `was rejected` quotes to `BACKLOG.md`, one of
  the files any such scan reads — and every version miscounted in both directions, matching defects that merely
  describe something being dropped while missing rejections phrased another way. A grep over prose cannot census
  rejections, which is the entry's own point turned on itself. To look at the population rather than count it:
  `git grep -niE 'measured and rejected|was rejected|was dropped|would false-positive'` — one line, because a
  wrapped command's first half ran on its own and returned a silent zero. It is a starting point, not a census:
  its output includes this entry's own text and misses whatever is phrased differently. *Observation source:* reading that output, 2026-08-07, not a figure from it.
  *Current reaction or bound:* none enumerates them, though
  `docs/observation-bounds.md` shows the practice already reaching a generated projection by hand — it carries
  one refused approach with its measurement ("scanning paragraphs instead of lines … would not have caught it").
  One instance in a projection is the pattern existing without being enumerated, not the gap being closed.
  *Risk:* a rejection is re-proposed, re-measured and re-rejected; the reverse risk is equally real and is why
  this is WATCH rather than READY — see the trigger. *Promotion trigger:* **not fired.** The candidate evidence
  points both ways. `crates/kanhe/tests/bound_register.rs:399` records that the harness enumeration "was rejected
  TWICE in this file's own comments, on an unmeasured premise", then measured at 107ms cold and adopted — a
  rejection that a later reader *did* consult and correctly overturn, which argues that a durable, projected
  register would have entrenched a wrong answer twice. The trigger is a rejection demonstrably **re-proposed and
  re-measured** by someone who could not find the record, evidenced by a tree artefact rather than by a report
  from inside the work. *Version class:* patch; repository-internal, shipping in no crate. *Authority:*
  `observation-bound-register`, whose spec records two rejections in this shape — "declaring once was rejected"
  and "Keying on statement similarity was rejected rather than overlooked" — so the practice is that
  capability's, while nothing carries it into an agent's context. The phrasing that names the purpose
  ("recorded as rejected rather than left to be re-proposed") is in `CHANGELOG.md`, not in a spec, which is part
  of the observation.
  *Shape, if it fires:* another instance of the enumerate → react → audit cycle beside those
  `projection-register` enumerates, not a new crate and not the separate ADR file class `AGENTS.md` forbids.
  Two constraints the survey already forces on the record type: separate the **load-bearing** reason from the
  **incidental** evidence — `AGENTS.md`'s prose-detector rejection leads with four closable false positives and
  buries the one unclosable false negative — and make strength **derived** from structural facts rather than
  authored, the way `Extent::demonstrates` is, so it cannot be self-assessed.

- **WATCH: the rule shape the self-law relies on most is absent from `examples/`.**
  *Observed pressure:* `restrict_dependencies_to` carries six of the eleven boundaries in `AGENTS.self-law.md`
  and appears **zero** times anywhere under `examples/` (`grep -rc restrict_dependencies_to examples/`). Examples are the 潛移 imitation surface, so the allowlist shape the project
  governs itself with is one an adopter running the dogfood never meets. Narrow deliberately: `COOKBOOK.md` is
  also an imitation surface and *does* carry copyable recipes using it, so the pressure is the executable
  examples specifically, not the teaching surface as a whole — an earlier draft claimed the wider thing and was
  wrong. *Observation source:* `grep -rc restrict_dependencies_to examples/` against
  `grep -c 'restrict dependencies to' AGENTS.self-law.md` — note the two spellings, the DSL method and the
  projection's rendering; a grep for the method name against the projection returns zero and means nothing.
  Read 2026-08-07.
  *Current reaction or bound:* none. *Risk:* an adopter imitates what the examples show. *Promotion trigger:* an
  example is added or revised for another reason, at which point the shape is chosen deliberately rather than by
  omission. *Version class:* patch; `examples/` ships in no crate. *Authority:* `governance-dogfood`, which owns
  the examples' reaction. *Explicitly not claimed:* that enumerated denylists are wrong, or that the self-law
  avoids them. It does not — its inline-symbol-path confinements each carry an enumerated verb list, and
  `inline-symbol-path-confinement` already declares the unlisted remainder as a bound the adopter owns. An
  earlier draft of this entry claimed the opposite on four counts and was corrected by review; what survives is
  the absence above and nothing more.


- **WATCH:**
- **WATCH: a merge or publish made outside the wrapper is not observed.** *Observed pressure:* both
  assertions guard the sanctioned path — the wrapper's `1 passed` and the reaction pinning the identifier it
  cites. A `cargo publish` run directly, or a merge made in the browser, reaches neither. *Risk:* the record
  or the published source escapes the gate that stands in front of it. *Next trigger:* an act reaching either
  without the wrapper. *Authority:* engine. *Compatibility:* none — reaching further means observing the
  operator's shell or GitHub's servers rather than this repository.
- **WATCH: which governance member a reaction belongs to is unobserved.** *Observed pressure:* the split
  between 繩墨 (the law and the delivered product) and 勘合 (this repository's record) is a judgement about
  what a reaction judges, and two mechanical rules were each measured unreliable — a text scan reads a comment
  naming `AGENTS.md` as governance while a reaction scanning every tracked file names nothing, and
  `TIANHENG_WORKSPACE_TESTS` means both "this needs the repository as its subject" and "this needs a fixture".
  *Risk:* a reaction lands in the wrong member and the two identities blur again, which is the failure the
  split was built to end. *Next trigger:* a third member, or a reaction whose placement two readers disagree
  about. *Authority:* engine. *Compatibility:* none — neither member ships.
  - **A `#[path]`-shared test module's `allow(dead_code)` cannot distinguish "used by no binary" from "used by
    some".** *Observed pressure:* `crates/tianheng/tests/support/` is compiled fresh into each `*_conformance.rs`
    binary, so an item only some callers use is genuinely dead in the others — which is why the blanket
    `#![allow(dead_code)]` there is correct and documented. It also silently covers an item used by **zero**
    binaries: `Header::comments` was added with the region newtypes, never called anywhere, and passed every
    clippy pass in the Definition of Done including the two that exist to catch dead code. *Observation source:*
    the 0.5.0 closing review's expanded dead-code sweep, after an external review found the sibling instance in
    the shell era's shared capture library; the method is deleted, the class is not. *Current reaction or bound:* none. Three
    clippy passes cover non-`pub` Rust and a `--workspace` pass exists precisely to catch dead code that ships,
    and none of them can see past a justified allow. *Risk:* a shared test helper accumulates an API nobody calls,
    which is worse in a support module than elsewhere — it reads as the blessed way to do something, so the next
    author adopts an untested path. *Promotion trigger:* a second item in `tests/support/` found dead by hand.
    One instance does not justify a cross-binary usage reaction, and building one is a design decision: the
    property is "referenced by at least one test binary", which needs an enumeration of binaries and their
    references rather than a compiler lint. *Version class:* tests only; no published surface. *Authority:* that
    module's own header, which states the allow and its reason, and `gate-shape-contract`, whose two-way
    correspondence between a gate and its twin is the shape any answer here would generalize.
  - **A changelog entry that refers to another by position breaks when the entries are regrouped.**
    *Observed pressure:* six positional cross-references in one `[Unreleased]` section; **three were broken** when
    found. Two had been wrong before anything moved — "the previous entry's own repair" pointed at
    `assert_projection_matches`, which has nothing to do with the repair it describes, and "a regression the
    previous entry introduced" attributed the shared exit-contract backstop to the entry beside it rather than to
    the backstop. The third was broken by merging the section's duplicate group headings: an entry saying "the
    entry below" pointed into `Documentation`, which the merge moved from last to first. *Observation source:* the
    closing review of the 0.5.0 window, sweeping `[Unreleased]` for positional references after the group merge;
    each antecedent was resolved by reading it rather than assumed. *Current reaction or bound:* none. The group
    merge's own verification compared the **multiset of lines** before and after, which is correct for "no entry
    text was lost" and structurally blind to "an entry still points at what it meant". *Risk:* an adopter follows
    a reference to the wrong entry, or to none — and a changelog is the one document written for people outside
    this repository. *Promotion trigger:* a positional reference appearing again after this sweep. Not a count:
    the sweep is the control, exactly as the un-reacted-SHALL entry above sets it up. *Why not simply forbidden:*
    three of the six resolve soundly and one of them is load-bearing — an entry citing the bullet immediately
    after it, within one group, which any regroup preserves. And references *into* `Documentation` are now
    structurally safe, since it is the first group and everything else is below it. A rule refusing "above" and
    "below" outright would refuse those three, so the rule has to distinguish a reference within a group from one
    across groups, which is a design decision rather than a grep. *Version class:* documentation only; no
    published surface. *Authority:* `release-coherence`, which owns what `CHANGELOG.md` must be true of, and this
    window's group-merge change, whose verification is the measured gap.
  - **Whether a gate's chosen exit code is the semantically right one.** *Observed pressure:* the class occurred
    once, in `86e8592`, and produced **both** directions of `gate-shape-contract`'s `1-versus-2` bound in one
    gate — every refusal was `1`, so a shallow clone reported *"the release surfaces disagree"* (a
    misconfiguration as a violation), while the exit-contract backstop converted every genuine incoherence into
    `2` (a violation as cannot-judge). *Observation source:* that commit's own reproduction, and the re-reading
    that corrected the bound's stated cause. *Current reaction or bound:* the enabling mechanism is **closed** —
    what let the inversion pass CI was the matrix asserting a non-zero status rather than a code, and
    the retired gate-shape capability's `exit codes` property required the exact code from every twin, citing this
    instance in its remedy. What is left unobserved is the semantic judgment alone, and the bound now says so.
    *Risk:* a gate whose twin asserts an exact code that is the wrong code reads as fully conformant, and the
    consumer of a wrong code is sent looking for the wrong kind of problem. *Promotion trigger:* an instance where
    a twin asserts an **exact** code and that code is semantically wrong — a distinction the bound's earlier text
    could not have drawn, which is why this entry exists rather than the old wording standing as evidence for
    itself. *Version class:* patch. *Authority:* `gate-shape-contract`'s *Observation bounds* requirement, whose
    own rule that a bound is narrowed rather than restated is what this correction followed.
  - **A capability whose reactions are shell gates cannot pin a bound of its own.** `PINNED-BY` resolves a
    Rust test under `crates/`, while every defence of `observation-bound-register`,
    `self-law-projection`, and the gate surface of `violation-baseline` is a shell fixture — so such a
    capability can *state* a residual but not *declare* it, and the projection under-counts exactly where
    the register describes itself. Filed as READY-PATCH one change earlier, on a live instance that has
    since **dissolved**: the instance was "a definition inside a block comment satisfies a citation", and
    it was declared a residual only because the harness enumeration had been rejected twice on an
    unmeasured premise. Measured, a throwaway fixture crate enumerates cold in 107ms, so the residual was
    closed rather than declared and this entry lost the pressure that justified its class. Demoted to
    WATCH rather than closed, because the observation itself stands and is independent of that instance.
    Promotion trigger: a second such residual that is genuinely out of reach — one where the exact
    observation source has been *measured* and found unaffordable, not estimated from inside the code.
    That qualifier is the entry's real lesson. `gate-shape-contract` did **not** fire it, and the way it
    avoided doing so is worth stating: the same limitation decided its shape — a shell-defended capability
    could not have pinned its own declared bounds — and the answer was to write the reaction in Rust, not to declare
    a residual. An entry recording that shell gates cannot pin bounds is not evidence for itself every time
    a capability chooses Rust because of it.
  - **`BoundaryKind` has no value a third-party participant owns.** *Observed pressure:* an outside
    `Observer` must label every violation it emits with one of 三儀's four kinds — `Crate`, `Module`,
    `Semantic`, `Runtime` — even when it governs nothing any dimension would call by those names.
    *Observation source:* `examples/observer-participant`, written for the change that gave
    `Observer::bounds` a consumer; its house rule is a file-header convention, and it reports `Module`
    as the nearest honest fit with a comment saying so. *Current reaction or bound:* none — the kind is
    accepted as written, and nothing checks that a participant's kind matches what it governs. *Risk:* the
    kind is the projection label a report, a SARIF render and a **baseline** all carry, so borrowed kinds
    make an adopter's recorded entries group under a dimension that did not produce them; a consumer
    filtering by kind sees an outsider's findings as 圭表's. *Promotion trigger:* a participant that is not
    this repository's own example — an adopter's, or a second one here — where the borrowed kind is shown to
    mis-group a real baseline or filter, rather than merely reading wrong. One example that chose the label
    itself is not evidence about adopters, which is the trap the shell-gate entry above records.
    *Version class:* minor at most. Adding a variant to a `#[non_exhaustive]` enum breaks no downstream
    match, and it changes no verdict; what it changes is the projection vocabulary, which is why it is a
    decision rather than an addition. *Authority:* `observer-protocol`'s requirement that a participant
    outside the family be demonstrated joining a run, and that example's README, which records the finding
    where someone writing their own participant will meet it.
  - Token/Lexer extraction (requires cross-scanner false negative or 3rd scanner).
  - `qianyi` generator & LSP/editor integration.
  - ~~A `#[cfg_attr(pred, path=…)]` remap on an **inline** `mod name { … }`~~ **CLOSED** in the 0.4.0
    window. Reproduced against the real entry point, as this entry's own trigger required, with the
    unconditional `#[path]` form as the control — and the reproduction **refuted the recorded risk
    class**. The entry predicted a false negative ("the arm's real children may never be scanned");
    the actual behaviour was a *constitution error* (exit 2, "source file could not be located"),
    so it was fail-loud, never a silent pass. What it really was is narrower and worse for adoption
    than for correctness: 圭表 refused to judge a crate that compiles cleanly, so an adopter using the
    idiom could not run `check` at all. The root cause was one line — `walk.rs` read only
    `direct_path_eq` and never `conditional_path_eqs` — and the fix follows every present candidate
    base, cfg-blind, exactly as 漏刻's spec already required for the identical shape. The lesson kept:
    a single-lens hypothesis can be right that something is broken and wrong about **how**, and the
    risk class is what decides urgency — so reproduce before promoting, which is what this entry's
    trigger said and why it was worth keeping.
  - ~~`xingbiao::crate_root_file` may collapse a multi-root package~~ **RETIRED — superseded by its
    own promotion.** Its trigger was "confirm `cargo metadata` actually emits multiple root files for
    one package before treating this as more than speculative". That was confirmed in the 0.4.0
    window (`[[bin]]` targets are reported alongside the `lib`, each with its own `src_path`), which
    promoted the hypothesis into the DESIGN-BREAKING entry above — now itself CLOSED by the
    per-target corpus. The WATCH line survived its own promotion and is struck here rather than
    deleted, since a reader of an older release may still be looking for it. Lesson kept: when a
    WATCH item is promoted, retire the WATCH line in the same change, or the index carries the
    question and its answer at once.
  - Baseline debt ratchet (`--require-baseline-reduction`, only-fix-never-add). This remains in
    tension with “baseline is a generated snapshot, not policy” and “not a governance platform”:
    a bounded opt-in gate may fit, while debt scheduling does not. Promote only after that tension
    is resolved with adopter pressure and an explicit observation/reaction design.
  - **`xuanji` sink for shared run/projection vocabulary.** Class: WATCH. Observed pressure:
    `pacta` consumes the composed facade while `modou` consumes and re-exports Guibiao's widened
    standalone check/baseline/projection surface. Observation source: those reference-consumer
    builds and the current crate dependency graph. Current bound: keep dimension-owned
    projection/check surfaces above vocabulary-neutral `xuanji`; sinking them now would break the
    proven standalone Guibiao consumer without demonstrated cross-dimension deduplication. Risk:
    duplicated shell vocabulary may grow if multiple instruments acquire standalone products.
    Promotion trigger: a second standalone instrument consumer (Hunyi or Louke) demonstrates the
    same projection/run need and makes a shared sink earn its migration cost. Version class:
    DESIGN-BREAKING. Authority: `PROJECT.md`'s crate-family and dimension-independence decisions,
    `openspec/specs/rule-model-surface/spec.md` reference-consumer contract, and the Pacta/Modou
    compatibility evidence recorded in `CHANGELOG.md`.
- **ACCEPTED DEBT:**
  - Macro/configuration coverage bounds. **One** named residual now that all three dimensions read
    `cfg_if!` arms: transparency covers **item position** only — an invocation inside an
    `impl`/`trait` body holds impl items, needing a parallel flattening across ~10 body walkers
    (measured, pinned by `a_cfg_if_inside_an_impl_body_is_a_stated_bound`). 漏刻's own lack of
    transparency, listed here as the second residual, is CLOSED: its byte scanner reads arm contents
    as real code in both passes, and `cfg_if_transparency_conformance.rs` now pins all three
    dimensions on the one fixture rather than two of three — its module doc already says so, and this
    entry was the site left behind.
  - ~~File-granular un-auditable-probe identity.~~ **RETIRED** — superseded, not accepted: the 0.4.0
    line qualifies an un-auditable-probe fact by its owner-qualified enclosing item and the offending
    expression's own text alongside its file, so the identity is no longer file-granular and this bound
    no longer exists. Kept struck rather than deleted because a reader of an older release may still be
    looking for it.
  - **漏刻's legacy directory corpus does not descend a symlinked subdirectory.** Measured on one
    fixture rather than hypothesised: a module reached through `#[path]` into a symlinked directory,
    holding a declared seam's only probe, is seen when the audit is given the **target root file**
    (clean — the module graph reaches it and reading a file follows symlinks) and unseen when it is
    given the **directory** (the seam reads as unprobed). Deliberate: the directory walk classifies
    entries with `file_type()`, which does not follow symlinks, so a symlinked directory is not
    recognized as one — which is what keeps a cyclic symlink from becoming an unbounded walk. Accepted
    because the input that matters has no gap: the 天衡 shell passes root files, and the directory input
    exists for source compatibility. Stated in `runtime-origin-assertion` and pinned in both directions
    by `a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory`. This replaces
    a WATCH entry that guessed a different mechanism — "`try_visit`'s cycle guard possibly bypassed by a
    second, weaker guard" — which is refuted: no guard is bypassed, and the entry did not distinguish
    the two corpora, which is where the whole answer lives.
  - **The baseline directory flush has no reacting test, by construction.** `sync_parent_dir` is
    infallible and best-effort on purpose — a platform or filesystem that cannot flush a directory
    must not turn an already-landed write into a reported failure — which leaves it with no
    externally observable behavior for a test to bind. Measured rather than assumed: `cargo mutants`
    over `runner.rs` reports both of its mutants (`replace sync_parent_dir with ()`, `delete !`) as
    MISSED, while the mutants for the flag and zero-length rules beside it are caught. Its evidence
    is the syscall sequence recorded in the PR that added it (`fsync(temp)` → `rename` →
    `fsync(dir)`), not a gate. Accepted because the alternative — making it fallible, or counting
    failures the way 漏刻's `dropped_sink_events` does — either reintroduces the regression it exists
    to avoid or adds an adopter-visible counter for a CLI-side hygiene step nobody can read. Revisit
    only if a directory-flush regression is ever actually observed. The *strict* half of the
    guarantee (the file flush) is not in this bound: it is covered by the `baseline_cli`
    suite.
- **DECLINED:**
  - Wall-clock auto-decay / auto-expiration (breaks determinism).
  - Trait method set freezing (API contract, not architectural shape).
  - Pre-creating empty crates/modules.
  - `GovernanceTest` standard-preamble generator API (`standard_preamble(test_name: &str)`
    or equivalent). Drift law: no API surface without adoption pressure. The preamble discipline
    (universals only, no architectural claims) is a documentation concern, not an API contract;
    `assert_projection_fresh_with_preamble` already accepts a caller-supplied `&str`. If three
    adopters independently write incorrect preambles containing architectural claims that then
    rot, revisit.
  - "`.github/CODEOWNERS`'s amendment reaction is unenforced" (0.3.1 sweep). Self-refuted by its
    own citation: the file's own disclaimer at the cited line already states plainly that
    designation alone only auto-requests review, and that branch protection must be separately
    enabled to make it binding — not a hidden gap.
  - "`ci.yml`'s Release-coherence job is the one Definition-of-Done gate CI runs but branch
    protection doesn't enforce" (0.3.1 sweep). Observations reproduced accurately, but none
    constitutes a defect under this project's own contract taxonomy — CI running a check and
    branch protection gating a merge are different, both-documented mechanisms, not a claimed
    single guarantee.
  - "漏刻 never diagnoses a module cycle (a circular `#[path]`/symlinked module directory
    silently collapses instead of exit 2)" (0.3.1 sweep). Refuted: a test already pins the exact
    shape, asserting the opposite of the claim — a documented, deliberately-pinned bound, not an
    undetected divergence.
  - "The self-law giving `xingbiao` its canonicalization monopoly only reacts to the
    free-function form; `path.canonicalize()` (the method call) escapes" (0.3.1 sweep). The
    mechanism reproduces, but it is an explicitly declared, spec'd, and test-pinned observation
    bound of the observation source itself — not a silent false negative, and not a reason
    outside the projected perimeter.
  - "A duplicated field name in a baseline entry's fact is silently last-wins, so the entry
    suppresses a different violation than the one it records" (0.3.1 sweep,
    `crates/xuanji/src/identity.rs`). Mechanics reproduced (`Baseline::from_json` on a
    duplicate-keyed `fact.fields` object does resolve last-wins), but the claim does not survive
    the contract lens — the shape cannot arise from any real fact construction path in this
    codebase, only from hand-authored malformed JSON.
  - "The composed baseline dogfood (`cargo test -p shengmo --test examples_suite`) exercises only the suppression
    direction, and its in-script justification misstates what the standalone test proves"
    (0.3.1 sweep). Refuted: a test-coverage complaint dressed as an unhonored claim — every
    documented claim it cites is in fact honored by the referenced test and README.
  - "Feature rules never read the target's own `[features]` table, so `serde/derive` declared
    there passes a rule that forbids every feature of `serde`" (0.3.1 sweep,
    `crates/guibiao/src/cargo_metadata.rs`). Mechanics accurate, but doubly refuted: the behavior
    is a stated bound (圭表 governs the *declared* per-target layer; resolved whole-graph
    feature unification is `cargo-deny`'s lane, per this file's own architecture decision), and
    separately the specific reproduction's classification didn't hold either.
  - "A `cfg_attr`-wrapped `#[path]` target is never enqueued, so the relocated file's typo'd /
    un-auditable probes are silently skipped" (0.3.1 sweep, `crates/louke/src/audit/scan/probes.rs`).
    Refuted: a stated bound spelled out in three places (spec, public API doc, `CHANGELOG.md`)
    and pinned by a deliberate test, not introduced by the diff under audit.
  - "`first_macro_arg_end` truncates an `as`-cast generic, merging two textually distinct
    un-auditable probes into one finding" (0.3.1 sweep, `crates/louke/src/audit/scan/lexer.rs`).
    Mechanics reproduce at the byte-scanner level, but the trigger is not reachable from
    compilable adopter input — refuted on the reproduction lens.
- **BUILT / HISTORY:**
  - Opt-in gate flag `--disallow-stale` enforcing zero stale baseline entries in CI gate mode.
  - Non-generic compound type alias target traversal in `hunyi` (tuples, arrays, slices, references, raw pointers).
  - Mid-path `super` and `self` module path normalization in `guibiao`.
  - `cfg_if!` macro expansion unstripping and ancestor-glob fail-closed observation in `guibiao`.
  - `cfg_if!` arm transparency in `hunyi` (item position: arm items collected, arm-declared modules
    walked, arm membership treated as cfg-conditional), pinned against `guibiao` on one shared
    fixture.
  - Configurable custom probe macro marker attributes in `louke`.
  - Three-layer agent-law artifact naming (Preamble + Projection + Law Source) and `COOKBOOK.md` recipe with `GovernanceTest` preamble discipline.
  - `cfg_attr(path)` observe-both semantics (union-scan over default and physically existing `cfg_attr(path)` target paths).
  - `guibiao`'s own module-boundary walk now tolerates a module backed only by one or more resolved `cfg_attr(path)` remaps (no plain conventional file, no direct `#[path]`), matching `hunyi`/`louke`'s identical rule for the same shape.
  - Reusable testing harness (`tianheng::testing::GovernanceTest` fluent builder in facade for reaction, coverage, projection freshness with `BLESS=1`, and fixture testing).
  - Self-governance observation depth upgrade (explicit ScanDepth declarations across crates/shengmo/src/law.rs boundaries).
  - `PublicSeam::InherentMethod`/`InherentAssoc` now carry the impl block's own declaring module,
    closing the two-different-modules false negative verified real during the 0.3.1 sweep
    (`hunyi-public-seam-module-injection`).
  - `unsafe_confinement`'s and `trait_impl`'s `allowed_locations` (`only_implemented_in`/`and_in`,
    `only_under`) now reject a malformed `::`-path entry (an empty segment) as a constitution error,
    sharing the `must_not_expose`/`must_not_acquire` forbidden-operand family's guard
    (`resolve::validate_path_operands`, extracted from four verbatim call sites at the same time).
    Closes the entry previously recorded here as ACCEPTED DEBT: reproduced directly against
    `trait_impl_findings`/`unsafe_findings` before fixing, confirming the failure direction the
    debt entry named (a malformed allowed entry made every real site look disallowed, a spurious
    violation rather than a silent pass) — both named call sites are now fixed, not merely one
    (`hunyi-shared-path-operand-validation`).
  - Detailed shipped capability ledgers for 0.1.x through 0.3.0 are archived in [`docs/history/0.1.0-0.3.0-built-ledger.md`](docs/history/0.1.0-0.3.0-built-ledger.md).

### Closed — reproduction records (0.4.0 onward)

These are **not** open work. Each was a live item in this index when the window opened and is now closed;
the original entry is kept verbatim beneath its closing note because the reproduction record —
what was observed, by which lens, and why the trigger was believed narrower than it was — is the part
that stops the same defect being re-found from scratch. The present-tense `Class:` / `Risk:` /
`Promotion trigger:` lines inside each retained entry describe the state **at the time it was written**.

They live here rather than under their own class heading because an index that carries a question and its
answer at once is a reader trap — the same reason a stale WATCH line was retired in `68e183b`, applied to
the larger entries it left in place. Neither the class nor the number is restated collectively: each
retained entry carries its own `Class:` line, and a count here would go stale the next time an item
closes — which is how the previous two sentences came to say "DESIGN-BREAKING" and "six" about a section
that also holds a closed READY-PATCH record.


- ~~**A bare trait name may not resolve against a same-module trait, contrary to the bound's own wording.**~~ **CLOSED** in the
  open window, in two steps, and the second is the one the record has to carry. The probe confirmed the gap: a
  bare principal trait declared in the governed module resolved to nothing when no `use` named it, so a
  boundary forbidding it did not react. The first fix resolved **every** unresolved single-segment principal
  to `{module}::{name}`, which over-reached in one direction and under-reached in the other — a bare name the
  module does not declare (a prelude trait, a glob import, a name the file never mentions) was fabricated into
  the module and reacted against a path that module never had, while a raw identifier was left as
  `crate::m::r#type` and never matched the canonical `crate::m::type` every other resolution site produces.
  What stands now: the fallback fires only for a name present in the **branch-local type namespace**, carried
  on `FileExternScope` (which already computed it for `externs_type`), and the segment is canonicalized with
  `strip_raw` first. `BareFallback::CurrentModule` is deliberately not used — it resolves without proving the
  name exists, which is the over-reach that was removed. Defended by four guards, two per capability: the
  re-pointed `dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound` and
  `impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound` (now forbidding the module-qualified
  spelling, which is what makes them observe the drop rather than a spelling mismatch), and
  `dyn_operand_bare_raw_identifier_local_trait_resolves_canonically` with its impl-trait twin. Version class:
  **minor**, as this entry's own promotion line predicted for a false-negative closure — see *Version
  horizons*.

- ~~**Five declared bounds have no pinning test.**~~ **CLOSED** in the
  open window. Closed by writing unit tests for all 5 remaining UNPINNED scenarios across `external-crate-confinement`, `runtime-origin-assertion`, and `semantic-dyn-trait-boundary`, bringing `unpinned` count to 0.

- ~~**`crates/kanhe/tests/reference_integrity.rs` has no companion failure matrix.**~~ **CLOSED** in the
  open window. Closed by adding `crates/kanhe/tests/reference_integrity.rs`, a throwaway git repository fixture proving every refusal (exit 1 and exit 2) and pass direction of `crates/kanhe/tests/reference_integrity.rs`.

- ~~**Owner-label identity collapses across a cfg-collided self-type alias.**~~ **CLOSED** in the
  0.4.0 window, after an independent review re-derived it. Closed by refusing to name the ambiguity
  rather than by inventing an encoding for it: an owner role whose head admits more than one distinct
  candidate now reaches the shared fail-loud identity gate (a constitution error, exit 2, naming the
  `#[cfg]` collision), because neither candidate may be preferred and the candidate SET is identical
  for both colliding sites, so nothing available separates them. "Cannot judge" over a silent collapse
  is the Core Contract's own ordering. The single-candidate `resolve_path` that fed every such label
  was **deleted**, not merely bypassed — `resolve_path_all` is now the only resolver, so a caller
  needing one value must decide what to do about `len() > 1` instead of receiving an arbitrary pick,
  and the defect class is unrepresentable rather than fixed at three sites. `semantic-signature-coupling`
  gains the requirement and two scenarios. The original entry is kept below for its reproduction record.

- **Owner-label identity collapses across a cfg-collided self-type alias.** Class:
  DESIGN-BREAKING. Observed pressure: reproduced by the maintainer during round-3
  adversarial review of `change/hunyi-cfg-branch-use-reexport-merging` (PR #149) —
  two genuinely independent violations sharing one cfg-collided self-type alias
  (`#[cfg(unix)] use crate::a::Foo as X; #[cfg(not(unix))] use crate::b::Bar as X;`,
  each arm implementing the same governed trait) render the identical single-candidate
  owner label and collapse to one finding under exact-identity dedup, across
  trait-impl-locality, forbidden-marker, unsafe-confinement, and signature-coupling at
  once. Observation source: that review's reproduction, described above. Current bound:
  `canonical_self_owner`/`canonical_self_owner_without_fallback`
  (`crates/hunyi/src/resolve/shape.rs`) and `canonical_unsafe_owner`
  (`crates/hunyi/src/scan/unsafe_sites.rs`) each render a label from a single resolved candidate,
  cfg-blind, feeding straight into a dedup key. Risk: a real, enforce-severity
  violation is silently lost whenever this exact cfg-collision shape occurs — a false
  negative, the one bug class PROJECT.md's Core Contract forbids outright — but the
  trigger (a cfg-gated self-type alias reused across cfg-exclusive impls of the same
  governed trait/type) is narrow and has not yet been observed outside adversarial
  probing. Promotion trigger: either a redesigned owner identity that stays injective
  across cfg-ambiguous candidates, or a different anti-collapse key not fed by a
  single-candidate renderer — real design work, not a mechanical multi-candidate swap
  (explicitly scoped out of PR #149 as its own follow-up). Version class:
  DESIGN-BREAKING (an injective owner identity almost certainly changes baseline
  `finding_key` shape for every affected capability). Authority: `PROJECT.md`'s
  violation-identity decisions, PR #149's commit body (which names this exact gap and
  scopes it out as a follow-up).

- ~~**An absolute `#[path]` literal's identity still disagrees across checkouts when its target
  coincidentally lies under one checkout's own anchor.**~~ **CLOSED** in the 0.4.0 window — the last
  open identity gap. Closed by stopping the relativization rather than by threading provenance through
  a labeling decision: a file reached through an absolute literal keeps the path the literal wrote, in
  every checkout, and the flag is inherited by the files that target reaches in turn. The recorded
  promotion trigger described the fix as threading "was this file reached via an absolute `#[path]`
  literal" through four functions, which read as a broad refactor and is why it sat; it is not, because
  `Path::join` discards its receiver EXACTLY when the joinee is absolute, so the fact is knowable at the
  single line that resolves the literal and only has to ride alongside the `(file, base)` pair the walk
  already carries. The test that pinned the gap failed with EQUAL identities the moment the flag landed
  — its own doc had said that is what closure would look like — and is replaced by one asserting it;
  the test that pinned "relativized when inside the anchor" is inverted, that behaviour having been the
  gap's mechanism. `runtime-origin-assertion` gains the rule and drops the two scenarios describing the
  old behaviour. The original entry is kept below for its reproduction record.

- **An absolute `#[path]` literal's identity still disagrees across checkouts when its
  target coincidentally lies under one checkout's own anchor.** Class: DESIGN-BREAKING.
  Observed pressure: found during round-2 adversarial review of
  `change/louke-unauditable-probe-relative-identity` (PR #157), which closed the
  general relative-path case but left this one already-non-portable construct
  unresolved. Observation source: pinned regression test
  `a_nested_absolute_path_literal_still_disagrees_across_checkouts_a_known_residual_gap`, which failed
  with EQUAL identities the moment the fix landed and is replaced by
  `a_nested_absolute_path_literal_now_agrees_across_checkouts`
  (`crates/louke/src/audit/tests.rs`). Current bound: `strip_prefix` succeeds by pure
  textual match wherever an absolute `#[path = "/…"]` literal's target happens to share
  the anchor's prefix, producing a relative-looking label in that checkout and falling
  back to the raw absolute path in another — the two checkouts' `unauditable-probe`
  identities differ. Risk: narrow — an absolute `#[path]` literal is already
  non-portable and machine-specific by construction; this only affects that one already
  fragile idiom, not the realistic relative sibling-share case PR #157 fixed. Promotion
  trigger: threading "was this file reached via an absolute `#[path]` literal" through
  the whole `resolve_path_module`/`external_module_files`/`collect_scope_modules`/
  `collect_reachable_probes` pipeline so such a file's label is never relativized at
  all — a separate, scoped refactor. Version class: DESIGN-BREAKING (changes
  un-auditable-probe identity shape for this construct). Authority: PR #157's commit
  body. The bound was also stated in `crates/louke/src/finding.rs`/`audit.rs`'s own doc comments and in
  `openspec/specs/runtime-origin-assertion/spec.md`; all three now state the CLOSED rule instead (an
  absolute literal is never relativized), so this record's description of the old behaviour is history,
  not a description of any current surface.

- ~~**`InherentGenerics` seam identity has no per-block distinguisher WITHIN one module.**~~ **CLOSED**
  in the 0.4.0 window. The seam now carries the **bounded thing** each exposure sits on — a parameter's
  own name, or a where-predicate's rendered bounded type — so two blocks in one module bounding
  different parameters to the same forbidden type are two distinct facts. The distinguisher this entry
  called a design decision is resolved by taking the one the codebase already had: it is keyed exactly
  like a trait `impl`'s own `where` position, and both now come from one shared walk over an impl's
  generics positions, so the vocabularies cannot drift. An impl-block **ordinal** was ruled out rather
  than chosen against, since `semantic-signature-coupling` forbids identity resting on scan order or
  item ordinal.
  What remains is a limit, not a gap, and is stated in the seam's own doc: two blocks whose bounds are
  *textually identical* still resolve to one seam. Nothing structural distinguishes them — their
  contents carry their own seams and position is forbidden — so two blocks bounding the same parameter
  to the same forbidden type state one architectural fact twice, exactly as one import on two lines is
  one violation. Completeness of the position walk rests on a language rule verified against a real
  `rustc` rather than assumed: an `impl`'s generic parameters cannot carry defaults, so a parameter
  contributes only its bounds (or, for a const parameter, its type). The original entry is kept below
  for its reproduction record.

- **`InherentGenerics` seam identity has no per-block distinguisher WITHIN one module.**
  Class: DESIGN-BREAKING. Observed pressure: verified real during
  0.3.1 sweep cleanup (2026-08-02/03) — two separate inherent impl blocks on the same
  type, each exposing the same forbidden subject through a different where-clause
  bound, collapse to one violation. Observation source: direct reproduction against
  `hunyi::check` (`crates/hunyi/src/collect/exposure.rs`'s inherent-generics collector) —
  described above. Current bound: `PublicSeam::InherentGenerics` is keyed on
  `{module, owner}`. The **cross-module** half of this entry is CLOSED — the module role was
  added in the 0.4.0 window, after an independent review re-derived it, and the adjacent doc
  comment that wrongly claimed owner-qualification alone kept the seam "distinct... from
  another block's generics" was corrected in the same change. What remains is strictly
  narrower: two impl blocks for the same owner **in the same module** still resolve to one
  seam, since owner-plus-module distinguishes where an impl is written, not which block it is.
  Risk: a false negative on a narrow idiom (two inherent impl blocks on one type in one
  module, each with its own where-clause bound exposing a forbidden type) — not yet observed
  as adopter pressure, and one step narrower than when this entry was written. Promotion
  trigger: a real per-block distinguisher added to `PublicSeam::InherentGenerics` — real
  design work, and constrained rather than open: `semantic-signature-coupling` forbids
  identity from resting on "scan order, item ordinal, and renderer fallback position", so an
  impl-block ordinal is NOT available as the distinguisher and a stable structural key (a
  rendering of the block's own generics, say) has to be designed and its collision behavior
  argued. Version class: DESIGN-BREAKING. Authority: this entry's own reproduction record
  (above); `openspec/specs/semantic-signature-coupling/spec.md`'s ordinal prohibition for the
  constraint on the trigger.

- ~~**Trait-impl-locality's violation target/rule-key reads the constitution's declared trait
  spelling instead of the already-resolved canonical anchor.**~~ **CLOSED** in the 0.4.0 window, after
  an independent review re-derived it. Both roles are now keyed on the resolved anchor, threaded out of
  the function that already computed it rather than recomputed (a second resolution site could
  disagree with the one that decided the matches). The multi-candidate question this entry named as the
  reason it was design work is answered by refusing rather than picking: a declared anchor whose
  re-export closure reaches more than one distinct local trait DEFINITION is a constitution error
  naming the candidates, since the ambiguity is in the declaration and choosing would make the
  governed target arbitrary. A declaration that already names the defining path — the ordinary case —
  is unchanged, so the fix churns only the baselines it must. `allowed_locations` deliberately stays in
  the rule key (it keeps two boundaries on one trait distinct) and the in-code comment that wrongly
  claimed it was NOT identity-bearing is corrected: `ViolationId` compares `rule_key` in full.
  `semantic-trait-impl-locality` gains all three rules and two scenarios. The original entry is kept
  below for its reproduction record.

- **Trait-impl-locality's violation target/rule-key reads the constitution's declared
  trait spelling instead of the already-resolved canonical anchor.** Class:
  DESIGN-BREAKING. Observed pressure: verified real during 0.3.1 sweep cleanup
  (2026-08-02/03) — declaring the identical boundary via two different (but
  re-export-equivalent) spellings of the same trait produces two `ViolationId`s for the
  same real-world fact. Observation source: direct reproduction against
  `hunyi::check_trait_impl_locality` — described above.
  Current bound: `target`/`rule_key` in `crates/hunyi/src/trait_impl.rs` are built from
  `canonical_path_str(&boundary.trait_path)` — raw-identifier stripping only, never
  re-export resolution — even though the same function already computes a
  re-export-resolved `true_anchors`/`canonical` value for match-decision purposes,
  unused for identity. Risk: real baseline-stability consequence — renaming a
  constitution declaration from a re-export spelling to the canonical spelling (a pure
  refactor, no code behavior change) silently flips every affected violation's identity
  and defeats the baseline. Promotion trigger: thread the already-computed resolved
  anchor into `target`/`rule_key` instead of the raw declared string — real design work,
  since `true_anchors` is a cfg-blind multi-candidate set and picking one deterministic
  canonical member for identity purposes is itself a design decision. Version class:
  DESIGN-BREAKING. Authority: this entry's own reproduction record (above).

- ~~**`OriginEntry`'s public constructor lets any code in the process assert an arbitrary runtime
  origin, defeating origin-based fail-closed confinement.**~~ **CLOSED** in the 0.4.0 window, by the
  `louke-origin-derived-from-type` change. Closed by removing the caller's input rather than by
  guarding it: the expansion target is now generic over the registered type and takes **no
  arguments**, so an entry's whole content is a function of that type and an origin the type does not
  have is unrepresentable. The original entry is kept below for its reproduction record and for the two
  dead ends it cost, both of which were recorded as viable before being tested.

- **`OriginEntry`'s public constructor lets any code in the process assert an arbitrary runtime
  origin, defeating origin-based fail-closed confinement.** (Spelled `OriginEntry::new` when this
  entry was written; renamed `__from_register_origin` and made argument-free in the 0.4.0 window.)
  Class: DESIGN-BREAKING.
  Observed pressure: verified real during 0.3.1 sweep cleanup (2026-08-02/03) — a
  hand-built `OriginEntry::new(TypeId::of::<RogueAdapter>(), "loukehot::good", "RogueAdapter")`
  passed to `install` alongside genuine `register_origin!` entries produces zero
  reaction for a seam declared `.only_origins(["loukehot::good"])`, even though
  `RogueAdapter` never legitimately registered that origin. Observation source: direct
  reproduction against the real `louke::install`/`assert_boundary!` public API, and later an in-tree
  test (`a_forged_origin_silently_passes_a_seam_its_type_may_not_cross`, since removed with the
  gap it pinned) that reproduced the same
  silent pass through the registry the way `install` builds it, then stopped **compiling** when the
  closure landed — that transition being the evidence, not a passing assertion.
  Risk: HIGH — it defeated the crate's core stated
  guarantee outright for any code sharing the process, not merely a narrow idiom;
  unlike the other entries here, it was a capability gap in the trust boundary
  itself, not an identity-collision edge case.
  **Two recorded promotion triggers turned out not to work, and both were recorded before being
  tested** — the lasting lesson of this entry. (a) `pub` → `pub(crate)` on the constructor breaks the
  legitimate `register_origin!` path, since `macro_rules!` visibility is checked at the expansion site.
  (b) A `#[track_caller]`/`std::panic::Location` redesign yields a *file path* where an origin's whole
  vocabulary is a module path. (c) A **proc-macro** does not help either: it is expanded into its
  caller's crate and resolved there exactly as a `macro_rules!` is, so a private constructor fails with
  `error[E0603]` at the adopter's own call (three-crate probe). All three share one shape — hunting for
  a macro that can pass something hand-written code cannot. No such macro exists, which is why the
  closure had to stop taking the origin from the caller at all. A backlog entry naming a fix that
  cannot be built is worse than one naming none: the next reader spends the budget before finding out.
  Authority: `openspec/specs/runtime-origin-assertion/spec.md`; this entry's own reproduction record.


- ~~**Only the ONE resolved crate root of a package is governed; its other compiled roots are not
  observed at all.**~~ **CLOSED** in the 0.4.0 window. Every compiled root is now its own corpus in both
  static dimensions, and an observation carries the compilation unit it came from as an identity role, so
  the same violation in two roots stays two facts rather than one that masks the other.
  Three measurements changed the shape of the work and are worth keeping: 漏刻 ALREADY governed every root
  (`member_root_files`, in production), so this was two dimensions diverging from a third rather than a
  family-wide bound — correcting this entry's own earlier claim that the scope question was shared because
  渾儀 uses the same single-root function, which was true of 渾儀 and never checked against 漏刻; a spike over
  the existing machinery passed 316 of 圭表's 317 tests, so the corpus model tolerated N roots and the
  remaining work was identity, not a rewrite; and a target's NAME turned out not to be unique within a
  package (this repository builds a `lib` and a `bin` both named `tianheng`), so the role is the root's
  path relative to the package directory.
  Two things the work found that the entry had not predicted: a sibling conventional root leaks into every
  other root's walk (a top-level `lib.rs`/`main.rs` is `crate` in each), which without exclusion reported
  one violation once per root — a duplicate worse than the false negative being closed; and a per-root
  loop that deferred ANY error swallowed genuine scan failures whenever a sibling root happened to be
  governable, which is a silent pass. Both are fixed and pinned. The original entry is kept below for its
  reproduction record.

- **Only the ONE resolved crate root of a package is governed; its other compiled roots are not
  observed at all.** Class: DESIGN-BREAKING. Observed pressure: promoted in the 0.4.0 window from a
  single-lens WATCH hypothesis and a one-line `ACCEPTED DEBT` mention ("multi-target conventional-path
  conflation"), both of which understated it. Measured through three independent lenses:
  (1) **mechanism** — `xingbiao::crate_root_file` returns the first library-kind target, else the first
  `bin`, one root by construction, and 圭表's `package_src_dir` plus reachability walk both start there;
  (2) **minimal shape** — a package of exactly `src/lib.rs` + `src/main.rs` with the identical offending
  construct in both files and a boundary on `crate` reports ONE violation, in `lib.rs`;
  (3) **broader shape** — `src/main.rs`, `src/bin/conventional.rs`, a `[[bin]] path` inside `src/`, and
  a `[[bin]] path` outside it are all unobserved when a library root exists. Observation source: those
  measurements, pinned in both directions (the resolved root reacts; the others are silent; a
  library-less package governs its first `bin`) by what is now
  `crates/guibiao/tests/per_target_corpus.rs` — renamed and inverted when this entry was closed, since
  the transition those assertions existed to detect is what happened.
  Current bound: stated in `module-boundary`'s single-governed-root requirement, replacing a
  requirement whose premise — "both crate roots (`lib.rs` and `main.rs`) resolve to `crate`" — was
  factually wrong, and whose recorded limitation (a cross-root same-named submodule) was a narrower
  story than the truth. Risk: HIGH by class, and it lands on the most ordinary Rust package shape — a
  library beside its binary. A real violation written in `main.rs` passes silently, which is the one bug
  class the Core Contract forbids; what limits it in practice is that the governed root is usually where
  the architecture lives, and that a boundary an adopter declares on `crate` still reacts to everything
  the library reaches. Promotion trigger: **per-target module graphs**. Two roots of one package both
  denote module path `crate`, so observing both raises a violation-identity question the current model
  does not answer (which `crate` a finding names) — the same reason the requirement this replaces
  already named it an amendment beyond the conventional-path scanner. Note also that 渾儀 resolves roots
  through the same `crate_root_file`, so the scope question is shared rather than 圭表-local. Version
  class: DESIGN-BREAKING (a per-target corpus changes which facts exist, and plausibly their target
  role). Authority: `openspec/specs/module-boundary/spec.md`'s single-governed-root requirement and its
  two scenarios; the pinned test above.

- ~~**An inbound rule's target match is namespace-blind, so a name declared in two namespaces at once is
  observed only as its module.**~~ **CLOSED** in the 0.4.0 window. Closed by observing the value
  namespace, not by unioning both readings — the entry's own "deliberately NOT" still holds, and that is
  what keeps `shallow_inbound_rules_protect_only_the_exact_module` passing unchanged: an import reacts
  only when the anchored module really declares a `fn`/`const`/`static` of that final segment, so an
  ordinary `use m::child;` naming only a module stays silent.

  The promotion trigger asked for "a value-namespace item observation for the anchored module" as though
  it had to be built. **It already existed**, one module away: `symbol_scan`'s definition collector reads
  every module's own top-level `fn`/`const`/`static` from declaration-cleaned source, with the
  true-inline-module keying and module-top-level-only disciplines already worked out for the
  strict-external local-precedence ladder. The trigger was a missing *connection*, not a missing
  capability — the same misjudged cost as the absolute-`#[path]` entry earlier in this window, whose
  trigger read as a four-function refactor and turned out to be one line. Worth recording as a pattern:
  a promotion trigger written from inside the code that lacks the observation tends to describe building
  it rather than looking for it.

  The residual is now the observation rather than the resolution, and is narrower than the entry's own
  note: a value declared inside a macro body or arriving through a re-export is unobserved, matching
  every other declaration reader in the dimension, and directs the reaction toward the module reading —
  the pre-existing behaviour, not a new gap. `rule-model-surface` states it with two scenarios, and the
  pinned test is inverted rather than deleted (`shallow_inbound_target_match_observes_the_value_namespace`).
  The original entry is kept below for its reproduction record.


- **An inbound rule's target match is namespace-blind, so a name declared in two namespaces at
  once is observed only as its module.** Class: READY-PATCH (the correction only adds reactions;
  no public API and no identity shape changes). Observed pressure: raised by an independent review
  of the 0.4.0 window against `crates/guibiao/src/module_check.rs`'s `resolve_import_module`, then
  verified against **rustc** rather than reasoned — a crate declaring `pub mod foo;` and
  `pub fn foo()` in one module compiles, and a single `use crate::internal::foo;` in another module
  binds both bindings (the importer can call `foo()` *and* reach `foo::INSIDE` from that one
  `use`). Observation source: that build, plus the pinned test
  `shallow_inbound_target_match_is_namespace_blind_a_stated_bound`
  (`crates/guibiao/src/tests/symbol_confinement.rs`). Current bound: the resolver sees only the
  path, so it returns the longest reachable module — the module reading — and the value reading is
  unobserved. The two disagree only under `ScanDepth::Shallow` anchored at that module's own
  parent: the value reading reaches the anchored module and should react, the module reading
  resolves to the descendant and does not. Under `Subtree` both readings lie within the anchored
  module, so nothing is lost. Risk: LOW-but-real — a false negative, the one class the Core
  Contract forbids reacting silently in, confined to one exotic shape (a module and a value
  sharing a name in the protected module) in one depth cell. Deliberately NOT closed by reacting
  on both readings: that makes every ordinary `use m::child;` react under `Shallow`, contradicting
  `rule-model-surface`'s own exact-seam scenario — a narrow false negative must not be traded for
  a broad false positive. Promotion trigger: a value-namespace item observation for the anchored
  module (`fn`/`const`/`static` declared directly in it, inline `mod` bodies included), so both
  readings are unioned exactly when the ambiguity is real; note it carries its own bounds a
  macro-generated or `pub use`-re-exported `fn` would still escape, which must be stated with it.
  Version class: PATCH. Authority: `openspec/specs/rule-model-surface/spec.md`'s stated
  namespace-blind bound and its scenario; the rustc verification and pinned test above.

## Version horizons

The version follows SemVer honesty (`AGENTS.md`), not milestone size: **non-breaking →
patch, breaking → minor**, and never a vanity minor bump. `AGENTS.md`'s *Versioning* section owns what
counts as breaking — any change the adopter has to act on, a stale recorded baseline included — so read
it before assigning a horizon here; the entries below are horizons, not a second definition.

- **0.2.x (shipped)** — additive depth on an existing observation source, false-negative closures. A
  historical record, not a precedent: those closures were classified as patch-class before the 0.4.0
  window settled that a change requiring adopter action earns a minor. The same work today is
  minor-class.
- **0.3.0 (shipped)** — stable rule identity (`RuleKey`), `StructuredFactIdentity`, unsafe-site decomposition, async seam identity.
- **0.4.0 (shipped)** — every compiled root governed, identity-coordinate completeness, the `cfg_if!`
  and conditional-remap conformance across all three dimensions.
- **The open window — now minor-class (`0.5.0`), no longer `0.4.1`.** It opened patch-class: packaging and
  hygiene, prose and specs, opt-in depth, performance, and diagnostics whose exit code and emitted documents
  do not move, with a false-negative closure explicitly deferred to the next minor. That deferral is what the
  window then spent. A bare-principal resolver closure landed carrying a `BREAKING CHANGE:` footer, and it
  earns a minor on the definition above rather than on its diff size. Stated against the **shipped** baseline,
  which is the only one a version answers to: in `0.4.0` a bare single-segment principal did not resolve at
  all, and now one the governed module declares does — new depth reacting by default, so a recorded baseline
  no longer describes the adopter's tree and regenerating it is work they did not choose. The window's own
  intermediate states (a fallback that over-reached, then the canonicalization that fixed it) are not the
  reason and must not be quoted as one: neither shipped, so neither is an upgrade anyone performs.
  `CHANGELOG.md` marks it `**BREAKING**` accordingly, and states the same delta from the same baseline.

  What earns the minor is one item, and it is named rather than counted: the bare-principal resolver closure in
  `crates/hunyi/src/crate_scope.rs`, the only change in the window that moves behaviour an adopter already had.
  The window has since added a substantial **additive** public surface — the observation protocol (`Observer`,
  `Run`, the three dimension observers), the typed bound model, and each dimension's `observation_bounds()` — which
  is minor-class on its own terms and does not change the reasoning above: additive API asks nothing of an adopter,
  while a recorded baseline going stale does. Re-derive the range rather than trusting a figure here — the counts
  this paragraph used to carry ("of the 44 commits … the two other product-code touches") were written early and
  falsified by the window itself:

  ```bash
  git rev-list --count v0.4.0..HEAD                                   # commits in the window
  git log --format='' --name-only v0.4.0..HEAD -- crates \
    | grep '/src/' | grep -v '/tests' | sort -u                       # packaged sources it touched
  ```
  The **branch now carries the number**: `release/0.4.1` was renamed to `release/0.5.0` on 2026-08-06, so the
  first squash target names the release it will become — the role and the result agree, which is what
  `AGENTS.md`'s branch rule asks of every branch. The rename was clean because nothing pointed at the old
  name: no open pull request targeted it, CI triggers on `main` and on any pull request rather than on a
  branch name, and the only prose naming it was this paragraph.

  What is deliberately **not** done yet: the workspace version is still `0.4.0`, and the bump, the dated
  CHANGELOG section, the internal pins, and `Cargo.lock` all move together at release preparation. Until then
  `crates/kanhe/tests/release_coherence.rs` reports `development: 0.4.0`, which is the coherent state for an open window —
  it reads versions, never a branch name, so the rename changed nothing it judges.
- **Next breaking window (if earned)** — requires real adopter or correctness pressure.

## Explicitly not on the roadmap

- Active code-shaping / generation.
- Prescriptive framework you build inside.
- Lints (opinionated style checks rather than declared intent).
- Universal graph API (whole-graph analysis rather than declared per-target boundaries).
- Supply-chain policy engine (cargo-deny's lane).
- DSL macro consolidation (repetitive builders are designed-to-be-imitated for 潛移 gravity; leave explicit).
