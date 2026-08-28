# AGENTS.md — 天衡 (Tianheng)

Working agreement for humans and AI agents. `PROJECT.md` is the contract (the *why* and
the invariants); this file is the *how* of contributing. Keep both short.

## Agent workflow — read the law, react against it, repair toward the reason

When you (human or agent) change code in a Tianheng-governed project, work *with* the
reaction, not around it.

**AI context order** — entering this repo, read in this order, then stop: `PROJECT.md` (the
contract and the 潛移 thesis) → [`AGENTS.self-law.md`](AGENTS.self-law.md) (the enforced self-law,
in imitable form) → the relevant `openspec/specs/*` (the capability you are touching) → the code.
`PROJECT.md` and this file stay short on purpose; the law's per-boundary detail lives in the
generated projection, and requirement detail in the specs — read those, do not inflate these.

Where the law stops is written down too: [`docs/observation-bounds.md`](docs/observation-bounds.md)
projects every **observation bound** — each claim that a product reaction or repository check deliberately stops at a named shape —
with the test that defends it, or the tracker that owns closing the gap. Read it before reporting a
behaviour as a defect: a declared bound means the shape is governed policy, and the projection leads with
the count of bounds nothing yet defends.

1. **Before changing code — read the declared law.** `tianheng list --format markdown`
   (or `--format json`) projects the whole constitution: every boundary's target, what it
   forbids or restricts, and its declared reason. Read it so you know the architectural
   shape you must not drift. (The published binary's `list` projects the *demo* constitution;
   for Tianheng's **own** enforced self-law, read [`AGENTS.self-law.md`](AGENTS.self-law.md) — a
   projection generated from `crates/shengmo/src/law.rs` and staleness-checked by `cargo test`.)
2. **After changing code — react.** `tianheng check --format json` evaluates the
   constitution against the workspace. Exit `0` is clean (or warn-only / fully baselined),
   `1` is an enforced violation, `2` is a constitution/scan/usage error.
3. **On a violation — repair toward the declared reason.** Each violation carries its
   `reason` — the intent the boundary protects. In any projection (text report, `--format
   json`, `--format sarif`), **read the `reason` first** — it is the repair direction — then
   `file` (where), then `finding` / `rule` (what tripped). Repair the code so the reason holds
   again; do not weaken the boundary to make the reaction pass.
4. **To change the law itself — amend it deliberately.** A boundary is wrong only by a
   human-reviewed amendment (a spec amendment / steward review), never by quietly editing
   the constitution so CI turns green. Before amending, read the law
   projection (step 1) so the amendment reasons against the declared shape, not a guess.

This SOP is **orientation, not the binding mechanism**: the reaction (a failed `check`, a
runtime probe) is what binds: reading the law first does not *grant* compliance, it just
saves a round-trip. It is convention, not constitution — an observable architectural fact
belongs in the declared law and reacts; a working agreement like this one does not, so the
drift law keeps it here, not in `Constitution`.

## Writing a boundary's `reason` — for 潛移 (gravity)

A boundary's `because(...)` is read twice: once by a human, and — projected into an agent's
context by `list` — once by an autoregressive model that *imitates* it (see PROJECT.md, 潛移).

Governance has **three carriers**, and which one a claim belongs to is decided by what can hold it:

- **The reaction (code)** — functional boundaries (`restrict_dependencies_to`, `must_not_call_inline`) enforce
  hard invariants. Minimalism forbids redundant reactions (do not add a denylist for a prohibition an
  allowlist already enforces).
- **The reason (prose, projected)** — `because(...)` reasons project into `AGENTS.self-law.md` to condition
  LLM continuations. Write reasons strictly in a **forward voice** ("the kernel depends inward only"), never
  as a backward justification or historical debrief ("we once hit a cycle in 0.2.2"): **provenance belongs in
  `PROJECT.md` decisions and git history, not in the live context reason.**
- **The provenance (record)** — historical rationale, lessons learned, and decision context stay in
  `PROJECT.md` decisions and commit history, keeping live context dense and noise-free.

**Named, not numbered, because the numbering collided.** `COOKBOOK.md` and `self-law-projection`'s own
`SHALL` both use *Layer 1 / 2 / 3* for the **Three-Layer Agent Law** — the anatomy of the projected document:
preamble, generated body, Rust law source. This paragraph used the same three ordinals for a different
subject and put the reaction backstop at Layer 1 where the other two put it at Layer 3. One vocabulary, two
referents, inverted — which is the shape this repository removes on sight, in the governance documents
themselves. The ordinals now have one owner; these three have names.

Keep every reason **within the boundary's observable perimeter** — a reason must never assert structure the law does not react to (that is prose prescription, an open loop with no backstop). Forward voice, bounded to what reacts, minimal in reactions.

## What earns a place in a doc comment

A `///` or `//!` passage carries the item's contract. **The test is whether the passage carries an
observation source or a falsifier for a claim the item makes.** Where it does, it stays and is load-bearing;
where it is only *how the code got here*, it is provenance and belongs with the record.

That distinction, and not a ban on past-tense verbs, is what separates the two. `Measured:
bash -c 'printf a;#b' prints a, so bash opens a comment there` is the **observation source** for a declared
over-inclusion — delete it and what remains is an assertion nothing can falsify, which is the defect class
this file spends four rules closing. `fixed in round 6` is provenance: it names when, not what, and nothing
downstream reads it.

Applied to the shapes a review is likely to file:

| shape | disposition |
|---|---|
| a measurement, with the command or the corpus that produced it | **stays** — it is the observation source |
| a rejected alternative **with** what measuring it showed | **stays** — it is what stops the next person re-running it |
| a version range or sweep that *found* something | the finding stays; the sweep's identity is provenance |
| a past defect described **with** the invariant it violated | keep the invariant, drop the debrief |
| a review round number, a pull request number | provenance |
| a rejected alternative with no measurement | provenance |
| **a relative anchor — `this window`, `the previous round`, or a `path.rs:120` into this tree** | neither: it names a moving reference, so it is stale the moment the window closes or a line is inserted above it — and unlike a retired term, **nothing goes red**. Anchor it to the moment, or name the item (a function, a scenario, a direction) instead of its position. Measured when this was written: the tree held no such reference, and the only `path:line` in it was a **reaction's own output** — a gate saying where it found an offence, which is the one place the shape is right and the reason this is a rule and not a check |
| **a commit object in live text** | neither, and this one **has a reaction**: `main` carries one commit per release, so a development commit is unreachable from a fresh clone by construction. Anchor to the release window |
| **a branch a guard has already made unreachable** | neither. A reaction closes the **decidable** part — `str::split` and `str::rsplit` always yield an item, so a fallback on one is dead — and stops there: `xs.max()` or `entry.file_name()` are always-`Some` only on a non-empty producer, which takes the surrounding code rather than the line, so widening the reader would refuse live sites. The rest is this row |
| **a hosting serial in live text** | neither, and this one deliberately has **no** reaction. A reader over text cannot decide it: the bare shape *is* the fixture for the squash-serial check, and inside one clause a cue and a numeric value are told apart only by meaning. One was built, needed three declared bounds to say what it could not decide, caught nothing, and was withdrawn — a narrow instrument defending a wide rule is worse than the rule alone |
| **a whitespace run inside an emitted refusal message** | neither, and this one deliberately has **no** reaction. Measured across every `violation_at` / `cannot_judge_at` message in the tree — the corpus, not a count of it: after the two instances found by review were repaired — one in a refusal, one in a declared bound's reason, which a projection was carrying — the only remaining run is column alignment a reader wants (`subject:` above `title:   ` in one multi-line refusal). A universal rule would refuse that, and a rule with it allowlisted is two lists that must agree. The measurement had to be revised several times before it could see the known instance at all — a predicate returning zero over a corpus it cannot see into is indistinguishable from a clean one — which is the other half of why the rule is a rule and not a check. `CHANGELOG.md`'s dated entry carries the figures, where a measurement belongs |

**Invariant first, observation second, in separate sentences.** The disposition above decides what stays; this
decides how it is written, and it is what makes the table applicable without a re-read. A passage that fuses
the two — *delegates rather than re-deciding: the two used to be separate implementations and answered
differently at the overflow boundary* — cannot have its provenance trimmed without taking the observation
with it, and a reviewer reaching for the debrief proposes deleting the falsifier. Measured: two reviews of
this repository proposed exactly that substitution for two such passages, and in both the replacement left an
assertion nothing could falsify. Split into two sentences, each edit reaches only what it means to.

**No reaction, and the reason is structural rather than a count.** The decidable subset collides with itself:
the bare `#NNN` shape a detector would match includes the **fixture** for the squash-serial check, so a
detector for that shape would refuse the very check that forbids it. The larger classes are not decidable at
all — each needs the criterion applied per site, which is a judgement over prose this repository has designed,
measured three times and rejected. So the criterion is the instrument, and the relative-anchor row is the one a
sweep can enumerate.

**No figures are given here, and what stood here is why.** Four counts sat in this paragraph with no producer
and no anchor, inside the section whose own subject is that hand-written figures drift. Three were wrong by the
time two independent reviews measured them — and wrong *because* sweeps in the same window, from the same run
that typed them, had moved what they counted. A census belongs where something enumerates it, and this
paragraph enumerates nothing.

## Document authority & provenance

Each document has one job, so a fact lives in one place. `PROJECT.md` is the contract — the *why*
and the invariants, with significant calls recorded in its Decisions section.
[`AGENTS.self-law.md`](AGENTS.self-law.md) is the enforced self-law, projected from
`crates/shengmo/src/law.rs` (never hand-edited). `openspec/specs/*` is the per-capability requirement
truth. `BACKLOG.md` records deferred work and explicit non-goals. This file is the operating
protocol for humans and agents. **Provenance — why a change was made — lives in its commit body and
PR, not a separate ADR file class.** When two documents conflict, fix the conflict (an OpenSpec
change, or a doc PR) before building on it.

Backlog entries are decision inputs, not an undifferentiated wish list. Classify live work by its
evidence and next trigger (`READY-PATCH`, `DESIGN-BREAKING`, `WATCH`, or `ACCEPTED DEBT`), keep
rejected directions under `DECLINED`, and move shipped work to `BUILT / HISTORY`. Promotion into
implementation requires the entry to name its observation source, risk, compatibility class, and
authority in `BACKLOG.md`; a breaking candidate does not promise a minor release until its recorded
trigger fires.

## Working a capability change — OpenSpec, both halves, one requirement truth

**Both halves are used, and they answer different questions.** `openspec/specs/*` is the
per-capability requirement truth — the one place a requirement is read from.
`openspec/changes/<slug>/` is the **working lifecycle of one open change**: `proposal.md`,
`design.md`, `tasks.md`, and the delta spec, written and committed on the development branch while
the change is being worked. Where a requirement lives and how one change is planned are separate
questions, and the two halves answer one each.

So a capability change is:

1. **Read** the capability's spec, and the law projection the SOP above names.
2. **Propose** — write `openspec/changes/<slug>/` if the change wants a plan. It is optional; a
   small change may skip straight to step 3.
3. **Write the requirement onto its spec** — a new or amended `### Requirement:` and its
   `#### Scenario:`s, edited in place on `openspec/specs/*`.
4. **Write the reaction that answers it**, in the same branch.
5. **Strip the change directory** before the squash, so the requirement has exactly one home.
6. **Land the rest as one squash pull request**, whose subject names the product effect
   (`feat(xuanji)!: …`, `fix(hunyi): …`, `refactor(guibiao)!: …`) per *Commits & PRs* below.

**Step 5 is the invariant, and it is what the two halves being used does not cost.** `main` and
every `release/*` track nothing under `openspec/changes/` but `archive/.gitkeep`, so a closed
change's requirement is read from its spec and from nowhere else — a proposal is never a second
place to read a requirement from. What is transient is the plan, not the requirement.

The spec and the reaction move together **because they are one change**, not because a sync step
merges them later. A pull request that edits a requirement and ships no reaction for it is the thing
this arrangement exists to make visible.

**The seam runs both ways, and only one direction had an owner.** The rule below asks a *scenario* to name
its reaction. The reverse — requirement prose gaining a clause with no scenario, or a scenario gaining a
`PINNED-BY` with no clause declaring what it pins — had none, and both halves of it landed in one commit whose
own subject was doc-comment discipline: a `SHALL` about that specification's own prose that nothing could hold,
and a third direction pinned by a scenario the requirement never mentioned. Requirement prose and scenarios are
edited in one pass, or they diverge in whichever direction was not looked at.

**Every new or materially changed scenario carries its observation evidence in that same pull
request**: either name the existing reaction or repository check in the PR's `## Verification`, or
add a new guard and record the required negative run. If a property cannot fail because the data
model constructs it, state that construction in requirement prose rather than inventing a scenario.
A scenario with neither form of evidence does not belong in a spec.

**Four repository checks already encode the lifecycle, which is how the paragraph this replaces was
found to be false.** `reference_integrity` excludes a change directory from its corpus — a plan
names the files it intends to create, so its references are forward by construction — and
`openspec/specs/reference-integrity/spec.md` states that exclusion as a requirement with its own
scenario, so a *capability spec* is one of the authorities saying active plans exist. A basename
tracked only under `openspec/changes/` is that same lifecycle's vocabulary rather than a stale
reference. `law_restatement` excludes it alongside `docs/history/` and `CHANGELOG.md`.
`capability_subjects` takes it as a corpus: it holds a proposal's declared capability set against
the subjects its diff touches, which is a direction that runs on a development branch and is empty
on the release spine.

**The `change/<openspec-name>` branch role is retired, and the half is not.** *Branching and release*
below once named it as one of two fixed roles; a capability change now takes `<type>/<scope>-<slug>`
like every other piece of work, and a change directory rides that ordinarily-named branch. Retiring
the role was right and the sentence that retired it over-reached: it said the role went *with the
half*, and the half stayed.

Nothing enforces any of this, deliberately: a check that refused a change directory would forbid the
lifecycle, and a check that required one would forbid the small change that skips step 2.

## Bind a claim to its measurement — construction where you can, a reaction where you must

A claim about this repository — a figure, a verdict, a declared set — is bound to whatever measures it, and
**how strongly depends on what carries the claim**:

- **The claim carries no information the producer lacks, and nothing downstream filters on it** → *derive it*.
  Do not keep a list. The retired dimension-list bound is the case: a literal beside an enumerator that
  contributed nothing was two things that had to agree, so one of them was removed rather than checked.
- **Something downstream filters on the claim** → *declare it, and hold it to the producer both ways*. The
  literal is not a weakening here; it is what gives the enumerator something to disagree with. Measured:
  removing `guibiao` from `self_governance`'s dimension list left a `guibiao` allowlist naming `hunyi` green,
  because the coverage assertion filtered on the literal. A set-equality against the enumerator closes that,
  and a one-way check does not.
- **The carrier is text** → *a reaction is the only option*, and that is the weak branch by necessity. You
  cannot make a Markdown sentence unconstructible, so a sweep compares it after the fact.

**A claim carried by a value belongs in the first two branches, never the third.** `Outcome::Clean` was a
value asserting the result of work while carrying none of it, and nothing compared it to anything: the weakest
possible binding on the carrier that admits the strongest. It now carries the subject it was reached over, and
the combination that would be a lie is unconstructible.

*A census is produced, never typed* is this law's **text branch**, not a special case beside it — the strongest
binding available when the claim lives in a sentence. It stays stated on its own below because it is the branch
with a working instrument and a measured cost, and because a general principle nobody can check is the shape
this repository dissolves rather than keeps.

**This rule has no repository check, and that is stated rather than left to be discovered.** Its text branch
does — `crates/kanhe/tests/census.rs` holds every declared census — but the rule *above* that branch asks
which binding a carrier admits, and answering it means reading what a value is a claim **about**. Nothing can
see that: a check would have to know that `Outcome::Clean` asserts the result of work while `Severity::Warn`
names a category, and the difference is intent rather than shape. What the rule is instead is the question to
ask when a new check is written or a review names a corpus defect — and where it is answered by construction,
the compiler enforces it afterwards rather than this paragraph. `BACKLOG.md` carries it as the un-reacted-SHALL
class requires.

## A reader reads its whole subject — four shapes, and only three close by construction

The dominant defect class this repository ships, by count: a reader whose input is narrower than the thing it
claims to judge, reporting clean over a subject it never read. It stays correct for exactly as long as a second
instance happens not to exist, so it is found by review rather than by running anything.

**Name the four separately.** They have different remedies, and treating them as one class produces one tool
and leaves the rest open — which is this very failure applied to its own repair.

- **Lossy selection** — several candidates exist and one is taken. `split_once("pub use super::{")` read the
  first of the prelude's re-export statements; `trim_end_matches("::*")` folded a glob's whole set into one
  identifier. *Closes by construction:* make the candidates a value first, then answer **how many** with
  `kanhe::selection::{all_of, the_only}`. The bug is never that the wrong answer was chosen — it is that no
  choice was made, because `split_once`, `.next()` and `.first()` are reached for by habit.
- **Lossy acceptance** — input the reader cannot understand is skipped rather than refused. `machinery_names`
  `continue`d on a failed prefix strip and enumerated 0 of 8 members. *Closes by construction:* a return type
  with a third state, exactly as `capability_subjects::Declared` already does — `Absent`, the value, and
  `Unreadable`. This needs no new policy; it is a precedent to copy.
- **Lossy accumulation** — everything is read and the accumulator cannot hold it. `unpinned_bare: bool` could
  not answer *more than one* because two collapse to `true` as one does. *Closes by construction:* widen the
  binding to what the domain admits. A flag cannot count.
- **Corpus narrower than the claim** — the reader's input set is a subset of what the requirement governs.
  `marks_a_bound` gated citation resolution so 5 of 75 were never validated. *Does not close by construction:*
  only a set comparison in both directions catches it, and there is nothing to compare against until someone
  states what the subject is.

**The falsifier is uniform, and it is not "give it two".** Give it two **where the second is the candidate this
reader would have dropped**, and assert that one appears in the result. Two identical inputs can be masked by
deduplication; the dropped one cannot. The negative run then needs no separate fixture — restore the narrower
read and the case fails naming the member that vanished.

**Finding instances is a class-directed sweep, and it belongs in every pre-release review.** The shapes above
say what to look for; this says how, and it is a **review procedure rather than a reaction** — a distinction
that is load-bearing here. `inline-symbol-path-confinement` declares a receiver-method read unobserved, and
`split_once`, `next` and `first` are receiver-method calls, so this product's own dimensions cannot see these
shapes at all. A reader applying two predicates can.

1. Sweep the modules that parse or enumerate anything for `split_once`, `next`, `first`, `last`, `nth`,
   `find`, `filter_map`, `unwrap_or_default`.
2. Ask each hit two questions: **can the input hold more than one?** and **does the reader's claim cover the
   extras?** Both yes, and it is one of the shapes above.
3. Most hits die on the first question — splitting a path into segments, normalising a name — so the funnel is
   steep and the reading cost is a pass, not an audit. Nothing is reported automatically, so the
   false-positive cost is zero.

It is written down rather than remembered because of what it caught. The defects it found in the
release-readiness gate — a manifest read that took the first `name` key in any table, and a value parser that
reported *cannot read* as *not present* to three consumers — sat in the largest unread logic module and had
been read past by five linear review rounds. The sweep found both in one pass, and one of them let the gate
report a clean release while a crate's lock version went unchecked.

**Only the fourth shape is residue.** The first three are closed by construction where they occur, so filing
them would register as unclosable something that is not. The fourth is filed in `BACKLOG.md` with its
observation source inherited rather than invented: `inline-symbol-path-confinement` already declares that a
**receiver-method read is not observed** (no type inference on the receiver), and `text.split_once(…)`,
`iter.next()` and `vec.first()` are all receiver-method calls — so this repository's own dimensions cannot see
the shape, by a bound that already has an owner and a pinning test. The vocabulary above binds only the call
sites that use it, and nothing enumerates the readers that should; that residue belongs to the same entry.

## A census is produced, never typed

A figure saying **how many members a set in this repository currently has** is produced by whatever enumerates
that set — printed by its repository check on a clean run, or *computed* into a generated, staleness-checked projection.
It is never typed into prose, a doc comment, or a projection's template. Where nothing enumerates the set, anchor
the figure to a past moment so a reader cannot mistake it for current ("measured before proposing", "at `v0.4.0`"),
or drop it: the claim almost never needs the number.

This is not a style preference. Hand-written figures drifted **repeatedly in one release window, in every kind of
place they can live**: a doc comment saying fifty-three declarations against a register holding fifty-four; a backlog entry citing
fifty-five; a changelog sentence citing fifty-four with no time anchor; "eight files under `src/runner/`" in three
files at once; "all five gate matrices" after a sixth gate arrived; and a version-horizon paragraph — the one that
assigns the release number — whose measured commit count and "nothing else is packaged" claim the window itself
falsified.

**A figure inside a generated document is only safe if it is computed.** A literal in the template is the one place
a projection cannot self-correct: the freshness check compares the generator's own text with itself, so
the retired gate-shape capability's projection typed both a figure and a list of bounds and silently omitted one added in the
same window. Where a projection discloses a set, derive the membership from the source of truth and hold it in
**both** directions; the figure is then `len()`.

**Do not add a detector over prose.** It was designed and measured three times, and it is the wrong instrument:
widening the recognized phrasing false-positives on the generated projections' own headers, on a gate's diagnostic
string, and on six expected-output literals in its failure matrix; widening the corpus to `scripts/` false-positives
on the fixture censuses the register's own failure matrix writes deliberately; and the one instance that occurred in a **code
doc** was spelled in words, which no digit-based matcher reads. Most numbers in this repository describe a *shape*
("two files of one module yield one violation"), not a census, so a matcher over numbers is mostly false positives.
The census direction in `crates/kanhe/tests/bound_register.rs` stays what it is — armed for the one set whose
phrasing is stable, with nothing in the tree stating it — and the rule below is what keeps a figure honest.


**A hand-written count of a live set is not written down.** The declared-census mechanism is retained and
armed — `crates/kanhe/src/census.rs` enumerates a set, names the one sentence its figures may be written in,
and one sweep holds every tracked document to it. The one sentence the tree carries is in a **generated
projection**, whose figure a renderer computes over a freshness-checked document — the produced form this rule
points at rather than an exception to it. What the rule is about is the **typing**: a figure a person
maintains drifts, and one a renderer recomputes cannot. An earlier wording here said *produced or not*, which
forbade the very thing the next sentence recommends and contradicted that projection two paragraphs down. A declared census is the *safe* way to write a count, and safe was never the same as worth
having: it still has to be re-typed every time the set moves, and this repository re-typed the one it had three
times in a single release window, each time by hand and once with the wrong figure. Where a figure matters,
**the reaction prints it on every clean run** — `bound_register` and `pin_bites` both do — and the reader gets
it from the run rather than from prose that can disagree with the run. **The mechanism stays for a reason a grep can check, not because an armed reaction is nice to have.** That
was the first answer given here and it was the weaker one: an armed reaction with no subject is exactly the
narrow-instrument shape this repository withdraws. What earns `census` its place is that it has become the
crate's **cited exemplar** — `refusal`, `whitespace_hygiene` and `repository-checks` all point at
`census::sweep` as the reference implementation of a vacuity guard, *an unread document is not a document
without one* — and it is an entry in `gate_exit_classes`'s declared set of targets that spawn a process.
Deleting it would leave those sites naming a deleted thing, or abstract descriptions where a reader could
previously go and read working code. Its directions are held against written fixtures rather than against a
count in a live document, which is what makes them independent of the rule above.

**And it is not subjectless, which printing the state is how anyone found out.** The claim that the tree
states no census was made twice in this window and was wrong both times, because the sweep that produced it
looked for one declared phrasing and skipped the record locations. The sweep now says on every
clean run how many documents state one, and the answer is a **generated projection** whose figure the renderer
computes. That is the one place a figure belongs — produced, not typed — and this reaction is what makes
*produced* checkable rather than asserted. A reaction that reacts to nothing is silent for the same reason a
broken one is; saying the number out loud is what tells the two apart.

**A count of something this repository does not produce is not written either.** That is the other half, and
nothing observes it. Eight figures were found wrong in one change and three of them counted sets no repository check
enumerates — how many spans of a shape a document carries, how many commits a window holds. Each was
decoration: the sentence it sat in said the same thing without it, right up until the number stopped being
true. Where the count matters, produce it and let the producing document carry it; where it does not, write
the property and leave the number out.

**A line count is the sharpest case of this and gets named rather than left to the general rule.** It counts
nothing the repository enumerates, it moves on every edit to its subject — a rename, a rewrap, a comment —
and it is almost always decoration by the test above: a sentence naming three files, their three lengths, and
the property they share says exactly what the same sentence says without the three lengths. Measured when
this was written: one live instance in a tracked document, correct at that moment and held by nothing — and
removing it took one clause out of a sentence that still means what it meant. The instance is not quoted
here with its figures, because an example carrying live numbers is the thing it warns against. **A live line count is not written.** A line count of something that is *gone* is a record
and is fine — `repository-checks`'s `1562 lines of it — a figure measured when that shell was deleted and
standing as a record of that moment, not a census` is the form: it says when it was taken and that the set
it counted no longer exists, so nothing can drift out from under it.

**No detector is proposed, and that is a decision rather than a gap.** Telling a live count from a record is
reading what a sentence means, which is the prose instrument this repository designed, measured three times
and rejected. The decidable alternative — refuse every line count in a tracked document — would refuse the
`1562` above, which is correct and useful. So this half stays a rule the reviewer holds, and the sweep that
finds its violations is a `git grep` run deliberately, not a reaction that runs itself.

Two shapes are outside a census by construction, and stating them is the point. A figure in a **record** — a
commit message, a dated changelog section, `docs/history/` — is a measurement of the moment it was taken and
stands as one; only a live document owes a produced figure. The rule reaches commit messages otherwise, and
the commit that added it broke it in its own message within the hour. A figure about a **past state** — what a
document said before a change — is a record for the same reason: holding it to today's enumeration would demand
that the record change every time the tree does. And a figure written in a sentence no census declares is
unheld; the declaration is the coverage, and widening the match to prose instead is the detector this
repository designed, measured three times and rejected.

**One decision, made when the figure is written.** What a census is, where one may live, and which carriers
stand outside are each settled by this section; this states the whole of it as a single choice, because the
parts were each correct and sat apart, and a rule that cannot be applied in one read is applied by nobody.
Every figure in a live document is in exactly one of these states, and reaching one is the author's act rather
than a reviewer's discovery:

1. **Declared** — something here enumerates the set, so the figure is *produced* by that enumerator and the
   sentence carrying it is declared in `crates/kanhe/tests/census.rs`. The only state that cannot drift, and
   therefore the one to reach for first.
2. **Anchored** — nothing enumerates the set, so the figure is bound to the moment it was taken, and the
   stronger form is worth a clause. **Reconstructible:** name the commit and the command or perturbation, in
   the form `BACKLOG.md`'s promotion measurement uses — *measured at `<sha>`, by `git grep` over `<paths>`* —
   and any reader can re-run it. **A moment alone** — *measured when this was written*, *at `v0.4.0`*, *when
   that shell was deleted* — cannot be re-run, and is the floor rather than a lapse: it is all a figure about
   a vanished subject can have, and `repository-checks`'s line count of the deleted shell library is the
   worked example. Reach for the first wherever the producer still exists. Either way the figure is a fact
   about a past moment rather than the state of the session that wrote it. A **judgement** over the measured
   set — *most of them were X* — is anchored by neither form, however exactly its subject is named; where one
   carries a decision it becomes cases rather than an adjective.
3. **Dropped** — the sentence says the same thing without it. Most figures belong here, and the test is to
   read the sentence with the number gone.

**At review the disposal is those same three, and it is not a discussion.** A figure found in none of them is
moved into one: ask whether an enumerator exists, then whether the sentence needs the number at all. This has
no reaction, for the reason *Do not add a detector over prose* gives, so the sweep stays a reviewer's — and
what it then costs is a clause, in place of an argument about which rule applies.

## A repair loop is a diagnosis, not a schedule

**When a round of repairs produces its own findings, count what kind they are before deciding what to do
next.** Sort the round's findings into three: the code was wrong; *one rule has more than one implementation*;
or *a claim about the code was wrong while the code was right*. The third dominating is the signal, and the
signal is not "review harder" — it is that the property is stated where nothing can falsify it. Add a round and
the next round finds the next sentence; change the shape and the class ends.

**A governance rule measured as un-reacted is given a reaction or filed, in the same change.** Finding that a
prose rule has no backstop and then leaving it as prose is how the same rule keeps costing: the measurement is
the expensive part and it is already done. Two rules in this repository were measured un-reacted in one window
and only one of them was answered — `.github/CODEOWNERS`'s *a merge cannot relax the law without a human
accepting it*, measured against `main`'s protection as `require_code_owner_reviews: false`. What got a reaction
there is its **naming** half: a structural amendment must now produce a second explicit artifact. The
acceptance half is a judgement boundary, recorded as one, because a single-steward repository has no
mechanical second party. The reason-perimeter falsifier, measured over four rounds and eight corrections, got
neither at the moment the two were compared, and was noticed only because a review put them side by side —
`BACKLOG.md` carries it as a `WATCH` from the same commit that wrote this sentence, so what the example
records is the comparison rather than a class still open. Filing counts as answering — a class with a measured
cost and a recorded trigger is not the same as one nobody has priced — but silence does not. Say which of the
two the change is doing.

Measured, in the window that produced this rule: three consecutive repair rounds on one text reader, and across
all three **not one finding was a new code defect**. Every one was a sentence describing what the reader does —
"the line start refuses a mention", "the two cannot diverge", "three inputs decline", a declared bound's WHEN
and THEN — or a rule implemented twice. Each repair corrected the sentence review had named and wrote the next
one. The reader's behaviour was fine throughout.

Two moves end those two classes, and neither is a review:

- **A claim about what a repository check does becomes an executable case, never a comment.** Enumerate the shapes the
  check decides and assert the decision for each, so the description *is* the run. A declared bound's WHEN
  and THEN are then read off that table rather than typed beside it, and a reviewer's perturbation lands as a
  row instead of a finding.
- **One rule gets one implementation returning a typed result, and its consumers match exhaustively.** Where
  two callers each re-derive the rule they agree by maintenance; where they match on one function's return they
  agree by construction, and a new case forces every consumer to answer it or the build fails. A doc comment
  enumerating the outcomes is then a census of a set the type already holds — see *A census is produced, never
  typed*.

**This rule has no repository check, and that is stated rather than left to be discovered.** Deciding that a comment
describes something a run could falsify is a judgement over prose, which this repository has designed and
measured three times and rejected. What it is instead is the question to ask when a repair round comes back
non-empty — and the second move above, once applied, is enforced by the compiler rather than by this paragraph.

Do not read it as licence to skip the round. The round is what tells you which class you are in.

## Adversarial review stance

Work is gated by adversarial review, not performed agreement. At **propose**, challenge the design
before it is accepted: does it earn its weight against the drift law and minimalism; does it push
`xuanji` or a dimension past measure-only, or breach 三儀 ⊥ 三儀; is it a name without a reaction?
At **apply**, challenge the implementation: does the declared reaction still *bite* the boundary the
prose claims, or has the code drifted so the law passes without protecting its reason? Prefer an
independent reviewer, and verify each finding against the code before acting on it; reject or
redesign a change rather than let it pass diluted (the no-weakening-to-pass rule itself is
*Self-governance*, below). (`propose` / `apply` here are the OpenSpec phases above.)

**A guard is not a guard until it has been seen to fail.** Every new test that claims to protect a
change must be run against the code *without* that change, and the observed failure recorded in the
PR's `## Verification`. A test written from the same understanding as the fix inherits its blind
spots, so passing afterwards proves nothing on its own; only the negative run distinguishes a guard
from a restatement.

The trap this exists for is the change whose outcome is unaltered. When a fix improves a
**diagnostic** while the exit code, return value, or wire output stays identical, a test bound to
that outcome passes equally before and after — it pins the surrounding contract, not the change.
Choose the observation level the change actually moved (stderr text, the emitted document, a
syscall sequence), and where a test genuinely cannot reach it, say so in the PR and state what
evidence stands in its place instead of leaving the reader to assume a green suite covered it. A
test kept for the contract rather than the change earns a comment saying which it is.

**A sweep's completeness claim names the corpus it swept, and excludes the record describing it.** *"No `X`
remains in the tree"* cannot be true of a change that documents what it removed: the entry explaining the
retirement has to name the retired thing. Measured — that exact sentence was written after a `grep` returned
nothing and was false by the time the same commit landed, because its own `CHANGELOG` entry quotes the term.
Say which corpus was swept.

**A negative run's record is pasted from the run's output, never written from the intention.** Composing it
from what the change was *going to* do reads identically and is not evidence: measured in the 0.5.0 window,
one `## Verification` block was written that way and was wrong in three ways at once — the order, the figure
it cited, and the progression it narrated — while every other negative run in the same window was pasted
verbatim and every one of those is accurate. Nothing can react to this: a check can require a `## Verification`
block's *shape*, never that it is true of a run. What would close it is the runner emitting its own transcript
to a path the record cites, so the evidence is generated rather than restated.

**A mechanical rewrite over code matches an exact literal and asserts it occurred once.** Rewriting ~44 call
sites here with a non-greedy pattern and the dot-matches-newline flag, the match crossed a statement boundary
and turned an unrelated `assert_eq!(outcome.exit_code(), 0, "…")` into `matches!(outcome.exit_code(), 0, …)`.
That one broke brace balance, so the compiler reported it — a corruption that still compiled would have been
silent, and nothing in the suite was looking for it. Reverting the two damaged files and redoing them
literal-by-literal was faster than repairing the pattern's output.

So: no pattern that can span a statement, an exact substring for each edit, and a count assertion before
writing. Widening a variant is its own case of this — `assert_eq!(x, Expected)` becomes
`assert!(matches!(x, Expected(_)))`, never a fabricated expected value, because pinning one asserts more than
the test meant.

**A negative run proves nothing unless it moved only the thing under test.** Three ways one reads as
decisive and is not, each measured in this repository rather than imagined:

- **Observed at the wrong grain.** Three sweeps were perturbed by making a tracked file unreadable, and two
  of the three appeared to refuse *before* their fix — a **sibling** direction in the same test binary was
  failing. Name the direction (`-- --exact <fn>`), never the target, or the binary's colour is attributed to
  whichever direction you had in mind.
- **The subject was outside the reader's own corpus.** The same perturbation said nothing about a sweep whose
  corpus is line-comment formats by construction: the Markdown file chosen was excluded before any of its
  logic ran, so it passed for a reason unrelated to the change. The subject has to be something that reader
  would actually have read.
- **Both sides were perturbed.** Renaming a cited function together with its citation leaves the two
  matching, so the run is green for exactly the reason it was green before. Move one side only.

Each of these produces a green or a red that *feels* like evidence. The question that separates them is not
"did it change colour" but **"what else could have produced that colour"** — and answering it costs one
command, against a repair that would otherwise ship believing it was measured.

A vocabulary- or identity-level breaking change additionally requires grepping every touched spec
and doc for the retired term across its *whole* file, not only the new diff: sync bolting on a
correctly-worded requirement while the same file's older prose still names the retired shape is
itself an undetected drift, invisible to a diff-only read (the 0.3.0 `finding_key` lesson).

**A sweep is keyed to the claim, not to the phrasing it last saw — and its subject is any hand-maintained assertion about a live set, not only a wording.** Each correction of one inverted statement
of the Core Contract left instances behind, because each sweep searched for the wording in front of it: a
line-oriented one could not see an instance wrapped across two comment lines; joining lines fixed that and
kept the phrase, so another spelling of the same claim survived; and a review proposing the repair named a
pattern narrower than its own finding. The kinds are what generalise — a wrapped instance, an excluded
corpus, a variant phrasing — and `CHANGELOG.md` carries the window's own count of them. What finds all of them is a
line-joined search for the **claim** — here `forbid\w*.{0,40}more strictly` — over tracked content, with the
document doing the reporting inside its own corpus. **Record the pattern beside the correction**, or the next
round re-derives a narrower one: when a second run finds new instances of a phrasing variant, the pattern is
the subject rather than the sites.

**A count of a set the code owns is the same defect wearing a number.** An enum's doc saying *four states* is
a hand-maintained assertion about a live set exactly as a phrasing is, and it goes stale in the commit that
grows the enum — measured here more than once, including in `Declared`, forty lines from the
`Package` doc rewritten to remove that very shape in the same commit. The repair is the one `xuanji::bound::Reached` already states: each variant
documents the **distinction** that earns it a place, and the compiler enumerates them. Sweep for the shape,
not for the site in hand.

**The decidable subject is a type's own cardinality, not every number in prose.** Widened to *any figure about
a set*, the sweep returns about a hundred hits across this tree and nearly all are reasoning — *two forms
cannot bind a value*, *three answers, because `.is_ok()` gave two* — which name distinctions rather than count
a live set, and rewriting them would be the over-wide sweep this file warns about elsewhere. What drifts is a
doc stating how many variants **the type it documents** has, in the commit that grows that type. That is the
pattern to run; the hundred are the reason to keep it narrow.

Once a claim like that is corrected, its pattern does not return zero and should not: what it finds afterwards
is **the pattern's own occurrences and the marked quotations** — a dated entry recording what the wrong wording
said, in italics or backticks, beside the right one. The finding is an *assertion*, never a quotation, so a
sweep whose hits are all quoted is the finished state rather than an unfinished one.

**Retiring a capability requires the same sweep, not only renaming one.** A rename changes the word a
grep looks for; a retirement removes the subject a `CHANGELOG.md`/`BACKLOG.md` entry was narrating, and
an entry that still describes the retired mechanism in the present tense is exactly as stale as one
naming a retired term — the sweep above did not name this case because nothing had measured it yet.
Measured in the 0.4.0..0.5.0 window: two capability retirements inside the same review window
(the shell-to-Rust migration, the `gate-shape-contract` retirement) each landed cleanly
against the code they touched, but left `CHANGELOG.md`/`BACKLOG.md` entries elsewhere in the *same*
window narrating the since-deleted mechanism as current — nine such entries, found only by an
adversarial contract review reading every entry against `HEAD` rather than against the commit that
originally closed it. The retiring commit itself is not the place this is caught: it correctly
describes what it just did, and has no reason to know which earlier entry in the same unreleased
window it just orphaned. Before closing a change that deletes a capability, test/module, or mechanism,
grep **every tracked live document** for its name and for the commits that built it, and annotate every
hit that still narrates it as live — in `CHANGELOG.md`'s own established idiom (a follow-up sentence or
`### Self-governance` entry noting the retirement, as at the "third floor... is retired" example
above), in `BACKLOG.md`'s (a trailing note on the entry, not a rewrite of it). This has no repository
check: these are prose, so per *Bind a claim to its measurement*, a reaction is not the available
option — the sweep is stated here as the discipline in its place.

**A retired *observation bound* needs one more seed than its own name: the test that pinned it.** A bound is
retired wherever it is written down — its `BoundDecl`, its spec scenario, the `BACKLOG.md` entry tracking
it, the projections rendered from those — and `observation_bound_model.rs` holds the first two in a bijection, so a stale *id* cannot
survive. What survives is prose that **describes** the bound without naming its id: a sibling test's doc
comment saying *the bound that remains is …*, or a paragraph explaining why some neighbouring case is left
alone. No bijection can see those, because there is nothing in them to resolve. Measured in the 0.5.0 window:
a retirement swept the declaration, the scenario, the backlog and both projections, and left two doc comments
in the very file the retiring commit had edited — one of them directly above a direction asserting the
opposite. So the sweep seeds are the bound's id **and** the name of the direction that pinned it, run over the
whole tree including `tests/`: prose that describes a bound tends to sit next to the test that held it.

**The corpus is every live document, and naming two of them was this rule's own instance of the class it
closes.** It said `CHANGELOG.md` and `BACKLOG.md`, which are where a *narration* usually sits — and
`PROJECT.md`'s Decisions section carried `gate-shape-contract` in the present indicative for the whole window,
outside a sweep written for exactly that. A corpus narrower than the claim, in the paragraph about corpora
narrower than their claims. The record carriers stand outside it by construction, and they are the same three
this file already names — a commit message, a dated `CHANGELOG.md` section, `docs/history/`: each is a
measurement of the moment it was taken, so holding one to `HEAD` would demand that the record change every
time the tree does.

**An extraction's corpus is the pair of modules, not the function you came for.** Two extractions in
`crates/kanhe` each closed a twin and each left a sibling behind, and both say so in their own headers:
`hermetic_git`'s names a command builder that "lived twice, byte-identical, in `publish_source_gate` and
`release_coherence_gate`", and `manifest`'s names "two more twins left behind in that extraction" — the same
pair of files, twice. A later review found four more instances of the class in one sweep: a fixture `run()`,
a `WorkspaceVersion` consumption, and two tokens with a constant owner and a literal copy elsewhere
(`TIANHENG_WORKSPACE_TESTS`, `Do not edit by hand`). Converging the first of those then exposed a fifth, an
`add`-then-`commit` helper one module had written and its sibling had not.

So when an extraction is made, the thing to read is **everything those two modules share**, not the one
function that prompted it. Do it before the extraction lands, because the moment it does, the remainder
stops looking like a class and starts looking like ordinary code.

This has **no repository check**, and the reason is measured rather than assumed. A reaction was built for
it and deleted in the same change: over `crates/{kanhe,shengmo}/src` a window of four executed lines
carrying at least two executed statements reports the two structural twins and nothing else — but it sees
neither of the token-level instances, it reports a call's arguments as statements wherever rustfmt wrapped
one, and every tightening measured against that false positive also removed a true one. Worse, the
convergence it asks for is refused elsewhere: `refusal_register` registers a site by the string literal
opening the constructor's argument list, so two gates sharing a refusal arm **cannot** converge it without
either losing each gate's own per-capability identity or producing constructions the register cannot parse. An
instrument covering one instance of four, needing three declared bounds, and asking for a repair another
gate forbids is not the available option. The sweep is.

## Commits & PRs

- **Conventional Commits.** Every non-release subject is
  `<type>(<scope>)!?: <imperative summary>` using a lowercase type and, when present, a lowercase
  package or workflow scope. Use the narrowest honest type: `feat`, `fix`, `refactor`, `docs`,
  `test`, `build`, `ci`, `perf`, or `chore`. Append `!` for a breaking change and name the migration
  in a `BREAKING CHANGE:` footer. Do not use lifecycle phases, branch roles, issue numbers, or a
  vague `update` as the type.
- **Bodies carry provenance.** Except for the release snapshot below, every commit has a concise
  body that explains why the change exists and what contract, reaction, or repository check it preserves. Separate it
  from the subject with one blank line; do not merely repeat the diff or rely on a PR number.
- **PR title and body are merge inputs.** A PR title is the exact Conventional Commit subject
  intended for the squash commit. Its body uses `## Why`, `## What changed`,
  `## Adversarial review`, `## Verification`, and `## Compatibility`; the last section states the
  public/migration effect and whether manifests or package versions changed. Verification names the
  commands and external consumers actually checked — never an unqualified "tests pass" — and, for
  each new guard, the failure observed without the change (see *Adversarial review stance* above).
- **Curated squash message, through `scripts/merge-pr.sh`.** For a development PR into a release branch, set
  the squash subject exactly to the PR title with no auto-appended `(#N)`. That is the sanctioned path: the
  wrapper reads the PR title, runs `crates/kanhe/tests/merge_message.rs` over the proposed message, and
  only then reaches `gh pr merge --squash`. This rule was stated here and missed anyway — nine subjects, counted when the gate below was written, in
  this repository's history carry the serial — and a merged squash cannot be repaired, because amending it
  changes the hash the pull request's merge record cites. Replace GitHub's concatenated commit list
  with a self-contained body distilled from the PR's why, verification evidence, and compatibility result;
  retain any `BREAKING CHANGE:` footer. The branch's fine-grained commits remain review provenance,
  not the release branch's message body.
- **No AI/agent attribution.** Commit messages and PR descriptions must NOT contain a
  `Co-Authored-By: Claude` trailer, a "Generated with Claude Code" footer, a "🤖" line, or
  any other tool-authorship mark. The history records *what changed and why*, not what
  typed it. This is a project rule, not a personal preference.
- **Self-describing style.** A message says what changed and why, in its own words — not
  an issue/PR number as a crutch. A reader should understand the change from the message
  alone.

## Branching and release

`main` is release-only: it carries nothing but linear, non-merge `release: X.Y.Z` snapshot
commits, each tagged `vX.Y.Z`. The fine-grained development commits never land on `main` individually —
they collapse through two squash stages on the way up: a development branch is squash-merged into
`release/X.Y.Z`, and that release branch is squash-merged into `main`.

Branch names encode role and intent. One role is fixed — `release/X.Y.Z`, the first squash target — and all
other work uses `<type>/<scope>-<slug>`, where `<type>` is the *Conventional Commit type the work will land
as* (the same set *Commits & PRs* above admits — `fix`, `test`, `refactor`, `docs`, `feat`, `ci`, and so
on), so a branch's role and its squash subject cannot disagree. Deriving the role from the commit type is
deliberate: an enumerated list of blessed prefixes drifts from what the repository does, and a
governance rule that has drifted is read as license rather than law. Pre-release polish therefore
takes the type its own work lands as; there is no separate release-staging role, because a branch's
role is what it does, not when it happens. Slugs are lowercase kebab-case, describe the outcome
without an issue number, and never use a placeholder such as `spike` after intent is known. `main`
takes no direct work — it is release-only.

Both squashes are performed by a GitHub pull request's "Squash and merge", not a local merge. The
release-branch-to-`main` squash is the sole message exception: its subject is `release: X.Y.Z` and
its body is deliberately empty. A release snapshot's change is the whole tree; per-change why lives
in the curated commits and PRs below it. A PR that touches a steward-owned path
(`.github/CODEOWNERS`) is merged by the steward. A release branch is archived once it merges; it
carries no further work and is never a source of record for anything downstream.

**`main` is also the only publish source.** After the release squash and the signed `vX.Y.Z` tag, the
crates.io publish runs from a checkout of *that tagged `main` commit* — never from the release
branch. `cargo publish` stamps the sha1 of whatever `HEAD` it ran on into every tarball's
`.cargo_vcs_info.json`, and a published version can never be re-uploaded, so the pointer is permanent
from the moment it lands. An identical tree does not make a release branch's tip an acceptable
source: cargo records the **commit**, not the content, and the commit it would record belongs to a
branch the ritual archives. `bash scripts/publish.sh` is that path — it runs
`crates/kanhe/tests/publish_source.rs` (worktree clean; `HEAD` the `release: X.Y.Z` snapshot for the
workspace version; `vX.Y.Z` annotated, signed, and pointing at it; `HEAD` the live tip of
`origin/main`, read from the remote rather than a possibly-stale `refs/remotes/`) and only then
`cargo publish --workspace`. The gate is a `cargo test`, so it distinguishes a **violation** — the source
disagrees — from a **cannot-judge** in its own result type rather than in a process status; either stops the
wrapper before `cargo publish` is reached. The
wrapper forwards **only the arguments it names**, refusing everything else — including a spelling of a
named flag and a flag a future cargo adds — before the gate runs. Both wrappers work this way, by one
question: does the argument move what the gate judged, or what the act records? **The allowlist itself is the
wrapper's parser, which is where it is read from and the only place it is complete**; the arms below are
examples of each side, not the set. So `--manifest-path`, `--exclude`, `--no-verify`, `--allow-dirty` and
`--config` are among those refused, while `--dry-run`, `--package`, `--locked` and the destination-side
`--registry` and `--index` are among those forwarded — the last two changing where the result goes rather than
what it is. Naming the parser rather than restating it is deliberate: a second list here would have to agree
with the first, which is the shape an allowlist exists to avoid, and it had already fallen behind. Values go in
the argument after the flag; one spelling each.

**A published release snapshot is immutable.** Once a version is on crates.io, its `release: X.Y.Z`
commit must never be amended or force-pushed away: the published artifact points at that sha1
permanently, so replacing it orphans the pointer just as surely as publishing from the wrong branch
does. `0.2.2` was published from `main` correctly and then force-pushed away an hour later, which the
publish-source gate cannot foresee — at publish time it would have passed — so this half stays a
convention. What each published version actually records, and the two mechanisms that produced the
disagreements, is inventoried in
[`docs/history/published-artifact-provenance.md`](docs/history/published-artifact-provenance.md).

`TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test release_coherence` is the release-state check. During development it
requires an adopter-facing `[Unreleased]` entry and aligned workspace/internal dependency versions,
but deliberately tolerates historical lockfile drift. Once the workspace version moves forward for
release preparation—and at the exact `release: X.Y.Z` snapshot—the dated CHANGELOG section,
internal pins, and every workspace package entry in `Cargo.lock` must all name that version. The
check is read-only and needs full git history; it never bumps, commits, tags, or publishes.

**Setting the dated section's date is the last edit before the cut, not a step during preparation.** At the
snapshot the check holds that date against the `release: X.Y.Z` commit's own, so a date written days earlier
fails there — and it has: one release was prepared with a date four days behind the day it would be cut on,
and nothing said so until the check was given that comparison. During preparation the date is an intent
rather than a claim, so preparation may leave it stale for as long as it lasts; what must not happen is
cutting without touching it. Write it on the day the `release: X.Y.Z` commit is made, immediately before
making it.

A branching pattern is not an observable architectural fact, so the drift law keeps it out of the
constitution: it is a convention for humans and agents rather than a Tianheng boundary.

**That reason reaches the constitution and stops there, and this sentence used to carry it one word
further.** It read *not a Tianheng boundary or repository check*, comparing itself to the
self-describing-commit rule above — and the two halves are different things this file distinguishes in so
many words: `crates/kanhe` holds hand-written `cargo test` gates that govern this repository without being
the product running on itself, so *a claim about one is not a claim about the other*. The comparison had
also gone stale. `merge_message_gate` refuses a subject carrying `(#N)` and a body that is a bare list of
commit subjects — the two decidable halves of *not an issue/PR number as a crutch* and *understand the
change from the message alone* — so that rule has a repository check where its squash instance is
concerned, while a branch name has none anywhere. A premise a file's own new code has falsified, left where
an operator reads it first, is the shape the merge wrapper's header already records paying for.

Whether a branch name should acquire one is filed in `BACKLOG.md` rather than answered here.

## Self-governance — don't weaken the law to make CI pass

**Self-governance is Tianheng governing itself with the capability it ships.** `crates/shengmo/src/law.rs` declares a real constitution through the published surface an adopter uses, and `crates/shengmo/tests/self_governance.rs` runs it against this workspace as a `cargo test` gate. Its live invariants are declared in Rust and projected into [`AGENTS.self-law.md`](AGENTS.self-law.md); do not hand-maintain a second list here.

Beside it sit this repository's other checks — hand-written `cargo test` gates over its changelog, specs, scripts, and documents. They govern the repository too, and they are held to the same standard, but they are **not** the product running on itself: a claim about one is not a claim about the other. An earlier version of this sentence said every Rust integration test ran Tianheng's own reactions against the workspace, which was false for 20 of the 25 then present — none of them reached the shipped API at all.

**Projections are text views, not reactions or checks**: Contract projections and censuses (such as [`AGENTS.self-law.md`](AGENTS.self-law.md), [`docs/observation-bounds.md`](docs/observation-bounds.md), the retired gate-shape projection, [`docs/observation-bound-extents.md`](docs/observation-bound-extents.md), [`docs/projection-register.md`](docs/projection-register.md), and [`docs/refusal-register.md`](docs/refusal-register.md)) are derived text views. They are NOT reactions, NOT checks, NOT governance, and NOT shipped product code. Their freshness is asserted by Rust `cargo test` gates ("*A census is produced, never typed*").

If a change makes a self-governance test fail, **fix the change**, not the test. A boundary is altered only by a deliberate, human-reviewed amendment — never by quietly weakening it so CI turns green.

## Definition of Done

Run these from the workspace root before checking off an apply task, syncing, or reporting a change done. **The phrase names this list.** Writing *"Definition of Done all green"* over a
subset is a completeness claim about a set someone can count. Run the whole list and say so, or name the
commands that ran and do not use the phrase. The rule against a sweep claiming more than its corpus is the
same rule; this list is a corpus with a name, and the record of how often that was got wrong belongs in the
`CHANGELOG` entry that measured it, not here. This is the single source for the local pre-flight gate list (so other docs need not restate it); CI runs a superset of it:

**`git add` any file the change CREATED before running these.** The tree-wide gates take their path list from
`git ls-files` and their content from disk, so the two halves see different things — measured, both
directions: a **new** file that has not been added is invisible and its offences are not reported, while a
**tracked** file modified and left unstaged is read as it stands on disk and is judged normally. Staging is
enough; committing is not required.

The failure mode is a full pass that means less than it looks. Measured in the 0.4.0 window: one change's
Definition of Done ran green over a file it had never opened, and the next round's suite failed on that same
file, unchanged in between — the
per-crate steps compile the worktree and do see new files, so most of the suite is green for real and only
the repository-wide gates are blind. A partial blindness is far more convincing than a total one.

This has no repository check, and the reason is worth stating rather than leaving to be rediscovered: in CI
nothing is ever untracked, so a check would be vacuous exactly where it runs, and locally it would fire on
every scratch file mid-edit — a refusal that is right at one moment of the day and wrong the rest of it.

**The list assumes host tools it does not install, and only one of them refuses when absent.** `cargo`,
`git`, `npm` with a Node in `package.json`'s `engines` range, `cargo-deny`, `jq` and `gh` must be on the path.
`.npmrc`'s `engine-strict` makes `npm ci` stop on the Node range rather than warn past it; nothing stops on
the others. This paragraph is the **documented** half and has no reaction behind it, which is said plainly
because a list of prerequisites reads like a guarantee.

What earns it a place is that a missing tool does not present as a missing tool. Measured 2026-08-23 with
`jq` absent: 15 of `merge_workflow`'s 30 cases failed, each reporting `bin/gh: line 77: jq: command not
found` alongside the merge message *cannot read what CI said about this pull request* — so the operator reads
fifteen findings about the subject when the state met was one absent binary. The wrapper was right and the
fixture was not: its `gh` stub pipes through a host tool it never declares. `BACKLOG.md` carries that
instance with the repair that has teeth — the fixture states what it needs and stops before the subject — and
its promotion trigger is a second fixture found doing the same. This half is the reminder; that half is the
reaction.

```bash
cargo build --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --workspace -- -D warnings   # shipped lib/bins only (no --all-targets, default features)
cargo clippy -p louke -- -D warnings       # louke's audit-OFF library on its own
cargo test -p louke                        # louke's audit-OFF library ON ITS OWN
cargo fmt --all --check
TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --document-private-items
cargo deny check
npm ci --ignore-scripts                    # the pinned validator, reproduced from the committed lock
npx --no-install openspec validate --specs --strict
TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test whitespace_hygiene
TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test reference_integrity
TIANHENG_EXAMPLES=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p shengmo --test examples_suite
TIANHENG_PIN_BITES=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test pin_bites   # a line of its own because it is env-gated: it checks out a
                                           # worktree and builds it, so the ordinary suite must not pay
                                           # for it — and leaving it to run only when someone remembers
                                           # would be the worse half of that trade
```

The self-governance dogfood gate (`crates/shengmo/tests/self_governance.rs`, which runs the product reaction under `cargo test`) and its projection
(`self_law_projection_is_fresh`) must stay green — never weaken the law to pass it. So must
`observation_bound_model.rs`, which holds every declared observation bound's spec scenario and its typed
classification in a bijection and projects `docs/observation-bound-extents.md`; it needs no line of its own
above because it runs under that same `cargo test`. And so must `observer_protocol.rs`, which holds the
trait-driven fold and the built-in composition path to one verdict — two paths that could disagree silently
are the drift a seam is supposed to end.


## Versioning — SemVer honesty (the modou lesson)

Version literals in prose name only an immutable historical/provenance fact, a migration target, or the active
release-planning surface. An `[Unreleased]` adopter narrative may therefore name its intended release; long-lived
comments and live documentation do not restate a "current" or prospective version — say `[Unreleased]`, workspace
version, manifest requirement, or this checkout instead. The release-coherence check owns only the mutable
version-bearing surfaces it enumerates; do not add a general prose-number detector.

- Pre-1.0 and at `0.0.x`: **no inter-release compatibility is promised**; any release may
  break. Do not vanity-bump the minor for a non-breaking change.
- Graduate to `0.1.0` only when the public API has settled enough to promise
  `0.1.x`-patch compatibility. After that: non-breaking → patch, breaking → minor.
- **Breaking means the adopter has to act**, which is wider than a moved API. A changed public
  signature, wire format, or identity shape breaks — and so does a change that leaves a recorded
  baseline no longer describing the adopter's tree, because regenerating it is work they did not
  choose. **Closing a false negative therefore earns a minor**, however small its diff: the reaction
  is additive, the baseline is not, and "the defect was ours" does not spare them the work. So does
  new depth that reacts by default. Patch-class is what an adopter can take without doing anything —
  packaging and hygiene, prose and specs, opt-in depth, performance, and a diagnostic whose exit code
  and emitted documents are unchanged. `CHANGELOG.md` states the same fact as its `**BREAKING**`
  marking rule, which is that projection for adopters; the version consequence is here, because a
  release number is decided before the notes are written.

## Drift law & minimalism (inherited, non-negotiable)

- **No drift type without an observation source; no target or name without a reaction** —
  at module, crate, and dimension granularity. Do not pre-create empty `semantic`/`runtime`
  crates or stub modules; a dimension's crate is born when it is built.
- **Fail loud only on observable misconfiguration.** No defensive over-foolproofing of
  impossible states. One decidable instance of this is observed: `str::split` and `str::rsplit`
  always yield at least one item, so a `.next()` on either is always `Some`, and
  `unreachable_branch`'s sweep refuses a consumer that reads it as if it could be absent. The
  rest of the bound is judgement and has no reaction — a fallback nothing reaches still tells a
  later reader that the case happens.

## Outward / irreversible actions — confirm first

Merging to `main`, tagging, publishing to crates.io, force-pushing, and deleting a repo
are confirm-first: get explicit human sign-off even if a permission rule would auto-allow
it. (crates.io publishes are permanent — only yankable, never deletable, and a version's recorded
source commit is permanent with it — so *where* the publish runs from is gated rather than
remembered; see *Branching and release*.) A local
`.claude/settings.local.json` `permissions.ask` rule on `gh pr merge` is a recommended way to
mirror this in a dev environment, but the confirm-first rule binds regardless of local settings.

Before publishing, confirm every publishable crate **bundles its license texts**: `cargo
publish` packages only files inside each crate's own directory, so the workspace-root
`LICENSE-*` and the inherited SPDX `license` field are not enough — each crate must physically
carry `LICENSE-MIT` and `LICENSE-APACHE`, or it ships without them (as 0.1.0/0.1.1 did, before
this was caught). `cargo package --list -p <crate>` shows exactly what a crate would ship. This
is release/packaging hygiene, not architectural drift, so it is a **CI check** (the
`License texts bundled` job), never a Tianheng constitution boundary — the same reason the
branching/release ritual above stays convention rather than a check.
