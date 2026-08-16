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
   human-reviewed amendment (an OpenSpec change / steward review), never by quietly editing
   the constitution so CI turns green. Before proposing an OpenSpec change, read the law
   projection (step 1) so the proposal reasons against the declared shape, not a guess.

This SOP is **orientation, not the binding mechanism**: the reaction (a failed `check`, a
runtime probe) is what binds: reading the law first does not *grant* compliance, it just
saves a round-trip. It is convention, not constitution — an observable architectural fact
belongs in the declared law and reacts; a working agreement like this one does not, so the
drift law keeps it here, not in `Constitution`.

## Writing a boundary's `reason` — for 潛移 (gravity)

A boundary's `because(...)` is read twice: once by a human, and — projected into an agent's
context by `list` — once by an autoregressive model that *imitates* it (see PROJECT.md, 潛移).

Governance follows a **Three-Layer Architecture**:
1. **Layer 1: Reaction Backstop (Code)** — Functional boundaries (`restrict_dependencies_to`, `must_not_call_inline`) enforce hard invariants in code. Minimalism forbids redundant reactions (do not add a denylist for a prohibition an allowlist already enforces).
2. **Layer 2: Qiányí Gravity Pull (Prose Reason & Projection)** — `because(...)` reasons project into `AGENTS.self-law.md` to condition LLM continuations. Write reasons strictly in a **forward voice** ("the kernel depends inward only"), never as a backward justification or historical debrief ("we once hit a cycle in 0.2.2"): **provenance belongs in `PROJECT.md` decisions and git history, not in the live context reason.**
3. **Layer 3: Provenance & History (Doc)** — Historical rationale, lessons learned, and decision context stay in `PROJECT.md` decisions and commit history, keeping live context dense and noise-free.

Keep every reason **within the boundary's observable perimeter** — a reason must never assert structure the law does not react to (that is prose prescription, an open loop with no backstop). Forward voice, bounded to what reacts, minimal in reactions.

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

## Working a capability change — OpenSpec in `specs` mode

**This project uses OpenSpec's `specs` half and not its `changes` half**, and that is a chosen mode
rather than a stage on the way to something. `openspec/specs/*` is the per-capability requirement
truth; there is no proposal workflow, no change directory, and no separate lifecycle vocabulary.
`PROJECT.md`'s Decisions records why, and what would change it.

So a capability change is one ordinary piece of work:

1. **Read** the capability's spec, and the law projection the SOP above names.
2. **Write the requirement onto its spec** — a new or amended `### Requirement:` and its
   `#### Scenario:`s, edited in place.
3. **Write the reaction that answers it**, in the same branch.
4. **Land both as one squash pull request**, whose subject names the product effect
   (`feat(xuanji)!: …`, `fix(hunyi): …`, `refactor(guibiao)!: …`) per *Commits & PRs* below.

The spec and the reaction move together **because they are one change**, not because a sync step
merges them later. A pull request that edits a requirement and ships no reaction for it is the thing
this arrangement exists to make visible.

**Every new or materially changed scenario carries its observation evidence in that same pull
request**: either name the existing reaction or repository check in the PR's `## Verification`, or
add a new guard and record the required negative run. If a property cannot fail because the data
model constructs it, state that construction in requirement prose rather than inventing a scenario.
A scenario with neither form of evidence does not belong in a spec.

`openspec/changes/` stays present, empty but for `archive/.gitkeep`. **Its job is optionality**: the
directory exists so adopting the `changes` half later needs no setup and no exception, and the one
placeholder is what keeps `openspec/changes/` itself tracked. It is not evidence that the workflow is
in use — nothing has ever been written there. Nothing enforces the mode either, deliberately: a check
that failed when a change directory appeared would prevent the adoption the placeholder is held open
for.

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
The census direction in `crates/kanhe/tests/bound_register.rs` stays what it is — a backstop for the one set whose phrasing is
stable — and the rule above is what keeps a figure honest.


A census this repository can produce is **declared**: the repository check that enumerates the set names the one
sentence its figures are written in, and one sweep holds every tracked document to it — `crates/kanhe/tests/census.rs`.
Adding a census means declaring it, which is what makes it enumerable.

**A count of something this repository does not produce is not written.** That is the other half, and nothing
observes it. Eight figures were found wrong in one change and three of them counted sets no repository check
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

## A repair loop is a diagnosis, not a schedule

**When a round of repairs produces its own findings, count what kind they are before deciding what to do
next.** Sort the round's findings into three: the code was wrong; *one rule has more than one implementation*;
or *a claim about the code was wrong while the code was right*. The third dominating is the signal, and the
signal is not "review harder" — it is that the property is stated where nothing can falsify it. Add a round and
the next round finds the next sentence; change the shape and the class ends.

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

**Retiring a capability requires the same sweep, not only renaming one.** A rename changes the word a
grep looks for; a retirement removes the subject a `CHANGELOG.md`/`BACKLOG.md` entry was narrating, and
an entry that still describes the retired mechanism in the present tense is exactly as stale as one
naming a retired term — the sweep above did not name this case because nothing had measured it yet.
Measured in the 0.4.0..0.5.0 window: two capability retirements inside the same review window
(`64ed18c`'s shell-to-Rust migration, `5abda1f`'s `gate-shape-contract` retirement) each landed cleanly
against the code they touched, but left `CHANGELOG.md`/`BACKLOG.md` entries elsewhere in the *same*
window narrating the since-deleted mechanism as current — nine such entries, found only by an
adversarial contract review reading every entry against `HEAD` rather than against the commit that
originally closed it. The retiring commit itself is not the place this is caught: it correctly
describes what it just did, and has no reason to know which earlier entry in the same unreleased
window it just orphaned. Before closing a change that deletes a capability, test/module, or mechanism,
grep `CHANGELOG.md` and `BACKLOG.md` for its name and for the commits that built it, and annotate every
hit that still narrates it as live — in `CHANGELOG.md`'s own established idiom (a follow-up sentence or
`### Self-governance` entry noting the retirement, as at the "third floor... is retired" example
above), in `BACKLOG.md`'s (a trailing note on the entry, not a rewrite of it). This has no repository
check: both files are prose, so per *Bind a claim to its measurement*, a reaction is not the available
option — the sweep is stated here as the discipline in its place.

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
commits, each tagged `vX.Y.Z`. The fine-grained lifecycle commits (propose / apply / sync)
never land on `main` individually — they collapse through two squash stages on the
way up: a change branch is squash-merged into `release/X.Y.Z`, and that release branch is
squash-merged into `main`.

Branch names encode role and intent. Two roles are fixed: `change/<openspec-name>` exactly matches
an OpenSpec change directory, and `release/X.Y.Z` is the first squash target. All other work uses
`<type>/<scope>-<slug>`, where `<type>` is the *Conventional Commit type the work will land as* (the
same set *Commits & PRs* above admits — `fix`, `test`, `refactor`, `docs`, `feat`, `ci`, and so on),
so a branch's role and its squash subject cannot disagree. Deriving the role from the commit type is
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

Like the self-describing-commit rule above, this is a convention for humans and agents, not a
Tianheng boundary or repository check: a branching pattern is not an observable architectural fact, so the drift law
keeps it out of the constitution.

## Self-governance — don't weaken the law to make CI pass

**Self-governance is Tianheng governing itself with the capability it ships.** `crates/shengmo/src/law.rs` declares a real constitution through the published surface an adopter uses, and `crates/shengmo/tests/self_governance.rs` runs it against this workspace as a `cargo test` gate. Its live invariants are declared in Rust and projected into [`AGENTS.self-law.md`](AGENTS.self-law.md); do not hand-maintain a second list here.

Beside it sit this repository's other checks — hand-written `cargo test` gates over its changelog, specs, scripts, and documents. They govern the repository too, and they are held to the same standard, but they are **not** the product running on itself: a claim about one is not a claim about the other. An earlier version of this sentence said every Rust integration test ran Tianheng's own reactions against the workspace, which was false for 20 of the 25 then present — none of them reached the shipped API at all.

**Projections are text views, not reactions or checks**: Contract projections and censuses (such as [`AGENTS.self-law.md`](AGENTS.self-law.md), [`docs/observation-bounds.md`](docs/observation-bounds.md), the retired gate-shape projection, [`docs/observation-bound-extents.md`](docs/observation-bound-extents.md), and [`docs/projection-register.md`](docs/projection-register.md)) are derived text views. They are NOT reactions, NOT checks, NOT governance, and NOT shipped product code. Their freshness is asserted by Rust `cargo test` gates ("*A census is produced, never typed*").

If a change makes a self-governance test fail, **fix the change**, not the test. A boundary is altered only by a deliberate, human-reviewed amendment — never by quietly weakening it so CI turns green.

## Definition of Done

Run these from the workspace root before checking off an apply task, syncing, or reporting a change done. This is the single source for the local pre-flight gate list (so other docs need not restate it); CI runs a superset of it:

**`git add` any file the change CREATED before running these.** The tree-wide gates take their path list from
`git ls-files` and their content from disk, so the two halves see different things — measured, both
directions: a **new** file that has not been added is invisible and its offences are not reported, while a
**tracked** file modified and left unstaged is read as it stands on disk and is judged normally. Staging is
enough; committing is not required.

The failure mode is a full pass that means less than it looks. `#556`'s Definition of Done ran green over a
file it had never opened, and the next round's suite failed on that same file, unchanged in between — the
per-crate steps compile the worktree and do see new files, so most of the suite is green for real and only
the repository-wide gates are blind. A partial blindness is far more convincing than a total one.

This has no repository check, and the reason is worth stating rather than leaving to be rediscovered: in CI
nothing is ever untracked, so a check would be vacuous exactly where it runs, and locally it would fire on
every scratch file mid-edit — a refusal that is right at one moment of the day and wrong the rest of it.

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
npm ci                                     # the pinned validator, reproduced from the committed lock
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
  impossible states.

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
