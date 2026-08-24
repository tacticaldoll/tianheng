# Backlog

Forward-looking work, deliberately deferred. Promote an item to an ordinary `<type>/<scope>-<slug>`
branch when you pick it up. Every future reaction obeys the drift law:

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
  published API and current baseline/report identity wire. **It classifies the evidence and the
  compatibility, not how much design the correction still needs.** An entry whose pressure is measured and
  whose fix breaks nothing is READY-PATCH even when the fix is a capability someone has yet to design —
  reading the class as "small" or "next" is what made one such entry sit here for a window declaring no
  class at all, because neither WATCH (which is for *thin* evidence) nor DESIGN-BREAKING (which is for a
  *migration*) described it and this one looked too large.
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
conclusion). **A sweep's findings are entries in this file, closed or open — there is no second queue.**
The `docs/audit/*.md` convention that used to promise each sweep its own dated file is retired: a closed
finding's substance is its closing PR and `CHANGELOG.md`, an open one is a `READY-PATCH` or `WATCH` entry
below with the observation source and trigger this file already asks for, and a separate file was a second
place holding the same two things. It also regenerated a hazard every campaign — the last one carried
hand-written line counts of live functions, unheld by anything, written by the same review round that was
removing that class elsewhere.

The `v0.4.0..release/0.5.0` merged-review campaign is the first to land under this rule, and it is now fully
closed: its repairs are `CHANGELOG.md`'s `### Self-governance` section and PRs #603, #604 and #605, and its
three orphaned spec requirements closed with the last of those. Its reproduction record is under *Closed —
reproduction records*; what it left standing is filed by class in the index below — the deferred structural
findings and the lifecycle-prose finding, both `WATCH`. **No live queue.**

## Live decision index

### DESIGN-BREAKING

**Empty.** Every item that stood here when the 0.4.0 window opened is closed; each one's reproduction
record is kept under *Closed — reproduction records* below, out of this queue so the index cannot read as open
migrations. Nothing currently requires a public or wire migration.

The `xuanji` sink for shared run/projection vocabulary remains *classed* DESIGN-BREAKING while sitting
under WATCH — its promotion trigger (a second standalone instrument consumer demonstrating the same
projection/run need) has not fired, and acting on it speculatively would break the proven standalone 圭表
consumer for an undemonstrated deduplication.

### ACCEPTED DEBT

- **A hand-maintained pin has no mechanism keeping it from rotting — the action SHAs, and now the Node
  version.** *Class:* ACCEPTED DEBT.
  *Observed pressure:* pinning `.github/workflows/ci.yml`'s `uses:` entries to commits closed the one
  ecosystem this repository resolved fresh on every run, and opened its dual — a SHA nobody refreshes drifts
  from the upstream fix it was pinned before. `node-version` joined them for the same reason and at the same
  cost: `'24'` resolved whatever 24.x the runner's mirror carried that day, so the interpreter executing the
  digest-pinned validator tree was itself repointable while the step beside it was named *pinned*. It reads
  `'24.16.0'` now, and inherits this entry rather than a promise that someone will notice. *Observation source:* the `v0.4.0..HEAD` static review, which
  found every action on a mutable major tag while `Cargo.lock` and `package-lock.json` pinned their own
  ecosystems by digest; the refresh gap is this repair's own consequence rather than a second finding.
  *Current reaction or bound:* none, and that is the entry. Nothing here reads the workflow, and no
  Dependabot or Renovate configuration exists in this repository. *Risk, bounded rather than assumed:* the
  actions are `actions/checkout` and `EmbarkStudios/cargo-deny-action`, both running under
  `permissions: contents: read` against a repository whose CI holds no secret beyond that grant. A stale
  checkout is a stale checkout; the failure it invites is missing an upstream fix, not executing something
  unchosen — which is the direction the pin closed.

  ***Promotion trigger, and it FIRED.*** As written, this entry promoted on *a pinned action falling far
  enough behind to miss a security advisory, or **a second ecosystem arriving whose pinning would want the
  same answer***. The Node interpreter pin is that second ecosystem, and it was added to this entry — the
  trigger fired in the same commit that extended the entry naming it, and nobody read it. Recorded here
  rather than repaired by rewording, because a trigger quietly widened after it fires is a trigger that
  never fires.

  *Disposition, 2026-08-24:* **the half with teeth was built, and no refresh bot was adopted.** The rot
  splits in two and only one half was open. Falling *behind within* a major is bounded — `engines.node` at
  `">=24 <25"` with `.npmrc`'s `engine-strict=true` — and the residue is a patch-level lag on a tree resolved
  by digest and run under `--ignore-scripts`. Running the interpreter *past the point its major is
  maintained* had nothing reacting to it at all, and `interpreter_support_window` now does: the pin declares
  the window it is good for, and the reaction refuses when that window is absent, doubled, unreadable,
  declared for a major the workflow no longer pins, or reached.

  *And the evaluation below was of the wrong tool for half of this entry.* Dependabot's `github-actions`
  ecosystem updates the action versions a workflow `uses:` — **not the inputs those actions take**, so
  `node-version:` is outside it. Adopting Dependabot would have closed the action-SHA half and left the Node
  half exactly where it was, which is not what the paragraph below assumes. Renovate reaches both, through
  its custom managers and a Node datasource. That does not change the decision, and it does change what the
  decision is *about*: the cost named below — a bot-authored pull-request stream entering a merge path whose
  wrapper judges every squash message against its pull request — is unchanged, and is now the whole of it,
  since the half a bot would uniquely buy is the one just closed by a reaction that needs no bot, no second
  author, and no network.

  *Promotion trigger, restated so it can fire again:* a pinned action falling far enough behind to miss a
  security advisory; a **third** hand-maintained pin arriving, since two were answerable one at a time and
  three is a mechanism; or the declared support window being reached with no maintained major to move to.
  Node's half of this debt is bounded differently from the actions': `package.json` declares
  `"node": ">=24 <25"` and `.npmrc` sets `engine-strict=true`, so a runner or a contributor arriving on
  another major **stops** rather than proceeding on different bytes — measured on npm 11.13.0, where an
  unsatisfiable `engines` warns and exits 0 without it and exits 1 naming both sides with it. The pin can
  therefore fall behind within its major, and cannot silently leave it.

  *Compatibility class:* patch; CI configuration ships in no crate. *Authority:* this entry, and the workflow
  comment's refresh recipe — `repos/<owner>/<repo>/commits/<tag>`, which dereferences an annotated tag to its
  commit where `git/ref/tags` returns the tag object.

  A refresh bot — Dependabot for the actions, Renovate for both — was considered and not adopted **now**, for
  a reason rather than by omission: it would add a configuration file, a bot-authored pull-request stream,
  and a second squash-message author to a repository whose merge path is a wrapper that judges every squash
  message against its pull request. That interaction is real work and belongs to a change of its own, not to
  the pin that created the need.

- **A check that never wrote a region decision is invisible.** *Class:* ACCEPTED DEBT. *Observed pressure:* the
  region classifier exists because six defects were one shape, and two more were found afterwards in a check
  that never adopted it. Those two were *wrong* decisions and a reaction can see a wrong decision; a check that
  simply never distinguished executed text from commentary has written nothing to see. *Observation source:*
  none, which is the entry. *Current reaction or bound:* declared as
  `repository-checks/a-check-that-should-distinguish-a-region-and-does-not-a-stated-bound`; the requirement
  obliges the classifier and the classifier's adoption is what narrows the class. *Risk:* **corrected below —
  the sentence that stood here was falsified by the first instance.** *Promotion trigger:* a defect of this
  class observed in the wild — a check reported clean over text its property did not cover. Note that a
  candidate was reported during the `0.5.0` review and **refuted by measurement**, so the trigger asks for a
  confirmed one. *Compatibility class:* patch; the checks ship in zero packages. *Authority:*
  `openspec/specs/repository-checks/spec.md`'s requirement on the region a check judges.

  The mechanization was designed and rejected rather than deferred: refusing the inline comment marker inside
  the checks would refuse more legitimate sites than defects, because some select commentary on purpose and
  others parse a data format whose syntax marks comments. That measurement is the reason this is debt rather
  than a gap someone forgot.

  **First instance, and it cost something.** A static review of the `v0.4.0..HEAD` window found
  `kanhe::manifest::workspace_version` reading raw lines — the one manifest reader both git-reading gates
  share, and the last one in the crate that had not adopted the classifier. Its sibling `package_name` had
  been repaired the same way already, so one root `Cargo.toml` was being scanned under two different region
  decisions inside a single judgement, which this requirement calls a defect by definition. Found by a
  reviewer, as the entry predicts: the absence had written nothing for a reaction to see.

  *What the instance changes:* the risk sentence, which said a check in this class **reads a wider corpus than
  its property, so it errs toward reporting more**. That was wrong in both directions at once here.
  `[workspace.package] # …` made the reader close the table before it opened, so the version read as
  **absent** — a narrower corpus, not a wider one — and `version = "X.Y.Z" # …` carried the comment into the
  value, so the version read as **malformed**. Both are legal TOML and both produced a false *refusal*, in
  front of `cargo publish` and the release gate, on the manifest line an author is most likely to annotate
  while bumping. So the direction is not "errs toward reporting more"; it is *whichever way the missing
  decision happens to fall*, and a false refusal at an irreversible act is not the cheap error the old
  sentence assumed.

  *What it does not change:* the class, and the stated trigger. The trigger asks for a check that reported
  **clean** over text its property did not cover, and this reported the opposite, so it is an instance of the
  class without being the promotion evidence the entry asks for. The mechanization measurement has not moved
  either — refusing the inline marker would still refuse more legitimate sites than defects — so this stays
  ACCEPTED DEBT rather than becoming a gap with a design. Whether a falsified risk sentence alone should
  re-open the class is a steward's call, recorded here rather than taken.

- **`examples/observer-participant`'s own test fixture was not migrated to `xingbiao::claim_scratch`,
  unlike every other scratch-root claim in the workspace.** *Class:* ACCEPTED DEBT. *Observed pressure:*
  an adversarial review of the whole `v0.4.0..HEAD` campaign found `examples/observer-participant/tests/
  reaction.rs`'s `Fixture::new` still does `remove_dir_all` then `create_dir_all(root.join("src"))` on a
  predictable `temp_dir().join(format!("house-rules-{name}-{pid}"))` path — the exact symlink-adoption
  race the `claim_scratch` migration closed at every other one of the ~40 call sites it touched.
  *Observation source:* read directly; confirmed every other workspace fixture with this shape now calls
  `claim_scratch` and this one does not. *Current reaction or bound:* none. *Risk:* low and unchanged from
  before the migration — developer-machine-only, requires a local attacker able to plant a symlink at a
  PID-guessable path before this one test runs, in an example crate that ships in no package. *Why debt,
  not a gap someone forgot:* `examples/observer-participant/Cargo.toml` states its design goal explicitly —
  "One dependency, and deliberately nothing else... an example that needed a new export would be proving
  the opposite of what it exists to prove" — and `claim_scratch` is not re-exported through `tianheng`'s
  public prelude. Migrating this fixture would require either adding a new export to `tianheng` just for
  an example's own test helper, or adding `xingbiao` as a second dependency, either of which contradicts
  the example's stated purpose. *Promotion trigger:* `xingbiao::claim_scratch` (or an equivalent) becoming
  reachable through `tianheng::prelude` for an unrelated reason, at which point migrating this one fixture
  would cost nothing further. *Version class:* patch; an example, shipping in no package. *Authority:*
  `examples/observer-participant/Cargo.toml`'s own header comment, the one place this constraint is
  declared.

### READY-PATCH

- **The `**BREAKING**` marking rule is paired per section, so an unmarked breaking entry beside a marked
  one is invisible.** *Class:* READY-PATCH. *Observed pressure:* `require_section_shape` collects
  `shape.breaking` as a set of **section names**, and requires each such section to carry a `### Migration`
  heading. One marked entry plus one Migration heading therefore satisfies the whole section however many
  unmarked breaking entries sit beside them. Measured on this repository's own `[0.5.0]`: its `### Migration`
  section instructed an adopter to change every `Outcome::Clean` pattern, while the entry that change belongs
  to carried no mark. The gate was green throughout, before and after the correction. *Observation source:* the `v0.4.0..HEAD`
  static review, and the correction it produced. *Current reaction or bound:* `release-coherence`'s
  section-shape requirement, which is what this entry says is too coarse; nothing else reads the marking.
  *Risk, bounded rather than assumed:* the information is **present but unannounced** — a reader who opens
  `### Migration` finds the instruction either way, so the failure is an adopter skimming `### Changed` for
  what affects them and not seeing the mark that exists to be skimmed for. It cannot make a released
  guarantee false. *Promotion trigger:* a second unmarked breaking entry, or an adopter reporting a missed
  migration. The first instance is recorded rather than promoted on, because the shape below is not yet
  decided. *Version class:* patch; a repository check shipping in no crate. *Authority:*
  `openspec/specs/release-coherence/spec.md`, and `CHANGELOG.md`'s own marking rule.

  *Shape.* The direction that actually matters — *an entry that should be marked and is not* — is undecidable
  by any reaction, because it asks whether a change requires adopter action. The candidates below work around
  that, and they separate on whether the rule is a **theorem** — so it can be a violation without inviting a
  false refusal — and whether it catches **the instance that produced this entry**.

  - **A migration section in a release section that marks nothing `**BREAKING**`.** A theorem — telling an
    adopter to act while declaring nothing breaking contradicts `CHANGELOG.md`'s own rule — and nearly free.
    It does **not** catch the observed instance: `[0.5.0]` carried one marked entry, and what went missing
    was the *second*. A check that cannot cover the case it was written from reads as coverage while
    providing none, and this one has zero observations of its own besides, so it is named here to be
    rejected rather than kept as a cheap option.
  - **The counting asymmetry** — a section whose Migration bullets outnumber its marked entries. Pure
    counting, no semantics, and it would have caught this instance. It is **not** a theorem: one breaking
    change may legitimately need two migration steps. So it cannot be a violation without the false refusal
    this repository forbids its gates, and `refusal::Kind` is not the place to put it — *cannot-judge* means
    the source could not be read, not that it was read and is ambiguous, and borrowing it here would blunt
    the one distinction the verdict channel exists to carry.
  - **A named join, which is a theorem and does catch it.** Require each Migration bullet to name the entry
    it migrates by a stable handle, and run the correspondence both ways. This is the shape
    `observation_bound_model.rs` already uses between a spec scenario and its typed declaration, and it
    turns detection into **declaration**, which is where this family puts enforcement: the obligation lands
    on the author, not on a reader of prose. Two bullets may share a handle and one bullet may carry two, so
    neither multiplicity is a false refusal. The cost is real and is an authoring-convention change: every
    future Migration bullet gains a handle, and `[0.5.0]`'s existing bullets would be backfilled.

    A Migration section may also carry a bullet that deliberately names **no** entry — `[0.5.0]`'s
    *Nothing else requires action* is one — so the join has to admit a declared-empty form, or such a bullet
    moves out of the section. Which of the two is part of the design rather than a detail after it, and it
    was invisible while this paragraph carried a hand-typed count that happened to omit that bullet.

  **An earlier wording of this paragraph refuted the join outright**, on the ground that pairing an entry to a
  Migration bullet needs text matching between two prose bullets. That is true only of the *implicit* form.
  Making the handle explicit is precisely how this repository escapes prose matching everywhere else, and
  refuting the whole candidate for the weakness of its weakest form would have told whoever picks this up
  that the road was closed.

  *Leaving the changelog entirely* — reading the **published surface** and requiring a mark wherever it
  changed incompatibly — remains the most direct instrument and carries a cost this entry first omitted:
  both `cargo public-api` and a raw rustdoc-JSON diff need **nightly**, while this workspace pins a stable
  MSRV and CI installs no nightly toolchain. Adding one is its own decision, not a step inside this repair.

  All of which is why the entry is filed at its measured size rather than repaired inside the release window
  that found it: the only candidate that is both a theorem and covers its own instance is a capability, not a
  tightening.

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
  and citing it was wrong. This reader's corpus is the three files `DIMENSIONS` names, none of which carries a
  string literal with a comment delimiter, so the register's measurement does not transfer in
  either direction. Two candidate closures, **neither adopted, and neither actually closes the class** —
  checked against the real corpus rather than left as an abstract choice. Requiring the anchor to be preceded
  by an `impl Observer for` line was measured against the three real files and refuted as stated: every real
  `fn bounds(&self)` sits several lines and one sibling method below its `impl Observer for` line (a doc
  comment and `fn observe` come between them), so a literal adjacency rule would refuse the real definitions
  too. A looser scope-containment version (nested inside *some* `impl Observer for … { … }` block, not
  necessarily adjacent) survives that check, but only raises the bar rather than closing the class: a
  perturbation that wraps the *whole* fake `impl Observer for` block — not just the `fn bounds` line — inside
  the same comment or string would still be accepted, because a scope tracker blind to comments and literals
  reads a fake `impl` line inside a comment exactly as it reads a real one. The other candidate — reading the
  definition the compiler resolves rather than a textual condition — needs real Rust parsing, which `kanhe`'s
  declared dependency allowlist (`AGENTS.self-law.md`) currently forbids; adopting it would first require
  amending that law, not just this reader.

  **Not a defect unique to this reader.** `crates/kanhe/src/region.rs`'s own `Executed` abstraction declares
  the identical residue for the same reason (a `/\* … \*/` span and a string-literal marker both need
  nested-span lexing this tree has "defeated repeatedly," in that module's own words) and cites this exact
  bound as its precedent — the citation was one-directional until the 0.5.0 window closed it: see the
  `observer-protocol` spec, now amended to cite `region.rs` back. Filing them separately let each be
  rediscovered as a fresh problem; they are one class. If this is ever closed, the closure is a **shared**
  nested-span lexer serving both sites, not a point patch to either — `guibiao::module_scan` already scans
  nested block comments and is the instrument `region.rs`'s own doc comment names as existing if the residue
  is ever worth closing.


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
  of the same shape — declared headings read from the spec, held
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
  source-shape reaction that claimed to observe it is retired in the 0.5.0 window after four review rounds each
  defeated the narrowing before it — by name resolution, by the parameter's binding site, by which definition is
  the subject, by the caller frame, and by execution, which no reading of text reaches. `observer-protocol`
  declares the resulting gap as an unpinned bound owned by the engine, and it is the tracker for this entry.
  *Observation source:* that bound and the retired reaction's history on `change/refuse-ambiguous-delegation-extent`.
  *Current reaction or bound:* the declared bound; no reaction. *Risk:* the shell grows a second semantic
  behaviour owner and nothing says so — the drift a seam exists to end. The dimension's *equality* is
  construction-held since the 0.5.0 window; its *delegation* is the seam that is not, and the two are one word
  apart. *Promotion trigger:* fired; the bound is declared unpinned, which the register leads with.
  *Version class:* patch if the composition is restructured without moving a public signature; minor if the
  shell's entry point changes shape. *Authority:* `observer-protocol`, whose spec states both the obligation and
  the retirement. *Shape:* **not** the runtime route, which the 0.5.0 window
  tried and measured wrong: invoking the observer makes the *equality* construction-held and leaves an
  independent shell decision as writable as before, because the runtime arm had a second implementation to
  collapse and this one never did. What would close it is the shell's semantic outcome being unreachable except
  through the observer — a shape in which the guard stops compiling rather than one in which it merely has
  nowhere tidy to sit.

- **Every normative SHALL either has a reaction or is a declared bound.** *Class:* READY-PATCH — by the
  definition above, which classifies evidence and compatibility rather than remaining design effort: the
  pressure is measured and the correction preserves every published API. It declared no class at all until
  the classification reaction was built, because it reads as too large for a heading that sounds like "next".
  *Observed pressure:* **ten** found
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
  un-reacted **after** the 0.5.0 window's sweep* — the sweep being the control, so the four found before it could not
  stand as evidence for themselves. Rounds 6 and 7 then found three more, and **all three were requirements this
  window had just written**: a SHALL added in one change and left unreacted, in the window whose whole subject was
  closing that class. That is stronger evidence than the original four, because it shows the class reproducing under
  authors actively watching for it. Re-review #3 then found the fourth through sixth post-sweep recurrences, again
  in requirements or scenarios written in the same window; two were removed as inert, and semantic delegation
  gained a source-shape reaction.

  *Current reaction or bound:* none. Only a **bound** carries a `PINNED-BY`; an ordinary requirement is bound to
  nothing, so no gate can tell a SHALL with a reaction from one without. *Risk:* the class recurring and being found
  by hand or not at all — a normative rule nothing enforces is indistinguishable from one that is enforced, which is
  the failure the bound register was built to end one level down. *Measured before promotion, not estimated —
  at `ee15665`, by `git grep` over `openspec/specs/*/spec.md`:* the specs held
  **1048** lines carrying `SHALL`, across **310** requirements and **1177** scenarios. The register, by contrast,
  currently holds **92 bounds across 25 capabilities** — a live figure rather than part of the measurement
  above, written in that exact form because it is the one phrasing
  `crates/kanhe/tests/bound_register.rs` reacts to, and a census in any other wording is what that gate's own policy says must
  not exist in prose. A citation per SHALL would add on the order of a thousand hand-maintained pointers, which is
  the drift class this family already refuses.

  *The binding is written in 62 places, and this entry said it was written nowhere.* Measured at `36152c0`, by
  `grep -c '^- \*\*PINNED-BY\*\*'` over `openspec/specs/*/spec.md` and a per-name resolution against every
  `fn` definition under `crates/` and `examples/`: **161** `PINNED-BY` citations, covering **62 of 364**
  requirements, and every one of the 161 names a test that exists. They are not unheld either —
  `crates/kanhe/tests/bound_register.rs` resolves each through `cargo test -p <member> -- --list`, per package,
  and refuses an empty member enumeration as the vacuity direction; `pin_bites` runs **13** of them against a
  perturbed tree and requires each to fail. The one ambiguous name carries a crate prefix
  (`hunyi::a_cfg_gated_module_with_no_file_is_skipped_not_errored`) because it is defined in two crates, so the
  convention already handles the case a bare citation could not.

  *First step, in two parts, and the order is what the measurement changes.* **One:** extend the existing
  citation to the requirements whose reaction already exists and is uncited. Sampled in `repository-checks`,
  where 18 of 32 requirements carry citations and 14 do not, every one of five sampled uncited requirements has
  a reaction — `gate_exit_classes`, `projection_register`, `dod_coherence`, `one_spelling`,
  `backlog_classification`. That half needs no capability, and it is what makes *no reaction* distinguishable
  from *a reaction nobody cited*, which is the state this entry is actually filed for. **Two:** for what a
  citation cannot reach — 302 requirements, and a citation per SHALL would be the thousand hand-maintained
  pointers this entry already refuses — find a derivation. That half is a capability to design, and designing
  it inside the closing review of a window would be the same haste this entry documents. What the earlier
  draft got wrong was not the second part; it was reaching it by asserting that the first part did not exist. *Version class:* not release-affecting; a new
  capability with its own gate, preserving every published API. *Authority:* `observation-bound-register`, which
  solves the same problem for bounds and is the shape any answer here would have to generalize.

  *Interim discipline:* `AGENTS.md` now requires a scenario entering main specs to name an existing reaction in
  the same change or arrive with a new guard and its negative run; a construction-guaranteed property stays in
  requirement prose instead. This does not close the entry — review convention cannot derive the missing binding —
  but it prevents sync from knowingly admitting another un-reacted scenario while the derived capability is designed.

- **kanhe's TOML readers hand-parse a grammar `region::toml()` already tokenizes but does not structure.**
  *Class:* READY-PATCH. *Observed pressure:* `release_coherence_gate.rs`'s `declared_dependencies`,
  `package_name` and `require_lock_versions`, and `prelude_promise.rs`'s `block_body`, each route through
  `kanhe::region::Source::of(text).toml()` for comment/string-stripping but still hand-split the resulting
  lines for table and block structure. `crates/kanhe/tests/release_coherence.rs`'s own doc comments record
  the resulting bug class as a still-open well, not a closed one: a `[lib]` name-line preceding `[package]`
  misread as the package name, single- versus double-quoted TOML string values silently unreadable, a
  `[[patch.unused]]` table's fields bleeding into the `[[package]]` block above it because the block boundary
  was the literal string `[[package]]` and nothing else, a commented-out dependency line counted as a real
  pin, and a comment glued to a value with zero whitespace tripping an older lexer. *Observation source:* a
  `v0.4.0..release/0.5.0` review that also sampled the most recent 60 commits on `release/0.5.0` and found 46
  (78%) carrying `kanhe` in their Conventional Commit scope and 38 (64%) typed `fix` — this entry's TOML
  readers are the still-unclosed half of that pattern; the Rust-side half (`refusal_register.rs`,
  `observer_protocol.rs`'s body-extent reader) closed via syn as a `[dev-dependencies]`-only addition,
  requiring no self-law amendment because `restrict_dependencies_to` observes only `DependencyKind::Normal`
  by default. *Current reaction or bound:* none declared in any capability spec; the bugs above are recorded
  only in doc comments and `CHANGELOG.md` history, not in `openspec/specs/repository-checks/spec.md` as a
  stated bound. *Risk, bounded rather than assumed:* kanhe ships in no package (`publish = false`), so a
  misread here misfires only this repository's own release-coherence gate — never an adopter's build or a
  published artifact. *Promotion trigger:* unlike the Rust-side closure above, these four functions run from
  non-test `crates/kanhe/src/` code that the `release-coherence` production gate calls directly, so a real
  TOML parser (`toml` or `toml_edit`) would land in kanhe's `[dependencies]`, not `[dev-dependencies]` —
  crossing kanhe's declared dependency allowlist (`AGENTS.self-law.md`, rendered from `crates/shengmo/src/law.rs`)
  and requiring the self-law amendment ritual `crates/kanhe/tests/self_law_amendment.rs` records, with steward
  review per `.github/CODEOWNERS`. The trigger is that amendment being proposed and accepted — this entry does
  not promise a minor release until it fires. *Compatibility class:* patch; the correction itself ships in no
  crate, but the prerequisite amendment is an architectural change decided by the steward, not a code patch.
  *Authority:* this entry, `AGENTS.self-law.md`, `crates/kanhe/tests/release_coherence.rs`'s documented bug
  history.

  **A third reader of the same class arrived, and it is on the cheap side of this entry's own dividing line.**
  `crates/kanhe/tests/merge_workflow.rs`'s `workflow_shape` reads `.github/workflows/ci.yml` line by line to
  hold the premise `require_ci_green` states to an operator. Seven consecutive adversarial review rounds each
  closed its finding and surfaced one more position in the same reader: the job-name indentation, the job-key
  indentation, the scope of the two trigger keys, the flow form at the top level, and the flow form one level
  down. Every repair was correct and every one was verified by a fixture row and a negative run; the count is
  the argument rather than any single miss. Two of the five failed **open** — the reader reported a premise
  intact over a workflow that carried a real path filter.

  **That severity has since been withdrawn, and the withdrawal is the useful part of this paragraph.** The
  premise existed because the wrapper's refusal asserted it to an operator; the classification never read the
  workflow at all. With that sentence removed the reader decides **when** an operator learns a job may skip,
  not **whether** — a skipping job reports `SKIPPED` and the wrapper refuses either way. So the two open
  positions cost minutes rather than admitting a merge, and a sixth would too.

  **What this entry establishes and the YAML case inherits:** the Rust-side half closed with `syn` as a
  `[dev-dependencies]`-only addition, needing no self-law amendment because `restrict_dependencies_to`
  observes `DependencyKind::Normal` by default. `workflow_shape` lives in `crates/kanhe/tests/`, so a YAML
  parser lands in the same table `syn` and `proc-macro2` already occupy. It is therefore **not** blocked on
  the amendment ritual this entry's TOML half is blocked on — the two halves have different prices, and only
  one of them was priced. *Promotion trigger for this third reader:* **a second file in
  `.github/workflows/`**, and not a sixth position in the reader. The severity above is per mechanism and only
  three of the five keys carry it unconditionally: `if:`, `needs:` and `continue-on-error:` move a check's
  conclusion, so the check reports `SKIPPED` and the wrapper refuses whatever the reader did. `paths:` and
  `paths-ignore:` stop the workflow triggering, so its checks are **absent** from the rollup — which refuses
  today only because `ci.yml` is the sole workflow and an empty rollup takes the *no workflow has claimed this
  head* arm. Add a second and a filtered-out `ci.yml` contributes nothing to a rollup that is non-empty and
  green, so a missed filter is a merge rather than a delay. That count is held by
  `a_missed_path_filter_costs_a_delay_only_while_one_workflow_exists` rather than assumed, which is what makes
  this trigger a reaction instead of a sentence. The TOML half being promoted would also earn it, since the
  two share the argument if not the dependency. *Version class:* patch; `kanhe` is `publish = false` and the reader is a test.
  *Deferred here rather than done* on the same ground as the wrapper extraction: a dependency is not added at
  a release cut, and the reader is currently correct on every shape a fixture can construct.

- **A claim about this tree, written as prose, is held only where its author declared it.** *Class:*
  READY-PATCH. *Observed pressure:* two claims were found false in the 0.5.0 window by the same shape — a
  statement about an enumerable property of this repository, written with no producer and an outer edge wider
  than anything that reacts. `Cargo.toml`'s *syn quarantined to 渾儀 alone*, false from the moment the
  dev-table edge landed in the same window; and `PROJECT.md`'s *zero change directories have ever existed*,
  contradicted by `git log --all`, by two commit bodies on the tip that wrote it, and by a requirement with a
  scenario in `openspec/specs/reference-integrity/spec.md`. Both were true of a corpus neither named. *Observation
  source:* an adversarial contract review of `v0.4.0..HEAD`, plus this file's and `CHANGELOG.md`'s own record of
  the figure form of the same class — eight hand-written figures found wrong in a single change, which is what
  `crates/kanhe/src/census.rs` was built for. *Current reaction or bound:* `census` holds a **figure** written in
  a sentence its enumerating check declared, over tracked Markdown, and `repository-checks` declares two bounds
  for what that leaves — a count in a phrasing no census declares, and a census outside Markdown. Nothing at all
  holds a **set-membership or absolute** claim (`only X`, `never Y`, `alone`), in any carrier. *Risk:* the
  projected prose is what conditions every agent that loads it, so a claim wider than its reaction is a false
  statement of the law at the surface 潛移 makes most load-bearing. Bounded by the claims being prose: nothing an
  adopter resolves, and no exit class moves. *Promotion trigger:* a third instance, or the design below being
  written. *Version class:* patch; the carriers ship in no crate. *Authority:* this entry, `AGENTS.md`'s *Bind a
  claim to its measurement* and *A census is produced, never typed*, and `crates/kanhe/src/census.rs`.

  *Shape, because the obvious instrument is already refused:* a detector over prose was designed, measured three
  times and rejected, and that refusal has an observation source this entry does not get to ignore. The admitted
  shape is the one already built for figures — **declaration**: `Census` carries a phrase and produced figures,
  and the missing sibling is a declared phrase whose held value is a produced **set**, so *only 渾儀 names syn*
  is compared against the enumerator that answers it. What that cannot cover is stated here rather than
  discovered later: it reaches only claims some check enumerates the set for, so a role description like *the
  syn dependency lives here* stays a reviewer's, and coverage stays opt-in — declaring is an author's act. That
  residual is the honest floor of this repair, not an argument against it.


### WATCH / ACCEPTED / DECLINED / BUILT

- **`merge_workflow`'s fixture takes `jq` from the host without declaring it, and its absence reads as fifteen
  defects in the subject.** *Class:* WATCH. *Observed pressure:* measured 2026-08-23 on a machine where `jq`
  is not installed — 15 of that target's 30 cases fail, each reporting
  `bin/gh: line 77: jq: command not found` alongside the merge message *cannot read what CI said about this
  pull request, which is not the same fact as CI having agreed*. **The wrapper is right and the fixture is
  not:** `scripts/merge-pr.sh` could not read a verdict, so it declined to judge and said so — exactly its
  contract. What is wrong is that the fixture's `gh` stub pipes through a host tool it never declares, so a
  missing interpreter presents as a failing subject, and the operator reads fifteen findings about CI when
  the state met was one absent binary. Same shape this window closed twice elsewhere: a diagnosis SHALL state
  what to do about the state it met, not assert what is true of the world. *Observation source:* the
  `v0.4.0..release/0.5.0` campaign's verification runs, where the suite passed earlier the same day and the
  binary disappeared beneath it. *Current reaction or bound:* none — no capability spec states the host tools
  a reaction requires, and nothing checks for one before the reaction runs. *Risk, bounded rather than
  assumed:* local only. GitHub's `ubuntu-latest` images ship `jq`, so CI is unaffected, and `kanhe` publishes
  nothing — no adopter build and no released artifact can meet this. *Promotion trigger:* a second reaction
  fixture found taking a host tool it does not declare, or a CI image that stops shipping `jq`. Either makes
  this a class rather than one stub, and the repair is then the same for both: the fixture states what it
  needs and stops before the subject when it is absent.

  *Trigger measured, not waited for — at `36152c0`, and it has not fired.* The decidable half of it is
  enumerable: a stub is executed only where a test puts its directory on `PATH`, and exactly **two** test
  targets do — `publish_workflow` and `merge_workflow`. `merge_workflow`'s `gh` stub pipes through `jq`, which
  is this entry. `publish_workflow`'s `cargo` stub uses `printf` and `[[` and reaches no host tool at all. Every
  other occurrence of a host-tool name under `crates/*/tests` is either prose or a fixture's *subject* rather
  than something executed — `echo tail # cut` and `curl "$url#frag"` in `source_regions`, which are text the
  region classifier reads. So the class has one instance by measurement rather than by nobody having looked,
  and the `PATH` criterion is what a later sweep should re-run rather than a tool-name list, which cannot be
  known to be complete.

- **WATCH: a corrupt tag ref and an absent one are one exit status, so the publish gate cannot tell them
  apart.** *Class:* WATCH. *Observed pressure:* the tag read was split so that git declining to answer is no
  longer reported as *there is no tag*, and one residue survives the split: a ref FILE holding unparseable
  content exits the same status as a ref that does not exist. *Observation source, reconstructible:* measured
  on 2026-08-21 in a scratch repository —

  ```bash
  git init -q . && git commit -q --allow-empty -m x
  printf 'garbage\n' > .git/refs/tags/broken
  git rev-parse --verify --quiet refs/tags/broken; echo $?   # 1
  git rev-parse --verify --quiet refs/tags/absent;  echo $?   # 1
  ```

  and the bare form collapses them differently rather than better: both answer `128`, which is also what a
  directory that is no repository answers. *Current reaction or bound:* the gate reports **absent** for both,
  which is the safe direction — it refuses the publish either way, and the operator's next act is to look at
  the tag. Not declared as an observation bound, deliberately: a declared bound is pinned by a direction over
  its own WHEN, and this WHEN produces the same answer as the case beside it, so a pin would compare a value
  with itself. *Risk:* an operator told *there is no tag* when the tag's ref is corrupt goes to create one and
  meets a ref that already exists. Bounded to a diagnostic: no publish proceeds on either reading. *Promotion
  trigger:* a git version or a read that separates the two — `for-each-ref` and `cat-file` were both measured
  and neither does at this layer — or one occurrence of a corrupt ref in front of a real release. *Version
  class:* patch; the gate ships in no package. *Authority:* `publish-source-integrity`, whose class rule this
  residue sits inside rather than outside.

  A neighbouring case IS separated and is recorded so the two are not confused: a ref holding a well-formed
  sha with no object behind it exits `0`, is read as present, and the tag-object read downstream refuses it as
  unreadable.

- **WATCH: no reaction asks what each repository check observed, and the check surface is now an order of
  magnitude larger than the product it guards.** *Class:* WATCH. *Observed pressure:* the drift law — *no
  target or name without a reaction*, *no drift type without an observation source* — is stated over the
  product's boundaries and has never been turned on the repository checks themselves. Every check has a
  reason written in its header; nothing enumerates which ones have an **observation source**, in the sense
  this file demands of a backlog entry: a thing that happened, not a shape that could. *Observation source,
  reconstructible:* measured at `HEAD` on 2026-08-21, from **one** enumeration so the two sides cannot be
  counted under different rules:

  ```bash
  git diff --numstat --no-renames v0.4.0..HEAD -- crates > /tmp/n
  grep '/src/' /tmp/n | grep -vE 'crates/(kanhe|shengmo)/' | grep -vE '/tests?(\.rs|/)' \
    | awk '{a+=$1} END{print a}'                                          # product logic: +2230
  grep -E 'crates/(kanhe|shengmo)/' /tmp/n | awk '{a+=$1} END{print a}'   # machinery:     +28687
  ```

  `--no-renames` is load-bearing rather than decorative: nine fixture moves render as
  `crates/{tianheng => shengmo}/…`, which a path-shaped filter reads and a name-shaped one does not, and the
  two forms disagreed on both figures until the enumeration was pinned. No file count is given — the ratio is
  the claim, and the counts were the half that broke.
  and against this file's own anchored figure for the same window — 46 of the last 60 commits (78%) scoped
  `kanhe`, 38 (64%) typed `fix`. *Current reaction or bound:* none. `observation-bound-register` enumerates
  where each check *stops*, which is the opposite question; nothing enumerates why each check *exists*.
  *Risk, and both readings stated because the evidence does not choose between them:* the ratio is what
  dogfooding a governance thesis looks like, and the checks with a recorded observation source — a release
  dated four days behind the day it would be cut, a publish from the wrong branch, nine squash subjects
  carrying `(#N)`, a prelude promise narrowing unobserved — are the argument for it. The risk is the
  remainder: a check built from a shape someone imagined would be exactly the name-without-an-observation
  the drift law refuses in product code, and it would be indistinguishable from the others by reading, since
  every header reasons well. *Promotion trigger:* an audit that asks each `crates/{kanhe,shengmo}/tests/*`
  target *what did this catch, and when* — enumerated from tracked content the way
  `observation-bound-register` is, never hand-listed — and finds a target with no answer. Deliberately not
  attempted here: it is the audit-cycle shape `PROJECT.md` records, whose own decision says larger surfaces
  are named and not committed to, and running it is a window's work rather than a review round's. *Version
  class:* patch; both crates ship in no package, so any retirement it produced would reach no adopter.
  *Authority:* this entry, `PROJECT.md`'s drift law and its audit-cycle decision, `governance-dogfood`.

- **WATCH: the prose-claim class is filed but not closed, and no rule says how much of a class must react.**
  *Class:* WATCH. *Observed pressure:* `AGENTS.md` says a governance rule measured un-reacted is *given a
  reaction or filed, in the same change* — and filing counts as answering. The `READY-PATCH` entry above
  discharges that duty for the prose-claim class by proposing the declaration; this one records that the
  declaration does not close it, because its coverage is an author's act. What no rule in this repository
  answers is when a class whose only instrument is opt-in has been answered *enough*. *Observation source:*
  this file's own history of the figure form of the same class — declared 2026-08, reacted for one sentence,
  and the eight-figure change that followed it. *Current reaction or bound:* none, and by construction: this
  is a question about the coverage of a discipline, not about a shape anything observes. *Risk:* the class is
  rediscovered once a window and each rediscovery pays the measurement again; bounded by every carrier being
  prose. *Promotion trigger:* a third window in which this class costs a review round after the `READY-PATCH`
  above lands. *Version class:* patch. *Authority:* `AGENTS.md`'s *A repair loop is a diagnosis, not a
  schedule*.

- **WATCH: Four Markdown readers still take a bare `&str` where `region` exists.** *Class:* WATCH.
  *Observed pressure:* `region`'s own header made this an absolute — *a corpus is never handed to a
  recognizer as `&str`* — and four readers in the same crate contradict it:
  `release_coherence_gate::{require_changelog_state, require_section_shape, unreleased_has_item}` and
  `restatement::document_offences`. A fenced `## [Unreleased]` or `### Added` would be read as the section it
  resembles, and a fenced block naming a crate together with every member of its dependency allowlist would
  be read as a restatement. *Observation source:* a review read the sentence against the crate, and the
  sentence is now narrowed to executed text with the residue stated where it was claimed away.
  *Current reaction or bound:* the misread itself is unheld; its **latency** is held —
  `the_corpora_of_the_bare_str_markdown_readers_carry_no_fence_or_comment_span` produces the figures this
  entry used to type, over `CHANGELOG.md` and every `openspec/specs/*/spec.md`, with a vacuity guard on each
  half. Not covered there: `document_offences`'s wider corpus, whose residue is conditional rather than a
  count — a fenced block naming a crate together with *every* member of its allowlist — since holding that
  means running the restatement rule inside a fence, which is the reader this entry exists to replace.
  *Risk:* a release gate reading a fenced heading as a section, or a restatement check skipping a document it
  should judge — both toward a false pass, which is the direction the Core Contract forbids.
  *Promotion trigger:* that direction failing. It was *the first fenced block in `CHANGELOG.md` or in a spec*
  — a trigger only a person re-reading the tree could notice, over three zeroes measured once, which is a
  premise with no observation source and the class this repository refuses everywhere else. *Version class:* patch; every site is in a crate that ships in no package.
  *Authority:* `release-coherence` and `self-law-projection`. *Shape:* give `Prose` a numbered form —
  `numbered_lines`, as `Executed` already has — then take `Prose` in the four signatures.
  `document_offences` reports the line a block starts at, so the numbered form is the prerequisite and not a
  nicety. Filed rather than done because the release is being cut and the four signatures ripple into their
  callers, where a latent misread does not.

- **WATCH: The Definition of Done and CI mirror each other in one direction only.** *Class:* WATCH.
  *Observed pressure:* `dod_coherence` asserts that every command in `AGENTS.md`'s list appears in CI, and
  nothing asserts the converse. *Observation source:* a review measured it, and a record in this window had
  already asserted the converse existed — three named CI steps run suites the local list does not name
  (`release_coherence`/`publish_source`, `dod_coherence`, `bound_register`), all covered today by the listed
  `cargo test --workspace --all-features`, and `cargo test -p louke` is the one named step that run does not
  cover. It is in the list because a person remembered. *Current reaction or bound:* the one direction.
  *Risk:* the DoD's own prose records this exact class — *the gate a contributor actually runs went blind to
  the exact class this job had just learned to catch* — and the residue is a named CI step whose local
  coverage nothing holds. *Promotion trigger:* a second CI step that the workspace run does not cover, or a
  contributor's green local run followed by a red CI job on a step the list does not name. *Version class:*
  patch; governance and CI configuration only. *Authority:* `repository-checks`. *Shape:* assert the converse
  — every named CI step running a suite is either in the list or covered by a listed command — which needs a
  reader that can say which suites a listed command covers, and that is the half not built.

- **WATCH: The two irreversible-act wrappers are one lifecycle written twice.** *Class:* WATCH. *Observed
  pressure:* `scripts/merge-pr.sh` and `scripts/publish.sh` share a whole shape — resolve the repository root,
  parse an argument allowlist, open a verdict channel, run the gate, read the class, clean up, `exec` the
  tool — and a review measured most of `publish.sh`'s code as byte-identical to its sibling, including
  every named construct they share (`cannot_judge`, `refuse`, `require_a_verdict`, `require_one_pass`, the
  ERR trap with its bash-5 measurement paragraph, and the mktemp/EXIT-trap/pre-exec `rm` sequence) plus both
  class constants and their diagnostics. **No count is given**: this entry first carried one, and the very
  repair recorded two sentences below it — `merge-pr.sh` gaining `refuse` — moved the figure in the commit
  that wrote it. The trigger reads the shape, not the arithmetic. *Observation source:* that review, plus the drift it already
  produced and which is repaired with it — `publish.sh` routed every argument refusal through a `refuse()`
  helper while `merge-pr.sh` spelled `printf … >&2; exit 2` inline at every arm, against
  `repository-checks`'s own clause that *the classification SHALL be chosen in one place per wrapper*.
  `merge-pr.sh` now has that helper. *Current reaction or bound:* `gate_exit_classes` holds the class
  constants and the presence of `require_one_pass` across both wrappers, with set equality in both
  directions — so the **identities** cannot drift; nothing holds the bodies or the reasoning, which is what
  did. *Risk:* the cost is recorded verbatim by `hermetic_git.rs`'s own header for its own extraction — *the
  undocumented copy was then given a doc written without reading the other, and that doc overclaimed*.
  *Promotion trigger:* a second divergence between the two wrappers' shared constructs, or a defect found in
  one copy and not the other. *Version class:* patch; both scripts reach no tarball. *Authority:*
  `repository-checks`. *Shape:* one sourced library under `scripts/` holding the shared constructs and the
  verdict-file lifecycle, with `gate_exit_classes` widened to assert **one** definition site rather than two
  agreeing copies. Filed rather than done because the extraction lands at a release cut and the two
  wrappers stand in front of the two acts that cannot be undone — the one place where a refactor's own risk
  outweighs the drift it removes until the release is out.

  **The trigger has fired, and what answered it is the narrower half.** A `v0.4.0..HEAD` review found the
  second form this entry names — *a defect found in one copy and not the other*: every refusal in
  `scripts/publish.sh` reached its class through a helper, while `scripts/merge-pr.sh` chose it inline at four
  sites, two of them defined above the helpers they should have called. Both wrappers now delegate every stop
  to a single `cannot_judge`, so each spells its diagnostic prefix once, and
  `each_wrapper_chooses_its_exit_class_in_one_place` counts the `exit` statements in executed shell text —
  which turns *chosen in one place* from a property a reader checks into one a run decides. **The extraction
  itself is still not done**, and the reason above is unchanged: it lands at a release cut, in front of the two
  acts that cannot be undone. The convergence performed was inside each wrapper, adding no file and no
  `source` path to resolve; the cross-wrapper library this entry proposes is what stays filed. Saying which
  half a change did is what *A repair loop is a diagnosis* asks for, so it is said here rather than left to
  the next review to re-derive.

  **Two review rounds later the copies still have not diverged, which reads as the trigger not-yet-fired
  rather than as safe.** Four changes have touched `scripts/merge-pr.sh` since — the exit class, the
  no-evidence class, a relative anchor, a reflowed comment — and every one landed outside the shared region or
  was applied to both wrappers in the same commit. Measured at `8604a05`: of `scripts/publish.sh`'s 109
  executed lines, 66 appear verbatim in its sibling. So the identities agree and the bodies agree, and nothing
  made either of those happen — a reviewer did. An adversarial review named the consequence precisely: the
  copies not having diverged is what makes the extraction **cheap now and expensive later**. That is an
  argument about timing rather than against the reason this stays filed, which is unchanged — it lands at a
  release cut, in front of the two acts that cannot be undone. Promoting it is the first work of the window
  after this release rather than the last of this one.

- **WATCH: A branch name is governed by prose alone, and the sentence saying so was wrong about why.**
  *Class:* WATCH. *Observed pressure:* `AGENTS.md` requires `<type>/<scope>-<slug>` with `<type>` drawn from
  the Conventional Commit set, and nothing reads a branch name at any point. Measured over every merged pull
  request into `release/0.5.0`: **84 head branches open with `change/`** — the role `AGENTS.md` itself records
  as retired — and **seven carry no `/` at all** (`release-prep`, `refusal-register-reads-rust-not-text`,
  `docs-agents-bind-a-claim-to-its-measurement` among them), against 180 `fix/`, 69 `docs/`, 17 `test/`,
  10 `feat/`, 9 `refactor/`, 2 `ci/` and 1 `build/`. Those are history and cannot be repaired; what they
  measure is that the convention drifts unobserved, which is the condition under which `AGENTS.md` says a
  governance rule *is read as license rather than law*. *Observation source:* measured at `94a42fb` by
  `gh pr list --state merged --base release/0.5.0 --json headRefName`. *Current reaction or bound:* none. The
  sentence that declined one has been repaired in the same change for a different fault — it gave the
  constitution's reason for a repository check's conclusion, and compared itself to a rule that has since
  acquired a gate — so the decision is now open rather than settled by a premise that did not hold.
  *Risk:* low and bounded, since a branch name reaches no artifact and no adopter; the cost is that the type a
  branch declares and the type its squash lands as can disagree, which is the disagreement the naming rule
  exists to prevent. *Promotion trigger:* a squash whose subject type differs from its branch's, or a second
  retired role accumulating. *Version class:* patch; no crate is touched. *Authority:* `repository-checks`.
  *Shape:* `scripts/merge-pr.sh` already resolves the pull request and could read `headRefName` beside the
  title it re-reads, and `merge_message_gate` already owns the admitted type set — so the judgement is one
  comparison against a list that exists, not a new vocabulary. What it needs first is a decision about the
  84 historical `change/` branches: a check that refused them would refuse nothing live, but a reader meeting
  it needs to know the rule governs new branches rather than the record.

- **ACCEPTED DEBT: A release date is only held at the snapshot, and an earlier check would be noise.**
  *Class:* ACCEPTED DEBT. *Observed pressure:* the dated CHANGELOG section for the version under preparation
  carries a date nothing compares until the `release: X.Y.Z` commit exists. Measured on this repository: the
  `0.5.0` section stood at `2026-08-16` while the branch tip was six days later, and one earlier release was
  prepared four days behind its cut. *Observation source:* `release_coherence_gate`'s own comment records the
  four-day instance, and the six-day one was found by an adversarial review of `v0.4.0..HEAD`.
  *Current reaction or bound:* `release-coherence#release-date-disagrees-with-its-commit` holds the date
  against the commit at `State::Snapshot`, so a stale date **cannot reach a release** — it fails the cut and
  the operator fixes it before retrying. The cost is one failed cut, not a wrong published date.
  *Risk:* bounded to that retry. *Promotion trigger:* a cut failing on this twice, which would mean the
  ritual sentence is not being read. *Version class:* patch. *Authority:* `release-coherence`.
  *Shape, and why the obvious one was declined:* the check could compare the dated section against **HEAD's**
  date during preparation — a cut happens at or after HEAD, so a date earlier than HEAD is already wrong, and
  that would have caught both instances the day they appeared. It is declined because preparation spans days:
  the reaction would turn red on the first commit of every new day and ask for an edit whose value is a
  guess, which is a refusal that is right at one moment and wrong the rest of the time — the same trade
  `AGENTS.md` records for the untracked-file guard it declined for the Definition of Done. `AGENTS.md` states
  the step instead, at the moment the value stops being a guess.

- **WATCH: A commit reaching a `release/*` branch without a pull request meets no CI.** *Class:* WATCH.
  *Observed pressure:* `.github/workflows/ci.yml` triggers on `push` to `main` and on `pull_request`, so a
  commit pushed straight onto a release branch runs no job at all — the branch that every development change
  squashes into, and the one a release snapshot is cut from. *Observation source:* measured at `4c3b512` by
  comparing every commit in `v0.4.0..HEAD` against the squash commit of every merged pull request
  (`gh pr list --state merged --limit 800 --json mergeCommit`): **428 of 428 commits resolve to a pull
  request**, so the pressure is structural rather than observed. The fifty that name no `release/0.5.0` pull
  request are `release/0.4.1`'s, carried forward. *Current reaction or bound:* `require_changed_files` in
  `scripts/merge-pr.sh` closes the adjacent shape this was first mistaken for — a pull request whose diff is
  empty because the content was committed onto the release branch itself — and `require_ci_green` refuses a
  merge whose rollup is not green, so every path *through the wrapper* is covered. Nothing covers a push that
  never opens a pull request. *Risk:* a release snapshot cut from content no job ever built. *Promotion
  trigger:* the first commit on a release branch that resolves to no pull request. *Version class:* patch; no
  crate is touched. *Authority:* `repository-checks`. *Shape:* `branches: [main, 'release/**']` on the `push`
  trigger — one line, whose cost is a full eight-job run on every push to a release branch, including the
  worktree-building `pin_bites`. Filed rather than done because this repository's own rule is evidence before
  promotion, and the evidence measured zero.

- **WATCH: The declared MSRV is observable only in CI.** *Class:* WATCH. *Observed pressure:* `rust-version =
  "1.85"` lives in `Cargo.toml` and nothing pins the local toolchain, so the Definition of Done compiles on
  whatever the contributor has. Measured: a single `if let … && …` the default toolchain accepts and 1.85
  refuses was green locally through **nineteen consecutive merges** and red in CI's `msrv` job the whole time.
  *Observation source:* that window, recorded in `scripts/merge-pr.sh`'s own `require_ci_green` header.
  *Current reaction or bound:* `require_ci_green` refuses a merge whose rollup is not green, so the specific
  cost that window paid — merging past a red MSRV job — cannot recur through the wrapper. What remains is
  latency: the contributor learns from CI rather than from the local list. *Risk:* a round trip per MSRV
  regression. *Promotion trigger:* a second failure only the `msrv` job catches. *Version class:* patch.
  *Authority:* `repository-checks`. *Shape:* **not** a `rust-toolchain` file — pinning the workspace to 1.85
  would take `--all-features` clippy off the current toolchain, trading a live check for convenience. What
  would close it is an MSRV step in the Definition of Done, which installs a toolchain and rebuilds, so it
  belongs beside `pin_bites` as an env-gated line rather than in the ordinary list.

  **The trigger has fired, and two of this entry's own premises were wrong.** The recorded trigger was *a
  second failure only the `msrv` job catches*, and it happened: a `let` chain in an `if` condition — stable
  well past `rust-version` — compiled on the default toolchain, passed the whole local Definition of Done, and
  failed CI's MSRV job with `E0658`. That is not a shape resembling the nineteen-merge window this entry
  records; it is the **same construct**, `if … && let …`, written again in the repository that wrote the
  paragraph about it.

  What the firing corrected. *Installs a toolchain* is not a cost here: `1.85` was already present on the
  machine that hit this, so the step is a rebuild and nothing more, and it ran clean in minutes when finally
  asked — `cargo +1.85 build --workspace` and `cargo +1.85 test --workspace --all-features`. And the
  mitigation this entry credits held exactly as written: `require_ci_green` refuses a red rollup, so the
  defect cost one CI round rather than reaching a release branch. Filed rather than promoted for the reason
  the sibling wrapper entry gives — the Definition of Done is not edited at a release cut. First work of the
  window after, with the cost now measured rather than assumed.

- **WATCH: A constant's literal copies outside its reach are unheld.** *Class:* WATCH. *Observed pressure:*
  `shengmo::workspace::MARKER` owns `TIANHENG_WORKSPACE_TESTS`, and seven sites in `tianheng`, `louke` and
  `xuanji` spell it as a literal because those crates cannot depend on `shengmo` without closing a cycle.
  `one_spelling`'s own module documentation puts them outside its subject, and that is right about the
  repair — they are not sites anyone declined to converge. It does not follow that they are unobservable:
  **`kanhe` sees both the constant and their text**, so a comparison is available where a convergence is not.
  *Observation source:* measured at `4c3b512` — one `pub const`, seven literals, nothing comparing them.
  *Current reaction or bound:* none; `one_spelling` states the exclusion and gives its reason.
  *Risk:* a mistyped literal makes its guard's skip condition permanently true, so the test silently stops
  running — a false negative, which is the one bug this project's Core Contract forbids, in the direction
  nothing is watching. *Promotion trigger:* a second constant acquiring out-of-reach literal copies, or one
  instance of the mistype. *Version class:* patch; test scaffolding only. *Authority:* `repository-checks`.
  *Shape:* a direction in `kanhe` asserting every literal spelling of the token equals the constant, with a
  non-empty assertion over the set so a rename cannot empty it into a vacuous pass. Filed rather than built
  because it widens a check whose stated corpus was chosen deliberately, and changing that is a decision
  rather than a repair.

- **WATCH: A backticked identifier in a live document is resolved by nothing.** *Class:* WATCH. *Observed
  pressure:* `reference_integrity` resolves paths and `bound_register` resolves pinning-test names; a bare
  identifier cited in prose is resolved by no reaction. Two survived a full pre-release review and four review
  rounds in live governance text: `BACKLOG.md`'s Shape clause — the actionable half of a READY-PATCH entry —
  instructed an implementer to mirror `the_projection_discloses_every_declared_bound`, a test retired with
  `gate-shape-contract` in this same window; and a live WATCH entry named the self-law generator
  `tianheng_constitution()`, which has never existed, two files from the projection header that spells it
  correctly. Both are repaired. *Observation source:* measured at `965f804` on 2026-08-19 over
  `AGENTS.md`, `PROJECT.md`, `BACKLOG.md`, `COOKBOOK.md`, `README.md` and `docs/`, extracting every backticked
  `snake_case` token and resolving it against every tracked path and every tracked file outside that set:
  **369 tokens, six unresolved, and all six legitimate** — four are explicitly past-tense (*since removed*,
  *is replaced by*, *was one*), one is in `docs/history/`, and one sits in an entry whose preceding line says
  it is kept for its reproduction record and names the surviving test. *Current reaction or bound:* none.
  *Risk:* a dead pointer in the actionable half of an entry sends an implementer to a name that is not there —
  and the class it belongs to is one this repository declares: *an entry that still describes the retired
  mechanism in the present tense is exactly as stale as one naming a retired term*. *Promotion trigger:* a
  third live instance, or a convention that makes the exemption decidable. *Version class:* patch; governance
  documents only. *Authority:* `repository-checks`. *Shape:* filed rather than built, and the measurement is
  why. The token and the question are both decidable — *does this name exist in tracked code* — but every one
  of the six exemptions is a name cited **as history**, and recognizing that is prose judgement, which this
  repository has designed, measured and rejected three times. What would make it buildable is exemption by
  **declaration** — the idiom `one_spelling` already uses — requiring a retired name cited in a live document
  to carry a marker. That is a change to how entries are written, not a detector, which is why it is filed
  with its measurement rather than half-built. The probe also needs the `name()` form, which the run above
  did not match. A second constraint, measured over the remaining sixty live documents by a later review:
  **a negative requirement cites the name it forbids** — `adopter-surface`'s *the obsolete public `FindingKey`
  SHALL be removed* and *SHALL NOT promise a `Dimension`/`ObservedFact` plugin trait* — so a resolver keyed on
  the bare name refuses exactly the requirements being honoured. Recognize by position and shape, never by the
  bare token, which is `projection-register`'s own recorded lesson arriving a third time.

- **WATCH: The reason-perimeter rule is prose applied by hand, and four rounds of applying it produced eight
  corrections.** *Class:* WATCH. *Observed pressure:* the falsifier in `constitution()`'s header — *delete
  what the clause asserts; if the boundary stays green while the clause turns false, the clause is outside* —
  was applied across four review rounds, moving two reasons (use clauses), then three (phrases about edges
  pointing **at** this crate), then three more (phrases about what **another** crate does). Eight corrections;
  no allowlist ever moved. By `AGENTS.md`'s own *a repair loop is a diagnosis, not a schedule*, that is a loop
  dominated by the third class — the code is right and a statement about it is wrong — whose stated remedy is
  to change the shape rather than add a round. The header itself now records *each pass missed a different
  shape*, and the third pass explicitly withdrew an exemption the second had written. *Observation source:*
  the one decidable subset a review proposed — *a reason naming a family crate that is neither its own target
  nor in its own allowlist* — was implemented against the current projection before being adopted, and
  measured at `1fa86d1` on 2026-08-19, over the thirteen boundaries the projection then rendered, by extracting
  each section's heading, reason and allowlist and reporting every family crate named in a reason that is
  neither its target nor in its allowlist: **eight boundaries fired, every one a false positive, none a true
  one.** The figures are anchored to that run rather than kept current — the prototype was not retained, and a
  census with no producer is what this entry is about. Every hit is a legitimate
  prohibition entailed by the allowlist (繩墨's and 勘合's *no edge to 圭表, 渾儀, 漏刻 or 璇璣 can exist*,
  圭表's *must not depend on the 天衡 shell*) or a module boundary naming the crate it must resolve through.
  And it reaches only one of the three shapes the third pass moved: 天衡's *remains the outward composition
  layer* names no crate at all, and 渾儀's *quarantined* names `syn`, which **is** in its allowlist. *Current
  reaction or bound:* none. `self_law_amendment` makes a reason **change** visible — it carries the reason in
  the identity — but says nothing about whether a reason is inside its rule's perimeter. *Risk:* the
  projection teaches every agent that loads `AGENTS.self-law.md`, so a reason asserting structure the law does
  not react to is a false statement of the law, at its most widely read surface. Bounded by four passes having
  been run. *Promotion trigger:* a fifth pass finding a ninth clause, or a decidable subset whose first run
  over the projection produces a true positive. *Version class:* patch; both crates ship in no package.
  *Authority:* `governance-dogfood`. *Shape:* what would end the class is constructing the entailed half of a
  reason rather than writing it, leaving only a genuinely additional clause by hand — a design change, not a
  detector, which is why this is filed rather than half-built.

- **WATCH: A `cfg_attr` whose applied attributes this reader cannot parse drops its `#[path]` candidate
  silently.** *Class:* WATCH. *Observed pressure:* a review of the 0.5.0 window read
  `hunyi::syn_util::cfg_attr_path_values` and `meta_path_value`, both of which take `.ok()` on
  `parse_args_with(cfg_attr_metas)`. The direction is unsafe: the values are module `#[path]` remaps and the
  reader's own doc says every one is a candidate a cfg-blind walker must union, so a dropped candidate means
  some platform's source file is never scanned and any violation in it is a silent false negative. The sibling
  reader twelve lines away, `sole_bare_cfg_predicate`, declares the same collapse in its own doc — its `None`
  has three named causes including a parse failure — and its `None` is the conservative direction. *Observation
  source:* two independent attempts to construct a reaching input, one by the reviewer and one here, both
  failed. Measured against real rustc: bracket-delimited meta lists are rejected by rustc (`wrong meta list
  delimiters`), an unknown attribute is rejected before delimiters are reached, and every applied form rustc
  *does* accept — `unsafe(no_mangle)` in both editions, `doc = include_str!(…)`, a leading-`::` path, a
  trailing comma, a nested `cfg_attr` — parses as a `syn::Meta` and yields its candidate. Measured with the
  same probe: two `path` values in one `cfg_attr` take the **first**, and rustc warns `unused attribute` on the
  second, so this reader's `find_map` agrees with rustc rather than diverging from it. *Current reaction or
  bound:* none; `observation-bounds.md` has zero `cfg_attr` entries, and `hunyi/src/bounds.rs`'s header
  describes the family without declaring this member. *Risk:* a silent false negative in a published crate,
  the one direction this family forbids — bounded only by nobody having found an input. *Promotion trigger:*
  an input rustc accepts and `cfg_attr_metas` rejects. That input is also what the bound needs: a declared
  observation bound is pinned by a direction over its own WHEN, and an unconstructible WHEN cannot pin one,
  which is why this is filed rather than declared. *Version class:* patch; `hunyi` ships. *Authority:*
  `semantic-*` capabilities, which own the resolver's reach.

- **WATCH: The one-spelling corpus reader matches a dependency by name, not by resolved identity.** *Class:*
  WATCH. *Observed pressure:* `one_spelling.rs`'s `members_reaching` builds its edge set by comparing a
  declared dependency's name against each workspace member's **directory basename**, with no path, workspace,
  or source check. A member declaring a registry dependency that happens to be named after another member
  would gain an edge to the local crate it does not depend on. *Observation source:* measured against this
  workspace — all eight members' directory basenames equal their package names, so no edge is silently *lost*,
  which is the direction that would matter; and the eight declare no registry dependency at all, so the
  spurious-edge input does not exist here and the self-law's own allowlists (`serde_json only`, plus syn
  quarantined in one crate) refuse it before this reader would see it. *Current reaction or bound:* the check's
  two-way `assert_eq!(declared, reaching)`, which a spurious edge breaks **loudly** — the corpus can only grow
  under this defect, never shrink, so the failure direction is a false positive. *Risk:* a gate refusing a
  crate that cannot in fact reach the constant. Low, and loud. *Promotion trigger:* a workspace member
  declaring a non-path dependency whose name matches another member's directory basename, or a member whose
  directory basename stops matching its package name. *Version class:* patch; the reader is repository
  machinery, shipping in no crate. *Authority:* `repository-checks`.

- **WATCH: A command a document hands a reader is only checked in one shape.** *Class:* WATCH. *Observed pressure:*
  `5be5678` closed the class *a command a document hands a reader names a target that exists* by
  resolving `-p <package> --test <target>` pairs against `cargo metadata`, and a fifth instance survived it in
  a **second shape**: `BLESS=1 bash crates/kanhe/tests/bound_register.rs`, a path handed to `bash` whose target
  is a Rust integration test. Run as written it printed shell errors and **exited 0**, so a reader following it
  got a silent no-op. *Observation source:* a review of the window, then a sweep of `bash <path>` across
  tracked Rust and Markdown: five occurrences, of which one was live, one is a real script (`scripts/publish.sh`),
  two are deliberate fixture strings in failure matrices, and one is a fictional example path in a doc comment.
  *Current reaction or bound:* none for this shape; the pair shape is held by
  `every_command_a_document_hands_a_reader_names_a_target_that_exists`. *Risk:* a reader meets a command that
  does nothing and reports success — worse than one that fails, because the failure is silent. Bounded by the
  sweep: one instance in the tree, now repaired. *Promotion trigger:* a second live instance of this shape, or
  a third shape of the same class. *Version class:* patch; repository-internal, shipping in no crate.
  *Authority:* `repository-checks`, which owns the pair-shape requirement. *Shape:* the corpus differs from the
  pair check's — this shape appears in **Rust doc comments** as well as Markdown, and the rule is *a `bash`
  target must be a tracked file with an executable mode*, which git records. Two corpora and two rules under one
  requirement, which is why it is filed rather than folded into the existing check.

- **WATCH: A private item's doc comment can be stolen by an item inserted above it, and nothing reacts.** *Class:*
  WATCH. *Observed pressure:* two in-window instances of one class, where a function was inserted between
  another's doc run and the function itself, so Rust attached the whole run to the newcomer and left the
  original undocumented. `604a4e1` did it to `bare_references` (public); `52bf5db` did it to
  `adopter_cited_machinery` (private), where the merged run reads as one doc opening with a paragraph about a
  different function, and it survived nine days and a crate rename. *Observation source:* a review of the
  window, then `#![deny(missing_docs)]` added to `kanhe` and `shengmo` — the last two crates without it —
  which produced 45 and 2 undocumented public items respectively, all now documented. *Current reaction or
  bound:* `deny(missing_docs)`, in every crate, catches the **public** half: any item inserted between a doc
  and its item leaves the original with zero docs, whichever of the two carries a doc of its own — measured by
  reproducing `604a4e1`'s exact shape, which the lint refuses naming the victim. It does not reach private
  items. *Risk:* a doc describes the wrong function while reading as though it describes the right one, which
  is worse than an absent doc; a reader is actively misled. *Version class:* patch; both crates ship in no
  package. *Authority:* none — this is a lint policy, not a Tianheng boundary or a repository check.

  **The trigger fired, and the decision was re-taken on a measurement rather than on the count.** A third
  instance arrived on 2026-08-18 — `imports_and_rest` in `crates/kanhe/tests/refusal_register.rs`, private,
  in a test binary, after this entry — satisfying both halves of the trigger this entry had written down:
  *a third instance, or any instance on a private item after this entry*. The reason recorded here for not
  adopting the lint was bound to the count (*a class with two instances*), so the count moving obliged a
  re-decision rather than a restatement.

  Re-decided: **still not adopted**, and now for a measured reason instead of a claimed one.
  `clippy::missing_docs_in_private_items` over `kanhe` alone reports **785** undocumented private items
  across its library and test binaries — measured the day this was re-taken. The lint is the only mechanism
  that closes the private half, and it closes it by demanding a doc on every private item; that cost is not
  proportionate to a class whose instances are individually cheap to repair and whose damage is a misleading
  doc rather than a wrong verdict. *Promotion trigger,* restated so it does not rest on a count nothing
  produces: an instance where the stolen doc changed what a reader **did**, rather than what they would have
  read — a repair made against the wrong function, or a bound declared from a doc describing something else.

  **What this entry could not do for itself.** Its trigger was written, both halves fired, and the entry sat
  unchanged for a full round of review — because nothing evaluates a promotion trigger. A `WATCH` is carried
  by whoever next reads it, which is the same shape as a requirement whose clause has no reaction, and it is
  the way this class of entry fails. It was caught by a review arriving at it sideways from an unrelated
  finding.

- **WATCH: A reader's corpus can be narrower than the requirement it serves, and this repository's own dimensions
  cannot see the shape.** *Class:* WATCH. *Observed pressure:* the dominant class of the 0.5.0 window. Live
  instances repaired here: `marks_a_bound` gated pinning-citation resolution so 5 of 75 citations were never
  validated; `machinery_names` enumerated 0 of 8 workspace members against its own subject; the root
  manifest's `exclude` named one fixture root of two while `.gitignore` named both. *Observation source:* a
  sweep of the window's findings, then the classification in `AGENTS.md`'s *A reader reads its whole subject*,
  which separates four shapes and closes three of them by construction. **This entry counts only the fourth**
  — corpus narrower than the claim. Lossy selection, lossy acceptance and lossy accumulation are closed where
  they occur (`kanhe::selection`, `capability_subjects::Declared`, and widening the binding), so counting them
  here would fire this trigger on instances that are already shut. *Current reaction or bound:* none, and it
  is not available. `inline-symbol-path-confinement` declares that a **receiver-method read is not observed**
  — no type inference on the receiver, pinned by `inline_receiver_method_read_is_a_bound` — and
  `text.split_once(…)`, `iter.next()` and `vec.first()` are receiver-method calls, so the shape sits outside
  the observation surface this repository ships. That is an existing declared bound with an owner, not a
  rationale invented for this entry. *Risk:* a check reports clean over a subject it never read, which is the
  one direction the Core Contract forbids, and it is invisible until a second instance exists. *Promotion
  trigger:* a third live instance of the fourth shape after this entry. *Version class:* patch;
  repository-internal. *Authority:* `repository-checks`. *Shape:* only a set comparison in both directions
  catches it, and there is nothing to compare against until someone states what the subject is — which is why
  the capability-subject declarations exist and why widening them is the likely form of any repair.
  Until there is a reaction, the interim instrument is the class-directed sweep stated in `AGENTS.md` — run
  it at each pre-release review rather than trusting that the next instance will be noticed. It is what found
  the release-coherence pair after five linear rounds had read past them.

  One residue belongs here rather than to the shapes that closed. `kanhe::selection` binds only the call sites
  that use it, and nothing enumerates the readers that should. (A second residue recorded alongside this one —
  `census::figures_in` reading only the first match on a line — was closed the following day and is no longer
  live; this entry went uncorrected until a later adversarial-review pass noticed the drift.)

- **WATCH: `Bind a claim to its measurement` is a governing rule with no reaction.** *Class:* WATCH. *Observed
  pressure:* nine review rounds in the 0.5.0 window, whose largest class by far was *the corpus was wrong or
  its narrowing was undeclared* — nine of roughly twenty findings. The rule was written from that sweep and
  names three bindings: derive it, declare it and hold it both ways, or compare it after the fact where the
  carrier is text. *Observation source:* the sweep itself, plus the refutation inside it — `WRAPPERS` was
  proposed as a derivation candidate and `self_governance`'s own comment defeated it, because removing
  `guibiao` from that literal left a `guibiao` allowlist naming `hunyi` green. *Current reaction or bound:*
  the **text branch** is enforced by `crates/kanhe/tests/census.rs`; the rule above it is enforced by nothing.
  *Risk:* bounded and of a particular kind — the rule tells an author which instrument to reach for, so its
  failure mode is a check built with the weak binding where the strong one was available, which is a defect a
  reviewer finds rather than one that ships. Every instance found so far was found that way. *Promotion
  trigger:* an instance where the wrong binding was chosen **after** this rule was written — the rule being
  the control, so the nine that produced it cannot stand as evidence for themselves. *Version class:* patch;
  repository-internal, shipping in no crate. *Authority:* `repository-checks`, which owns the shape of a
  check. *Shape:* not a prose detector — answering it needs to know that a value is a claim *about* something,
  which is intent rather than shape, and the measured-and-rejected class here is exactly judgements over text.
  If it is ever reacted, the reaction is more likely to be a **type** that makes the weak binding harder to
  reach than a scan that recognises it.

- **WATCH: a figure a repository check already produces can be repeated in prose that declares no census, and
  the join between producers and repetitions has never been taken.** *Class:* WATCH. *Observed pressure:* the
  0.5.0 window, where adversarial review found figures wrong across a single change in every kind of place one
  can live — `crates/kanhe/tests/census.rs` records how many — and the review after it, which found a figure
  warranting a design decision written unanchored in live source. Both were found by a reader asking; nothing observed either.
  *Observation source:* the declared census set is enumerable, and so is the set of checks that produce a
  figure — several print one in a refusal or on a clean run. Neither side has been held against the other, and
  the reachable subset is exactly the figures whose producer exists and never declared them. *Current reaction
  or bound:* the declared censuses are held by `crates/kanhe/tests/census.rs`; everything outside them is the
  declared bound
  `repository-checks/a-count-written-in-a-sentence-no-census-declares-a-stated-bound`, under-reacting and
  engine-owned, with `AGENTS.md`'s three-state disposal as the reviewer's half. *Risk:* bounded, and the
  bound is what makes it a WATCH rather than a patch — a repeated figure misleads a reader and is repaired in
  a clause, and the subset a producer could close is the only part closable by construction at all. *Promotion
  trigger:* a count finding in a window after the disposal rule was written, the rule being the control —
  the same form the `Bind a claim to its measurement` entry states for its own, and for the same reason: the
  instances that produced a rule cannot stand as evidence for it. *Version class:* patch; repository-internal,
  shipping in no crate. *Authority:* `repository-checks`, which owns the shape of a check. *Shape:* enumerate
  the **producers**, never the prose. A sweep over the digits in tracked documents fails the audit cycle's own
  second test — *is this figure a census* cannot be generated and staleness-checked, only hand-maintained, so
  such a sweep would produce the drifting artifact it exists to police, one level up. The checks are code and
  therefore enumerable. Its size is deliberately not written here.

- **WATCH: A third hand-written scanner of the `#### Scenario:`/`- **PINNED-BY**` grammar still disagrees with the
  other two on two edge cases.** *Class:* WATCH. *Observed pressure:* `bound_register_parse::bounds_in` and
  `bound_register_parse::citations_in` (the latter extracted from `pinning_citations` during an adversarial
  review the 0.5.0 window, so `pin_bites::cited_bounds` and `pinning_citations` share one recognizer instead of
  two) now agree with each other exactly. `crates/kanhe/tests/observation_bound_model.rs`'s own
  `spec_bounds`/`spec_defence` were not touched by that extraction and remain a **third**, independent
  implementation of the identical grammar, found by the same review to have already drifted from the other
  two in two ways: it requires a literal trailing space after `"#### Scenario:"` (`bounds_in`/`citations_in`
  trim the heading instead, so `"#### Scenario:Foo"` with no space is a bound to them and not to this
  scanner), and it checks an **untrimmed** line for a closing `"###"`/`"##"` heading (`bounds_in`/
  `citations_in` check the trimmed line, so an indented closing heading ends a bound's scope for them and not
  for this scanner). *Observation source:* the adversarial review's cross-file tracer, verified by grepping
  every tracked spec for both shapes — none currently has a `"Scenario:"` with no following space or an
  indented `"###"`/`"##"` heading, so both disagreements are latent. *Current reaction or bound:* none;
  `observation_bound_model.rs`'s bijection tests only compare its own scanner's output against
  `observation_bounds()`, which cannot see a disagreement with `bounds_in`/`citations_in` since nothing
  compares the three against each other. *Risk:* a future spec edit hitting either latent shape would have
  `observation_bound_model.rs` and the register disagree on a bound's existence or a citation's defended id,
  each internally consistent and both wrong relative to the other. *Promotion trigger:* either latent shape
  appearing in a tracked spec, or a fourth independent scanner of this grammar appearing anywhere in the
  crate. *Version class:* patch; repository-internal, shipping in no crate. *Authority:* none declared yet —
  the three readers currently agree on every live spec, so nothing has been violated to anchor one. *Shape:*
  closing this fully needs one shared low-level walker (`"#### Scenario:"`-untrimmed to open,
  `"#### "`/`"### "`/`"## "`-trimmed to close, matching `bounds_in`'s own rules) that all three call sites use,
  not a third point patch; filed rather than done here because it touches `observation_bound_model.rs`'s core
  logic, a file the 0.5.0 window's fixes did not otherwise touch, and reworking it deserves its own scoped review
  rather than folding into an adversarial-review pass over a different fix.

- **WATCH: `negates_bound_in_prose`'s one-interposed-word budget is measured from `a`/`an`, independently of
  `states_a_bound_in_prose`'s own budget measured from `stated`/`documented`, so a sentence stacking both
  qualifiers is read as a declaration rather than the denial it is.** *Class:* WATCH. *Observed pressure:* an
  adversarial review of the 0.5.0 window's own fix (which corrected a live case-sensitivity gap in both functions,
  see the `CHANGELOG.md` entry) also asked whether the two functions' independent one-word tolerances could
  ever disagree, and found they can: `"this is not a documented residual bound"` has `states_a_bound_in_prose`
  return `true` (one interposed word, `residual`, between `documented` and `bound`) and
  `negates_bound_in_prose` return `false` (its own one-word budget, between `a` and `bound`, is already spent
  on `documented`, leaving no room for `residual` before `bound`) — so `undeclared_prose_offences`'s
  `!states(...) || negates(...)` guard evaluates `false`, and a genuinely negated sentence is reported as an
  undeclared bound. *Observation source:* verified directly (`states_a_bound_in_prose` returns `true`,
  `negates_bound_in_prose` returns `false` for the sentence above), then grepped every tracked spec for
  `(not|never) a (stated|documented) [a-z-]+ bounds?` — zero matches, so this is latent. *Current reaction or
  bound:* none. *Risk:* a future spec sentence combining a negation with a states-side qualifier
  (`"...is not a documented residual bound"`) would be misreported as an undeclared bound rather than
  correctly exempted as a denial. *Promotion trigger:* a live instance of the stacked shape in a tracked spec.
  *Version class:* patch; repository-internal, shipping in no crate. *Authority:*
  `observation-bound-register`'s "A bound stated in prose but not declared as a scenario SHALL fail" — the
  requirement this pair jointly enforces. *Shape:* the two budgets need to share one accounting (e.g. count
  every interposed word from the negator through to `bound`/`bounds`, capped at two total: one for the
  negator's own qualifier, one for the states-side qualifier) rather than each independently assuming it owns
  the only interposed word in the sentence — a design question, not a one-line patch, which is why it is
  filed rather than widened here under time pressure.

- **WATCH: `requirement_heading_is_bounds_named` matches `bound`/`bounds` as a bare substring with no check on the
  character *before* the match, so an unrelated word like `Outbound`/`Rebound`/`Unbounded` in a requirement
  heading would falsely exempt that requirement from the undeclared-prose check.** *Class:* WATCH. *Observed
  pressure:* the same adversarial review, independently reported by two finders. The function checks only that
  the character *after* `bound(s)` is non-alphabetic (so `boundary`/`boundaries` are correctly excluded), with
  no symmetric check on the character before — unlike this file's own `contains_words`, used by
  [`marks_a_bound`], which checks both sides. *Observation source:* traced by hand
  (`requirement_heading_is_bounds_named("Outbound Requests")` returns `true`: `"bound"` matches inside
  `"outbound"`, and the character following it — a space, from `" requests"` — reads as "no letter follows"),
  then grepped every tracked `### Requirement:` heading for a `bound`-containing word that is not
  `bound(s)`/`boundary/boundaries` itself — none found, so latent. *Current reaction or bound:* none.
  *Risk:* a requirement heading using an ordinary English word containing `bound` as a substring
  (`outbound`, `rebound`, `abound`) would be wrongly classified as bounds-named, exempting real
  bound-stating prose beneath it from `undeclared_prose_offences` and instead charging that requirement with
  "declares no bound scenario of its own" — the wrong failure mode for a heading that was never about bounds
  at all. *Promotion trigger:* a live instance of such a heading in a tracked spec. *Version class:* patch;
  repository-internal, shipping in no crate. *Authority:* `observation-bound-register`, the same requirement
  the sibling entry above cites. *Shape:* add the same before-the-match boundary check `contains_words`
  already has, most simply by having this function call `contains_words(heading, "bound")` /
  `contains_words(heading, "bounds")` (or a variant tolerant of the `boundary` exemption) instead of its own
  one-sided scan — a smaller change than the sibling entry above, filed alongside it rather than folded into
  the 0.5.0 window's fix because both were found by the same pass and neither has a live instance forcing it.

- **WATCH: `AGENTS.md`'s OpenSpec lifecycle section describes a process with no instances.** *Class:* WATCH.
  *Observed pressure:* found while deciding the entry above — its four phases, commit-type conventions and
  archive-pruning guardrail are written in the present indicative ("A capability change **moves through**
  OpenSpec") about something that has never happened here. *Observation source:* zero
  `docs(openspec): propose`/`sync` commits in the full history; `openspec/changes/**` untouched since
  `2f903fb`; 151 direct spec edits in the last window. The path evidence is independent of commit subjects,
  so the 2026-07-17 history rewrite does not weaken it. *Current reaction or bound:* none — this is prose
  stating a fact about the tree that the tree contradicts, the same class as the three orphans above and
  larger. *Risk:* an agent reading `AGENTS.md` first, as that file instructs, plans work through a lifecycle
  nobody runs — which this session did, three times, before measuring. *Not simply deleted:* half of it is
  live and load-bearing. The sync-evidence rule (a materially changed scenario carries its observation
  evidence in the same change) **is** followed, and `openspec/changes/archive/.gitkeep` is tracked exactly as
  described. So the disposition is audit-then-dissolve/convert/keep per claim, not removal. *Promotion
  trigger:* a decision on whether the lifecycle is being restored or is being acknowledged as abandoned —
  a human call about intent, not a defect to repair. **Not fired** (evaluated 2026-08-18). *Version class:* not release-affecting.
  *Authority:* `AGENTS.md` itself.

- **WATCH: the family-coverage reaction credits ownership by type NAME, so a profile is invisible to it.**
  *Class:* WATCH. *Observed pressure:* a profile constructs its boundaries internally —
  `Constitution::sans_io_pure` builds a `ModuleBoundary` and an `AsyncExposureBoundary` from one
  `SansIoPure` — so a file declaring a family that way never spells the type, and
  `crates/shengmo/tests/family_coverage.rs` does not credit it. *Observation source:* found while checking a
  claim this review had already written down and been wrong about. `examples/sans-io-pure` declares async
  exposure in its `src/governance.rs` through the profile and is credited only because its
  `tests/reaction.rs` separately names `AsyncExposureBoundary` outright; reading `sans_io.rs` rather than
  grepping for the type is what showed it. **Measured, not argued:** renaming that one type in
  `tests/reaction.rs` makes the reaction report `["AsyncExposureBoundary"]` unowned while
  `src/governance.rs` still declares it through the profile. *Current reaction or bound:* none — the residual is stated in the
  reaction's own module documentation rather than left for a reader to rediscover. *Risk:* a **false
  refusal**, which is the direction that costs most here: delete one line of that test and a family reads as
  unowned while the example still teaches it, sending someone to write coverage that already exists. Bounded
  today — every family has an owner that spells its type. *Why not simply widen the reader:* the honest
  closure is not a longer name list but asking what a profile *expands to*, which means evaluating
  constructor bodies rather than reading declarations — a different instrument from the one this reaction
  is. *Promotion trigger:* a family whose only adopter-shaped owner declares it through a profile, or a
  second profile reaching a family no bare declaration names. **Not fired** (evaluated 2026-08-18). *Version class:* patch;
  repository-internal, shipping in no crate. *Authority:* `governance-dogfood`.

- **WATCH: the capability-subject filing join observes nothing under the declared OpenSpec mode.** *Class:*
  WATCH. *Observed pressure:* `capability_subjects.rs`'s
  `a_change_names_every_capability_whose_subject_it_touches` enumerates `openspec/changes/*/proposal.md` and
  returns early when that is empty. `PROJECT.md` records this project using OpenSpec's `specs` half and not
  its `changes` half, so the corpus is empty by declaration and the early return is always taken.
  *Observation source:* `git ls-files openspec/changes` returns one path, `archive/.gitkeep`; the join was
  built in the same window the mode was declared, four commits apart, and neither noticed the other. Three
  sibling `openspec/changes/` carve-outs are in the same position — `law_restatement.rs`'s projection filter
  and two in `reference_integrity.rs` — though one of those is exercised by a fixture that plants a
  synthetic change path, so it is a branch with no live subject rather than dead code. *Current reaction or
  bound:* none for the class it guarded. **The filing class is defended by review alone**, and it is live
  rather than hypothetical: `scripts/publish.sh` has two claimants, which is the shape the join was built
  from. *Risk:* a requirement filed under the wrong capability goes unnoticed until someone reads both
  specs. Bounded — the mistake is visible in the diff of any PR that makes it. *Why not re-point it:* the
  join compares a proposal's **declared** capability set against the subjects a diff touches, and where no
  proposal is present there is no independent declaration to compare against — reading the set from the
  touched spec paths is near-tautological, since touching a spec is naming its capability. *Promotion
  trigger:* a requirement found filed under the wrong capability. **Not fired** (evaluated 2026-08-18).
  *Version class:* patch; repository-internal, shipping in no crate. *Authority:* `capability-subjects`, and
  `PROJECT.md`'s adoption-mode decision.

  *Note, 2026-08-21: the trigger's second half — `the changes half being adopted, which makes the join start
  working with no edit` — is retired, because it had already happened. A change directory is committed on the
  development branch, so the join runs there and returns early only on the release spine. What stays live is
  the first half: the filing class is defended by review wherever the join returns early, which is every CI
  run.*

- **WATCH: nine structural findings the merged-review campaign deliberately did not take.** *Observed
  pressure:* a contributed review's Gate 4 found twelve functions long or deeply nested enough to hide more
  than one responsibility. Two were taken (`offences_in`'s four resolution rules, `judge`'s six phases) and
  ten were not. *Observation source:* that review, measured against `HEAD` — every length it reported was
  reproduced exactly. The residual sites are `publish_source_gate`'s `judge` and `verify_tag_signature`,
  `release_coherence_gate`'s `machinery_names`, `bound_register_parse`'s `bounds_in` and
  `undeclared_prose_offences`, `census`'s `number_at` and `sweep`, `region`'s `Prose::lines`,
  `reference_integrity`'s link normalisation. *Current
  reaction or bound:* none, and none is wanted — length is not a property this repository reacts to.
  *Risk:* low and legible. These are pure structural changes to gate code that stabilised inside the window
  that found them, and the risk of moving it is not symmetric with the risk of leaving it long. *Promotion
  trigger:* a defect found in one of the named sites whose diagnosis was made harder by its length — not the
  length itself, which is the measurement that already exists and is not the evidence. **Not fired** (evaluated 2026-08-18).
  *One of the twelve left this list on its own evidence, not on the trigger:*
  `examples/observer-participant`'s `observe` was split, because a second review pointed out that
  `COOKBOOK.md` sends adopters to it as the runnable version of its recipe — so a design whose layers cannot
  be tested apart is being *taught*, which is an adopter-facing consequence rather than the internal
  maintainability cost the other nine carry. The trigger stays as written for those nine; what moved this one
  was a different kind of argument, not a lower bar. *Version class:* patch; repository-internal.
  *Authority:* the campaign recorded in `CHANGELOG.md`'s `### Self-governance` section.

- **WATCH: the reference gate's dated-section exemption is widest while the section is still being
  written.** *Observed pressure:* the exemption keys on a section being *dated*, and dating is the freeze
  act — `chore(release): prepare X.Y.Z` cuts `[Unreleased]` into `## [X.Y.Z] - DATE`, after which
  `release-coherence` **requires** `[Unreleased]` to be empty in the release-ready state. So from the prepare
  commit until the release, every new CHANGELOG entry is written into a dated section
  `crates/kanhe/tests/reference_integrity.rs` skips, and that section is the largest and newest in the file.
  *Observation source:* the `[0.5.0]` section measured at 2026-08-16 — 2,572 lines carrying 41 distinct
  in-repository references (22 prefixed paths, 19 bare basenames), resolved by hand, **all resolving**. So
  the exposure is real and has cost nothing yet. *Current reaction or bound:* none for that window; the
  reasoning is recorded beside the `in_dated_section` skip rather than left for a reader to rediscover.
  *Risk:* bounded and self-limiting — a stale path written into the current version's section survives only
  until someone reads it, and the same paths are usually named in the code the entry describes, which the
  gate does read. *Why not simply narrow it:* making the current version an exception means this scan asking
  `release-coherence` which version is unreleased, so a reference verdict would begin to depend on the
  release spine and a shallow checkout would move it — trading a bounded blind spot for a verdict that
  varies with checkout depth. *Promotion trigger:* a stale reference found inside the section of a version
  **not yet released** — the 41 above are the control and cannot stand as evidence for themselves. **Not
  fired.** *Version class:* patch; repository-internal, shipping in no crate. *Authority:*
  `openspec/specs/reference-integrity/spec.md`.

- **WATCH: `PROJECT.md` restates facts a generated projection already holds, and states others nothing
  holds.** *Observed pressure:* three consecutive attempts at one paragraph were withdrawn, all failing the
  same way — asserting a location or an absence without sweeping for it. Classifying that file's claims
  against the generated projections splits them in two. The **architectural** ones are already carried:
  which crates exist, what each may depend on, that no dimension names a sibling — all projected from
  `shengmo::law::constitution()` into `AGENTS.self-law.md` and staleness-checked, and since
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
  for themselves. **Not fired** (evaluated 2026-08-18). *Version class:* not release-affecting. *Authority:* `projection-register`,
  which already enumerates the documents a claim could cite, and `self-law-projection`, which owns the one
  carrying the architecture. *Shape, if it fires:* not a detector over prose — that instrument was measured
  three times and rejected. The reachable direction is to make more of what the file asserts **citable**, so
  that restating is the strictly worse-looking option at the moment of writing.

- **WATCH: four limits of the mutual-independence check, each measured; three still declared, one closed and
  retired.** *Observed
  pressure:* closing the membership half of `三儀 ⊥ 三儀` exposed four more, all reproduced by writing them into
  the tree rather than argued about. **Wording, false refusal:** paraphrasing `guibiao`'s clause makes the
  check fire — it refuses a reason that genuinely states the law. **Wording, false negative:** a `because`
  carrying the literal clause while *negating* it passes, and `AGENTS.self-law.md` then teaches the negation to
  every agent that loads it. **Enumeration, the dimension list:** *closed, and its declared bound retired* — the
  literal is held against the workspace's own dimension crates, read from cargo and compared both ways, so the
  omission that left a `guibiao` allowlist naming `hunyi` green is refused. The bound was retired against a run
  of its own WHEN on the post-change tree rather than against the argument that it should be. It is retained
  here as the third of the four because the entry's own point is that these are one check's limits, and a
  closed one is what tells a reader the others are not.
  **Enumeration, the rule variant:** the filter admits only `RestrictDependenciesTo`, so a second
  boundary using `restrict_workspace_dependencies_to` — the more natural rule for this law — is never examined.
  *Observation source:* those four perturbations, run during review of
  `change/mutual-independence-reacts-to-membership`. *Current check or bound:* a check for the third, which
  closed it; none for the other three, whose statement lives in the check's doc comment and
  `self-law-projection` where a reader meets them. *Risk:* the second is
  the serious one — the agent-facing projection can teach the negation of the law it quotes. *Promotion
  trigger:* fired for the second; the others are recorded with it because they are one check's limits and
  closing them separately would re-open the same file four times. *Version class:* patch; a `tests/` check of
  this repository. *Authority:* `self-law-projection`. *Shape:* **pinning** any of them needs the check run over a
  supplied declaration rather than its predicate over a string, which means factoring the assertion loop to take
  a `Constitution` — that is what this entry owns. **Declaring** them needed none of that, and an earlier draft
  of this entry said it did: it read the pin requirement as a declaration requirement and withheld all four,
  which kept a measured false negative out of the register a reader is told to consult before calling a
  behaviour a defect. The three that remain are declared unpinned against this entry; the fourth's
  declaration was retired with the limit, against a run of its own WHEN rather than against the argument that
  the code now looked right.

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

- **WATCH: the self-governance residual is a judgement over an entry's subject.** *Observed pressure:*
  `CHANGELOG.md` is the adopter's document and offered no heading that was not an adopter's vocabulary, so
  twenty entries named that machinery, before the section was collapsed — eleven in `[Unreleased]`
  and nine in the released `[0.4.0]` — spread
  across `### Added`, `### Changed`, `### Fixed` and `### Documentation`. The rule that now refuses them reads an entry's
  **references** — a word equal to a path under `scripts/`, or to a basename `git ls-files scripts/` resolves
  — and an entry describing this repository's own governance while naming no such word stays invisible to it. *Observation source:* two live instances, not a hypothetical: after this
  window's move, the entries *The bound register refuses a restatement* and *The bound register's own citations
  can no longer read as coverage while defending nothing* both sit under adopter headings, both describe the
  bound register's own behaviour, and both name nothing the enumerator resolves. *Current reaction or bound:*
  declared unpinned as
  `release-coherence/an-entry-about-self-governance-that-names-no-machinery-a-stated-bound`; the five limits
  that *do* have a mechanical WHEN are pinned in `tests/release_coherence.rs`. *Risk:* low and
  one-directional — the adopter reads a paragraph about housekeeping, never a wrong claim about what they get.
  *Promotion trigger:* an entry of this shape carrying a claim an adopter could **act on** — a version, a
  migration step, a behaviour change — rather than a description of internals. That is a property of one
  entry and decidable by reading it, unlike a threshold on a population nothing counts. **Not fired** (evaluated 2026-08-18): every
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
  this is WATCH rather than READY — see the trigger. *Promotion trigger:* **Not fired** (evaluated 2026-08-19). The candidate evidence
  points both ways. The harness enumeration was rejected twice on an unmeasured premise, then measured at 107ms
  cold and adopted — a rejection a later reader *did* consult and correctly overturn, which argues that a
  durable, projected register would have entrenched a wrong answer twice. **Where that record now lives is the
  sharper half:** the rejection was written in the check's own comments and those comments did not survive the
  change that overturned them, so the account is here and in `CHANGELOG.md` and nowhere in the code it was about
  — which is the entry's own case for a durable record, made by the disappearance rather than by argument. The trigger is a rejection demonstrably **re-proposed and
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

- **WATCH:**
- **WATCH: a pin may defend a direction its bound does not declare.** *Observed pressure:* one live instance,
  found by review rather than by any reaction. `whether-a-mention-compiles-anything-is-not-observed` declares
  `UnderReacts` — the check counts a comment-only mention as named — and cited a test containing no comment at
  all, asserting instead that a substring is not a mention. That is a *reacting* distinction, so the citation
  ran, bit, and demonstrated the opposite of what the bound predicts. *Observation source:* that comparison,
  run against the declaration, its spec scenario and the cited test's body. *Current reaction or bound:*
  declared unpinned as
  `observation-bound-register/whether-a-citation-demonstrates-the-direction-its-bound-declares-a-stated-bound`;
  `Extent::demonstrates()` reaches the projection label and the contradiction classification, and nothing
  compares it with what the cited test asserts. *Risk:* the register's leading figure counts a bound as defended
  when its defence is about something else — which is the register's own failure mode, one level in. *Promotion
  trigger:* a **second** instance, or a derivation that decides the direction a test demonstrates without
  reading its source as prose. *Why not simply built:* deciding what a test demonstrates from its body is a
  judgement over code of the kind this repository has designed, measured three times and rejected over prose,
  and unlike a citation that never runs or never bites there is no reaction whose gap a fixture could exhibit —
  which is why this is declared rather than pinned. *Version class:* patch; repository-internal, shipping in no
  crate. *Authority:* `observation-bound-register`, and `observation-bound-model`'s sibling bound that a
  rationale contradicting its extent is accepted — the prose beside an extent was already free to disagree with
  it, and so, until this was found, was the test beneath it.
- **WATCH: a dependency declared under a quoted cfg target is not observed.** *Observed pressure:* the
  example-pin reader decides which tables hold dependencies from the heading, and admits
  `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, their `.NAME` detailed forms, and any of
  those under a bare target triple. `[target.'cfg(…)'.dependencies]` is not admitted. *Observation
  source:* the round that read the bare-triple form, which found the wider entry this replaces had taken a
  reason about the quoted grammar and applied it to the whole target corpus. *Risk:* a stale family pin
  under a cfg-guarded target table reaches a release unobserved. Bounded by there being no target table in
  any manifest in this repository, ever. *Next trigger:* an example acquiring one — at which point the
  shape has an instance to be written against rather than a guess. *Authority:* engine.
  *Compatibility:* patch; the check ships in no crate.

- **WATCH: a bare reference to a registered constructor's name cannot be told from a local variable sharing
  its spelling without name resolution.** *Observed pressure:* `refusal_register.rs`'s reader moved from a
  character-by-character scan to `syn`, which closed the *lexical* half of the wider bound this entry used
  to name — a byte char literal, a raw string, a wrapped import, a two-line closure parameter list are all
  read correctly now, by construction rather than by an arm added the day each was found wrong. What a real
  parser's AST cannot supply is name resolution: `let build = violation; …; build(x)` and a reference to
  `violation` after a local binding of the same spelling has shadowed it are syntactically identical to a
  call to the constructor taken by value, and no reading of syntax alone decides which one a given file
  means. *Observation source:* `crates/kanhe/tests/fixtures/refusal_scan/a_constructor_taken_by_name.rs.txt`
  and `a_siteful_constructor_taken_by_name.rs.txt`, the two fixtures this residue is written against.
  *Risk:* a module that actually shadows a refusal constructor's name with an unrelated local would be
  over-counted as constructing one — the safe direction, since it can only report a site that does not exist
  rather than miss one that does. Bounded by the corpus: `kanhe` ships in no package, and no file in this
  repository currently shadows a constructor's name this way. *Next trigger:* a real occurrence of the
  shadowing shape in this repository's own source, which would be the first evidence the residue is more
  than theoretical. *Authority:* engine. *Compatibility:* patch; the checks ship in no crate.

- **WATCH: a gate that is its own test is outside the refusal register.** *Observed pressure:* several
  gates are implemented under `crates/kanhe/tests`, where the judgement and the directions over it share a
  file; their refusals carry no site identity, because *which direction observes this branch* has no answer
  when every direction in the file can see it. *Observation source:* the refusal register, whose corpus is
  `crates/kanhe/src` — measured when deleting the site-less constructors broke those files and not the
  registered ones. *Risk:* a refusal in one of those gates is unexercised and nothing says so, which is the
  gap the register closed everywhere else. Bounded by those files being both judgement and test, so a
  refusal with no direction over it is visible to anyone reading the file it lives in. *Next trigger:*
  moving such a judgement out of its test file, at which point it enters the corpus and is triaged like
  every other. *Authority:* engine. *Compatibility:* patch; the checks ship in no crate.

- **WATCH: a promotion trigger is evaluated by whoever next reads the entry.** *Class:* WATCH. *Observed
  pressure:* an entry in this document recorded a class, its instances, its risk and its own promotion
  trigger — *a third instance, or any instance on a private item after this entry* — and both halves fired
  on 2026-08-18 without anything acting on them. The entry sat unchanged through a full round of review and
  was reached sideways, from an unrelated finding, rather than by anyone asking whether it had triggered.
  *Observation source:* that miss, and then a sweep of this document run on purpose the same day: of the
  triggers phrased as a count or an instance, two already carried a recorded *not fired*, one had fired
  and was re-decided, and the rest were evaluated and had not. A third scanner of the scenario grammar is
  still three — the two further matches are fixture text and prose, read to check. A third governance member
  has not arrived. A second orphaned corpus has not: every tracked fixture corpus is referenced. A second
  ecosystem pinning by digest has not, because the one this repository cannot refresh *is* the entry's
  subject. *Risk:* an entry states the condition under which a decision must be re-taken, and the condition
  arrives silently — which is the same shape as a requirement whose clause has no reaction, one document
  over, and the only way this class of entry fails. Bounded by the entries themselves: each names its
  trigger, so the sweep is possible and cheap; what is missing is anything that runs it. *Next trigger:* a
  second trigger found to have fired unnoticed, at which point the answer is a reaction over this document
  rather than a sweep run on purpose — the same escalation the orphaned-corpus entry declares for itself.
  **That trigger is decidable only because the annotations now carry a moment.** They were written as
  present-tense assertions — a bare *not fired* — so a reader could not tell one evaluated today from one
  written when the entry was filed and unread since, and *unnoticed* had no observable meaning. Every
  annotation matching `**Not fired.**` now reads `**Not fired** (evaluated <date>)`, and a sweep compares
  dates rather than re-reading every entry. **The corpus is named because the first sweep was stated
  universally and was not**: a mechanical replacement matched only the capitalised form, so one annotation
  spelled `**not fired.**` survived a claim that every annotation carried a date. *Authority:* engine. *Compatibility:* patch; this document ships in no crate.

- **WATCH: a tracked declaration nothing reads.** *Observed pressure:*
  `crates/kanhe/tests/fixtures/refusal_scan/` was tracked on 2026-08-10 and referenced by nothing until
  2026-08-18 — a case corpus naming exactly what a refusal-construction reader must handle, three of whose
  cases
  were rediscovered the hard way in the meantime and one of which had not been. *Observation source:* the
  register's own reader, which the corpus was written for. *Risk:* a declaration written and then orphaned
  reads as coverage to anyone who finds it and holds nothing. Bounded by measurement: every tracked fixture
  corpus in this repository is referenced by at least one reader — eight of eight, swept the day this was
  written — so this is one instance rather than a class. *Next trigger:* a second orphan, at which point the
  shape changes and a reaction enumerates them instead of a sweep run on purpose. *Authority:* engine.
  *Compatibility:* patch.

- **WATCH: a refusal reachable only by a broken tool is not observed.** *Observed pressure:* fifteen refusal
  sites are declared unheld in `crates/kanhe/src/refusal_bounds.rs` — every one of them a cannot-judge
  reachable only when a tool this repository invokes fails mid-run. *Observation source:* the refusal
  register, which measured which sites a direction observes by running rather than by reading their
  messages; five textual predicates asked the same question first and answered differently every time.
  *Risk:* one of those fifteen refuses with the wrong sentence, or the wrong exit class, and nothing says
  so. Bounded by all fifteen being cannot-judge: the class reserved for *this could not be read*, so the
  worst case is an unhelpful sentence rather than a defect reaching a release. *Next trigger:* a harness
  that can supply a failing tool without the fixture becoming a test of that harness — a recorded process
  boundary rather than a fake binary on the path. *Authority:* engine. *Compatibility:* patch; the checks
  ship in no crate.

- **WATCH: the window the publish wrapper can only narrow.** *Observed pressure:* the publish wrapper runs
  the source gate, then `cd`s and `exec`s `cargo publish`. Between those the repository can be altered — a
  commit, an amend, a tag moved, the remote's `main` advancing — and the gate's verdict is about the tree as
  it was. *Observation source:* a sweep for limits declared on one wrapper and not on its sibling. The merge
  wrapper declares this class for its own title and pins its other two inputs by construction; the publish
  wrapper, standing in front of the act that cannot be undone, declared nothing. *Risk:* a version is
  uploaded from a commit the gate never judged, permanently, which is the failure this whole capability
  exists to prevent. Bounded by what `cargo publish` re-checks for itself — it refuses a dirty worktree, so
  reaching this needs the tree amended *and* committed — and by the window being two statements wide rather
  than a whole `cargo test`. *Next trigger:* cargo gaining an argument that names the commit a publish must
  package, which is what closes the equivalent window on the merge path; until then narrowing is the only
  available move and it is already taken. *Authority:* engine. *Compatibility:* patch; the wrapper ships in
  no crate.

- **WATCH: the title race the wrapper can only narrow.** *Observed pressure:* the merge wrapper judges three
  inputs and pins two by construction — the body travels as the value the gate judged, and the commit set
  through `--match-head-commit`, which GitHub decides atomically. The title is the third, and `gh` offers no
  `--match-title`, so it can only be re-read after the gate. That shrinks the window from a whole `cargo
  test` to one API call and leaves it open inside that call. *Observation source:* the round that added the
  re-read, and the review that found the residue cited a tracker whose trigger — *an act reaching either
  without the wrapper* — can never fire for a race reached only by going through it. *Current reaction or
  bound:* `repository-checks/a-title-edited-inside-the-re-read-itself-a-stated-bound`, and the re-read
  itself, pinned by `a_title_edited_while_the_gate_ran_stops_before_the_merge`. *Risk:* the squash records a
  subject that is no longer the title, in the one act that cannot be repaired — bounded by the window being
  a single API round trip and by the editor being a collaborator rather than an adversary. *Next trigger:*
  `gh` gaining a `--match-title`, or another judged input arriving that can only be re-read — the second
  would make this a shape rather than an instance. *Authority:* engine. *Compatibility:* patch; the wrapper
  ships in no crate.

- **WATCH: a merge or publish made outside the wrapper is not observed.** *Observed pressure:* both
  assertions guard the sanctioned path — the wrapper's `1 passed` and the reaction pinning the identifier it
  cites. A `cargo publish` run directly, or a merge made in the browser, reaches neither. *Risk:* the record
  or the published source escapes the gate that stands in front of it. *Next trigger:* an act reaching either
  without the wrapper. *Authority:* engine. *Compatibility:* none — reaching further means observing the
  operator's shell or GitHub's servers rather than this repository.
- **WATCH: which governance member a check belongs to is unobserved.** *Observed pressure:* the split
  between 繩墨 (the law and the delivered product) and 勘合 (this repository's record) is a judgement about
  what a check judges, and two mechanical rules were each measured unreliable — a text scan reads a comment
  naming `AGENTS.md` as governance while a check scanning every tracked file names nothing, and
  `TIANHENG_WORKSPACE_TESTS` means both "this needs the repository as its subject" and "this needs a fixture".
  *Risk:* a check lands in the wrong member and the two identities blur again, which is the failure the
  split was built to end. *Next trigger:* a third member, or a check whose placement two readers disagree
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
    accepted as written, and nothing checks that a participant's kind matches what it governs.
    *Risk, corrected by measurement:* the kind is the label a **report** and a **SARIF render** carry, so a
    consumer filtering by kind sees an outsider's findings as 圭表's. It does **not** reach a baseline. An
    earlier version of this entry said it did, and the code says otherwise: `BaselineEntry` carries
    `{ id, rule, finding, owner, tracker }`, `ViolationId` is `{ target, rule_key, fact }`, `RuleKey` is
    `{ rule_type, fields }`, `Baseline::to_json` emits no kind at all, and de-duplication is by `ViolationId`.
    So a borrowed kind cannot mis-group an adopter's recorded entries and cannot make one stale — which
    lowers this entry's urgency by exactly the half that would have forced a decision before publication.
    *Promotion trigger, made fireable:* the previous wording asked for a participant that is not this
    repository's own example, which **could not fire before `0.5.0` ships** — no third party can write an
    `Observer` against an unpublished trait, so the trigger demanded evidence the release itself has to exist
    to produce. It now reads: an `Observer` implementation outside this repository — an adopter's, or a
    second one here written for a different purpose — whose borrowed kind is shown to mislead a real report
    consumer or SARIF ingest. One example that chose its own label is still not evidence about adopters.
    *Version class:* minor at most. Adding a variant to a `#[non_exhaustive]` enum breaks no downstream
    match, and it changes no verdict; what it changes is the projection vocabulary, which is why it is a
    decision rather than an addition. Deferring is the reversible half: a variant, once shipped, cannot be
    unshipped. *Authority:* `observer-protocol`'s requirement that a participant outside the family be
    demonstrated joining a run, and that example's README, which records the finding where someone writing
    their own participant will meet it.

    *What the rest of the new vocabulary was measured against, and passed.* Before `0.5.0` publishes the
    observation protocol, every vocabulary type it adds was read against one question — does a third party own
    a value, or must they borrow one of this family's? `BoundaryKind` fails it because `Crate`/`Module`/
    `Semantic`/`Runtime` name 三儀's own dimensions. Every other type passes, because each names a **role** an
    outsider occupies rather than a member of this family: `Owner` is `Engine` (theirs) / `Inherited` / `Adopter`;
    `Extent` and `Reached` name positions any reaction can be in relative to its own observation; `Demonstrates`
    names directions any pinning test can take; `FactGranularity` names properties of a fact; `Defence` is
    `PinnedBy` (their test) or `Unpinned` (their tracker), and its two variants cover the whole space of *bounds*
    because a construction-held property is stated in requirement prose rather than declared as one. `BoundId::new`
    accepts any `Into<Cow<'static, str>>` precisely so a computed id is expressible. The recurrence this review
    was opened to look for is therefore not there, and that is the finding.
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

These are **not** open work. Each was a live item in this index and is now closed;
the original entry is kept verbatim beneath its closing note because the reproduction record —
what was observed, by which lens, and why the trigger was believed narrower than it was — is the part
that stops the same defect being re-found from scratch. The present-tense `Class:` / `Risk:` /
`Promotion trigger:` lines inside each retained entry describe the state **at the time it was written**.

They live here rather than under their own class heading because an index that carries a question and its
answer at once is a reader trap — the same reason a stale WATCH line was retired in `68e183b`, applied to
the larger entries it left in place. **Saying it twice held it nowhere**: the governance section states the
same rule from the other side, and entries closed after this section was written kept accumulating under
`### READY-PATCH` and `### WATCH / ACCEPTED / DECLINED / BUILT` regardless — struck through, complete with
their answers, under headings a reader consults to ask what is left to do. They are moved here, verbatim and
in the order they sat in, and `a_closed_entry_does_not_stay_under_a_live_class` now decides the property
instead of two paragraphs asserting it. Neither the class nor the number is restated collectively: each
retained entry carries its own `Class:` line, and a count here would go stale the next time an item
closes — which is how the previous two sentences came to say "DESIGN-BREAKING" and "six" about a section
that also holds a closed READY-PATCH record.


- ~~**Three spec requirements the shell-to-Rust migration orphaned.**~~ **CLOSED** in the open window via
  `fix/close-the-migration-orphans`. All three landed: `governance-dogfood`'s focused-matrix requirement
  removed, `reference-integrity`'s fixture-policy requirement restated as what the port does, and its
  boundary-family requirement rebuilt as a derivation held both ways by
  `crates/shengmo/tests/family_coverage.rs`. The entry is kept verbatim below because two of its own claims
  were wrong and the corrections are the part worth not re-deriving: it called the work "one OpenSpec
  lifecycle" for a lifecycle this repository has never run, and it promised a `## Subject` guard that already
  existed and would not have caught these anyway.

- **Three spec requirements the shell-to-Rust migration orphaned.** *Class:* READY-PATCH. *Observed
  pressure:* a contributed adversarial review found two; a sweep run to decide their shape found the third.
  *Observation source:* each of the twelve shell files `64ed18c` deleted, mapped to the spec requirement it
  implemented — nine were correctly swept when they were deleted, three were not. *Current reaction or
  bound:* none; each states a live `SHALL` that nothing runs, which is the failure this whole family exists
  to end, committed inside its own governance.
  - `governance-dogfood`'s focused-matrix ordering requirement and its three scenarios — **REMOVE**. The
    decision was made and recorded in `dd6e1f8`, in `dod_coherence.rs`'s doc comment and in the retired
    remediation queue; it never reached the spec.
  - `reference-integrity`'s fixture-policy requirement and its three scenarios — **REVISE**. The capability
    changed shape rather than vanishing: the port parameterises `offences_in` directly instead of accepting
    a fixture-set option, and `GOVERNANCE_DOCUMENTS` is a compile-time `const`.
  - `governance-dogfood`'s boundary-family coverage requirement and its two scenarios — **REVISE + BUILD**.
    Measured: every family it names does have an adopter-shaped owner today, so the substance holds and the
    gap is that nothing would notice one losing its owner. Build it **derived both ways** — families from
    the boundary types, owners from the examples and the self-law — never as a hand-kept inventory. Drop its
    stale `0.2.x` anchor and its unrelated `GovernanceTest` clause. *(This entry also claimed an asymmetry —
    that `AsyncExposureBoundary` was owned only in `sans-io-pure`'s `tests/reaction.rs` and not in its
    `src/governance.rs` like every sibling. **False, and corrected when it was checked rather than grepped:**
    `Constitution::sans_io_pure` constructs an `AsyncExposureBoundary` internally, so that example does
    declare the family on its imitable surface, through the profile. What the observation really found is a
    limit of the coverage reaction, filed as WATCH below.)*
  *Risk:* a reader consulting a spec is told a rule is enforced when it is not — the same class the bound
  register was built to end one level down. *Promotion trigger:* fired; this is the work, not a candidate
  for it. One branch and one pull request, because each orphan needs the same decision made per orphan
  rather than three separate repairs.

  **This entry claimed a guard would have caught two of them, and that was wrong too.** The guard named —
  every spec's `## Subject` resolving to tracked paths — **already exists** as
  `capability_subjects::declaration_offences`, held by `every_capability_declares_the_subject_it_governs`,
  and it was green the whole time: all three orphans' subjects resolve perfectly well while a requirement
  under them described a deleted mechanism. Specifying it as new work would have rebuilt a live reaction.
  What actually found them was reading each shell file the migration deleted and asking which requirement it
  had implemented — a judgement over prose. **No reaction is proposed for this class**, because deciding
  whether a prose-described mechanism still exists is the instrument this repository has measured and
  rejected three times. The residual is stated instead: a requirement can outlive its mechanism, and only a
  reader comparing the two will see it.

  **This entry said "one OpenSpec lifecycle" and that was wrong about this repository.** Measured: the
  `explore → propose → apply → sync` lifecycle `AGENTS.md` then described has **never run here** — zero
  `docs(openspec): propose`/`sync` commits in the whole history, and `openspec/changes/` untouched since
  `2f903fb` (`release: 0.1.0`), which is when its `.gitkeep` was added. The `v0.4.0..0.5.0` window edited
  `openspec/specs/*/spec.md` in 151 commits, every one an ordinary `fix:`/`feat:`/`docs:`. `READY-PATCH` is a
  **compatibility class** in this file's own definition above, not a process exemption, and conflating the
  two is what produced the wrong claim. *Version class:* patch; repository-internal, shipping in no crate.
  *Authority:* `governance-dogfood` and `reference-integrity`.

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

- **`crates/kanhe/tests/reference_integrity.rs` has no companion failure matrix.** *Class:* READY-PATCH,
  reopened. Previously marked CLOSED here as of the open window, on the claim that
  `crates/kanhe/tests/reference_integrity.rs` gained a throwaway git-repository fixture proving every
  refusal (exit 1 and exit 2) and pass direction. **That closure named the wrong subject and does not hold
  at `HEAD`.** The fixture described was the shell era's throwaway test companion to its
  `reference_integrity` gate script, added by `f41b3b9` — both deleted by `64ed18c` when `scripts/`
  migrated to Rust. *Observed pressure:* the current `crates/kanhe/tests/reference_integrity.rs` (1,300+ lines) has
  no `git init` fixture, no exit-code matrix, and never uses `kanhe::refusal::Kind`; its `scratch()` helper
  builds plain temp directories for unit-testing isolated parsing subroutines, not a fixture repository
  driving the gate's own pass/violation/cannot-judge behaviour against itself. *Observation source:* direct
  inspection of the file at `HEAD` (13 commits since the migration, none adding such a fixture) and `git
  show f41b3b9`/`git show 64ed18c --stat` for the addition and deletion. *Current reaction or bound:* none.
  *Promotion trigger:* the fix this entry previously claimed, actually done — a fixture-driven
  exit-1/exit-2/pass matrix for the current Rust gate. *Version class:* tests only; no published surface.
  *Authority:* this entry's own prior (incorrect) closure, which an adversarial contract review over
  `v0.4.0..HEAD` refuted by reading the file rather than the claim.

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
  capability — the same misjudged cost as the absolute-`#[path]` entry earlier in the 0.5.0 window, whose
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

- ~~**A capability's declared subject does not reach every file whose change it should file, and the one
  declared false negative covering that was observed costing something.**~~ **CLOSED** in the open window
  via `change/claim-dimension-adopter-surface-subjects`. `adopter-surface`'s declared `## Subject` now names
  all four `adopter_surface.rs` files — the composed shell's alongside `guibiao`'s, `hunyi`'s, and
  `louke`'s — and `capability_subjects.rs`'s unclaimed-file count dropped from 109 to 106 (claimed 286 to
  289), exactly the three this closes. An independent adversarial review confirmed the factual premise and
  found no other capability already claimed any of the three files. The bound this entry was filed against
  (`repository-checks/files-no-capability-claims-a-stated-bound`) is untouched and its reason still holds —
  this closed one instance the promotion trigger named, not the bound itself. *Class:* READY-PATCH.
  *Observed pressure:* `observation_bounds()` reached the public surface of three published crates in the
  `0.5.0` window, was advertised as the way to read a dimension's declared bounds without composing a run,
  and was named by no `adopter_surface.rs`. It was not an oversight that a sweep would have caught: the
  three dimension contracts are tracked files **no capability's declared subject claims**, so the filing
  join never had them in scope. `adopter-surface`'s subject named only the shell's three files.
  *Observation source:* `crates/kanhe/tests/capability_subjects.rs`, whose
  `files_no_capability_claims_are_reported_rather_than_implied_judged` prints the unclaimed count on every
  clean run — the figure is produced there and deliberately not repeated here. *Current reaction or bound:*
  the declared bound `repository-checks/files-no-capability-claims-a-stated-bound`, an `UnderReacts` with
  `Owner::Engine`. Its stated reason is that requiring subjects to tile the repository would buy coverage
  with a claim per capability nobody could defend, and that reason still holds. *Risk:* a member of a
  published promise reaching `0.5.0` unheld — realised once already, closed for this member by naming it in
  the three contracts. *Promotion trigger:* a **second** distinct instance of a published surface member
  landing in an unclaimed file, or an adopter-visible defect traced to one. Tiling the repository is
  explicitly not the candidate; extending an **existing** capability's subject to the files it plainly
  governs is, and `adopter-surface` gaining the three dimension contracts is the worked example.
  *Compatibility class:* patch — the subject declarations and the filing join ship in zero packages.
  *Authority:* `openspec/specs/repository-checks/spec.md`'s *A capability SHALL declare the subject it
  governs* and the bound declared in `crates/kanhe/src/bounds.rs`.

- ~~**`observation-bound-register`'s own spec asserts a command's outcome that nothing runs.**~~ **CLOSED**
  in the open window via `fix/wire-openspec-validate-into-ci-dod`. `npx --yes @fission-ai/openspec@1.4.1
  validate --specs --strict` is now a line in both `AGENTS.md`'s Definition of Done and CI's `dod` job, so
  the claim the spec's own scenario made is now held by a reaction rather than by prose. Closing it also
  required a genuine fix, not only wiring: `repository-checks`'s own "The prelude promise SHALL be held
  against the contract compiled from outside" requirement failed strict validation because its opening
  sentence — the only line the validator's parsed `text` field reads — carried no SHALL/MUST keyword, even
  though the rest of the paragraph did; reworded the opening sentence to lead with the SHALL. *Class:*
  READY-PATCH. *Observed pressure:* the spec states `openspec validate --specs --strict` passes over every
  spec; measured, two did not, and `git grep` found the command named only in the scenario asserting its
  result — not in `.github/workflows/ci.yml`, not in `AGENTS.md`'s Definition of Done, not in any script.
  `CHANGELOG.md`'s entry recording the measurement said this gap was closed by adding the command to both;
  it was not — corrected in the same edit that added this entry. *Observation source:* `git grep -n
  "openspec validate"` over the tracked tree at `HEAD`, one hit, in prose. *Current reaction or bound:*
  none — the scenario's claim is unheld by anything that runs. *Risk:* a spec passing `openspec validate`
  is a precondition the requirement corpus assumes and nothing defends; a spec that later fails validation
  would surface only if someone runs the command by hand. *Promotion trigger:* wiring `openspec validate
  --specs --strict` into `ci.yml` and the Definition of Done, pinned to a validator version — the fix this
  entry's own prose already claimed once. *Version class:* patch — a CI/DoD addition, no published
  surface. *Authority:* `openspec/specs/observation-bound-register/spec.md`'s scenario asserting the
  command's result.

- ~~**The construction-held list is hand-maintained prose.**~~ **CLOSED** in the open window via
  `fix/construction-held-list-verification`, and by a cheaper route than the one this entry's own prose
  called for. The recorded plan was a perturbed build through `pin_bites.rs`'s mutated-worktree machinery,
  extended to carry a declaration whose subject is a spec sentence rather than a pinning citation. Reading the
  built-in composition path's own source instead answered the same question directly: for a construction-held
  dimension the built-in path does not call some function that happens to agree with the observer today, it
  *constructs that dimension's own `Observer` and calls `.observe()` on it* — one implementation to find, not
  two runs to compare. `crates/tianheng/src/runner.rs`'s `evaluate_constitution` was read to confirm this
  textually: the semantic and runtime arms each construct `SemanticObserver`/`RuntimeObserver` directly, and
  the static arm still calls `check_and_cover` without ever constructing `StaticObserver`. Added
  `the_construction_held_list_matches_the_built_in_composition_path`, and re-pointed the spec's scenario from
  `UNPINNED` to `PINNED-BY` it. Verified both negative directions: moving `SemanticObserver::new(...)` out of
  `evaluate_constitution`'s own body (into a sibling function it merely calls) fails the new test, and adding
  a `StaticObserver::new(...)` construction inside `evaluate_constitution` also fails it — restoring the real
  implementation passes both. *Class:* READY-PATCH. *Observed pressure:* `observer-protocol` requires the
  spec to say which dimensions' equality holds by construction, and nothing observed that the list was
  correct. The 0.5.0 window is the evidence: the list named runtime alone, the shell's semantic arm changed
  under it in the same window, and the list was repaired **by hand**. Falsifying it — say, claiming static is
  construction-held and runtime observed — passed the whole workspace suite and every gate. *Observation
  source:* the final sweep of that window, which ran exactly that perturbation. *Risk:* a reader takes a
  constructed equality for a measured one — the failure the requirement's own sentence exists to prevent, in
  the sentence that prevents it. *Promotion trigger:* fired; the list went stale inside the window that wrote
  it. *Version class:* patch; repository-internal, shipping in no crate. *Authority:* `observer-protocol`.

- ~~**Fixture scratch roots are claimed with `create_dir_all`, which adopts a pre-existing symlink.**~~
  **CLOSED** in the open window via `fix/shared-scratch-claim-helper`. The shared helper this entry called
  for now lives in `xingbiao` (the lightest-weight crate already depended on by `guibiao` and `hunyi`, and
  optionally by `louke`) rather than in `kanhe`, so it reaches every crate without amending `kanhe`'s
  dependency law: `xingbiao::claim_scratch` wraps `create_dir` with the same reasoning
  `kanhe::publish_source_gate::claim_scratch` already carried for its one production site, which is
  untouched — this closes the **test-fixture** instances, a different layer than the production gate the
  first instance was found in. Migrated 37 call sites across 24 files in all five crates (`guibiao`,
  `hunyi`, `louke`, `tianheng`, `kanhe`), leaving untouched every site that only builds a subdirectory
  *within* a root some other call had already claimed — that half was never the exposure. Added `xingbiao`
  as a `dev-dependency` to `tianheng` and `kanhe` (which lacked it; `guibiao`/`hunyi` already carry it as a
  regular dependency, and `louke` gained an unconditional dev-dependency alongside its existing
  `audit`-feature-gated optional one, so `cargo test -p louke` — audit OFF — can still reach it).
  `xingbiao`'s own test suite gained three cases, including a direct reproduction of the defect this closes:
  a pre-planted symlink that `create_dir_all` silently adopts and `claim_scratch` refuses. *Class:*
  READY-PATCH. *Observed pressure:* the Gate 8 review of the 0.5.0 window found it in
  `publish_source_gate::verify_tag_signature`, where the scratch holds `tag.sig` and `check_novalidate` reads
  it back — so a redirected directory let someone substitute a signature over the same payload made with
  their own key, in front of `cargo publish`. That site was closed by `claim_scratch` before this entry was
  filed. *Observation source:* a sweep of every `create_dir_all` in `crates/`, run when that fix landed:
  three harnesses already claimed their roots with `create_dir` and an `AlreadyExists` arm —
  `merge_workflow.rs`, `publish_workflow.rs`, `reference_integrity.rs`, which are the ones that put a
  controlled `bin/` on `PATH` — and roughly forty fixture roots across `guibiao`, `hunyi`, `louke`,
  `tianheng` and the remaining `kanhe` directions did not. *Risk:* bounded and unlike the closed production
  one — a redirected **fixture** root corrupts a test run on a developer's machine; it reaches no release
  artefact and no irreversible act. Measured on the machine this closed on: `create_dir_all` on a
  symlink-to-directory returns `Ok(())` and the writes land in the link's target, `create_dir` returns
  `AlreadyExists`, and `remove_dir_all` removes the link rather than following it — so every one of these
  was a race in a window, not a plant-and-wait. *Version class:* patch; repository-internal, shipping in no
  crate. *Authority:* `publish-source-integrity`, which paid for the first instance.

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

  **Retired two commits later, in the same window.** `gate-shape-contract` itself — the capability this entry
  closed with — was retired by `5abda1f`, once `scripts/` held nothing but `merge-pr.sh` and `publish.sh` and
  the capability's own projection read `0 gates, 11 properties each`: nothing left to pair, nothing left to
  enumerate. The pairing this entry describes, and every property and bound declared under it, no longer
  exists at `HEAD`. Recorded here rather than deleted, since the shape this entry closed — a gate's own
  convention, re-learned by breaking — is the part worth keeping; the capability that closed it is not the
  part that lasted.

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

  **Retired with the rest of `gate-shape-contract`, by `5abda1f`.** The eleventh property, the shared capture
  library, and the `--ordinary-empty` per-call-site contract this entry describes were all part of the same
  capability, deleted whole once it reached the vacuity its own bounds warned about. Nothing at `HEAD` still
  asserts either failure direction measured above; a future instance of a swallowed subshell status would
  need a new reaction, not a resurrection of this one.

- ~~**The pre-publish gate had no specification, and its stated bound had the cause backwards.**~~ **CLOSED** in
  the open window as the `publish-source-integrity` capability. Found in the 0.5.0 pre-release review, and worth
  keeping for the shape rather than the fix.

  All 34 specifications were searched: **none** stated that a publish must come from a signed annotated tag at
  the tip of `main`. The gate standing before the one irreversible act carried its contract in its own header
  comment, while `gate-shape-contract` exempted it from Definition-of-Done membership *by name* — the one place a
  reader was told it is special. A reaction with no requirement is the mirror of the class the 0.5.0 window kept
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

  Lesson kept, because it recurred four times in the 0.5.0 window: a gap suspected from a partial view dissolves
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

- ~~**WATCH: `AGENTS.md`'s OpenSpec lifecycle section describes a process with no instances.**~~
  **CLOSED** in the open window via `fix/declare-the-openspec-adoption-mode`. The diagnosis in the entry
  below was the second of three, and the third is the one that was acted on. Not *unrealized aspiration*
  and not *prose contradicted by practice*: **an adoption decision made from the beginning and written
  down nowhere.** OpenSpec offers a `specs` half and a `changes` half; this project uses the first, has
  never used the second, and `openspec/config.yaml` declared neither. So `AGENTS.md` described the mode not
  chosen, and a reader had nothing telling them it was a choice. The call is now in `PROJECT.md`'s
  Decisions with what would change it, and `AGENTS.md` states how a capability change is worked in the mode
  actually in use. `openspec/changes/archive/.gitkeep` **stays** — its job is now optionality rather than
  archive hygiene, so adopting the other half needs no exception — and nothing enforces the mode,
  deliberately, since a check that fired on a change directory would prevent that adoption.

  *Note, 2026-08-21: the mode this closure recorded was itself wrong, and the correction is in `PROJECT.md`'s
  Decisions. `has never used the second` was measured over the release spine and stated over the repository:
  `git log --all` reaches dozens of change directories, and this entry's own observation source — `openspec/changes/**`
  untouched since `2f903fb` — reads only what a release-spine checkout tracks. The closure's diagnosis
  survives it: the adoption decision was written down nowhere, and writing it down was right. Which mode got
  written is what the correction moves.*

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

- ~~**WATCH: the rule shape the self-law relies on most is absent from `examples/`.**~~ **CLOSED** in the open
  window, by its own trigger firing and being noticed rather than by a decision to go looking. The trigger was
  *an example is added or revised for another reason, at which point the shape is chosen deliberately rather
  than by omission* — and `examples/observer-participant` was added mid-window while
  `grep -rc restrict_dependencies_to examples/` still returned zero. The shape had been chosen by omission one
  more time, and the entry existed to make that visible.

  `examples/guibiao-standalone` now declares a crate-level allowlist beside its module boundary. The landing
  site was not a free choice: that example's manifest already claimed its one-dependency footprint **is** the
  圭表 pitch, "demonstrated here rather than asserted" — a claim in a comment with no reaction, which is the
  open loop this project exists to close. Declaring it makes the pitch react.

  **The evidence a holding boundary needs is different, and that shaped the change.** A passing allowlist is
  indistinguishable from one reading the wrong thing, so `tests/reaction.rs` points the same shape at the same
  real manifest with the dependency excluded and requires it to name that dependency and gate. Measured
  separately, by adding a second dependency to the manifest and running the demo: the declaration in
  `governance()` reports it and exits 1.

  **Two figures in this entry had drifted before it closed**, which is the class the window spent itself on: it
  said the rule carries six of eleven self-law boundaries, and measured it is neither number. They are not
  replaced with corrected ones — the projection enumerates that set, and the property the sentence needed is
  that the rule carries *more of the self-law than any other*, which is true without arithmetic. *Version
  class:* patch; `examples/` ships in no crate. *Authority:* `governance-dogfood`, which owns the examples'
  reaction. **Still not claimed**, unchanged from the original entry: that enumerated denylists are wrong, or
  that the self-law avoids them. It does not — its inline-symbol-path confinements each carry an enumerated verb
  list, and `inline-symbol-path-confinement` declares the unlisted remainder as a bound the adopter owns.


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

  **What earns the minor is not classified here, and that is the repair rather than a smaller claim.**
  `CHANGELOG.md`'s `[0.5.0]` marks every entry requiring adopter action `**BREAKING**` and its `### Migration`
  section states each step. That classification has one owner and this is not it. What belongs here is the
  version *consequence*: pre-1.0, an adopter having to act earns a minor whatever the diff size, and a recorded
  baseline going stale is such an action — which is why this window is `0.5.0` rather than `0.4.1`.

  This paragraph used to classify it anyway: it named the bare-principal resolver closure as the one item, and
  called the rest of the window's public surface additive and therefore free. Both halves went stale inside the
  window. Another entry earned the mark — `Outcome::Clean` gaining the subject it was measured over — and it is
  part of the very surface the sentence called additive, carrying a `### Migration` bullet that reads *the
  compiler names every site*. Two owners for one classification, and the one with no producer drifted, which is
  the shape *Bind a claim to its measurement* refuses everywhere else.

  Re-derive the range rather than trusting a figure here — the counts this paragraph used to carry ("of the 44
  commits … the two other product-code touches") were written early and falsified by the window itself:

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

  What was deliberately not done **when this entry was written**: the version bump, the dated CHANGELOG
  section, the internal pins and `Cargo.lock`, which move together at release preparation. `a9b3f42` has
  since done all four, so the gate reports `release-ready: 0.5.0` rather than the `development: 0.4.0` this
  paragraph described. Corrected rather than deleted, because the point it was making survives and is the
  one that matters here: the gate reads versions, never a branch name, so the rename changed nothing it
  judges.
- **Next breaking window (if earned)** — requires real adopter or correctness pressure.

## Explicitly not on the roadmap

- Active code-shaping / generation.
- Prescriptive framework you build inside.
- Lints (opinionated style checks rather than declared intent).
- Universal graph API (whole-graph analysis rather than declared per-target boundaries).
- Supply-chain policy engine (cargo-deny's lane).
- DSL macro consolidation (repetitive builders are designed-to-be-imitated for 潛移 gravity; leave explicit).
