# self-law-projection Specification

## Purpose

Put Tianheng's **own enforced self-law** into an agent's context as a faithful, imitable,
staleness-checked Markdown artifact. The published binary's `list` projects a *demo*
constitution, so an agent working on this repo never naturally sees the self-law that
actually reacts (`tianheng_constitution()` in `self_governance.rs`). This capability closes
that entry-point gap — the first dogfood of the 潛移 (gravity) face (see `PROJECT.md`): the
declared law, rendered where an agent reads it, so its continuations imitate the real shape
rather than the demo. Two contracts are kept distinct: the **repo artifact** must not drift
from the enforced law (a test reacts), and the **public renderer's Markdown layout** is a
human/agent surface that may evolve (a doc contract, never frozen — JSON remains the machine
contract).
## Subject

- `crates/shengmo/src/law.rs`
- `crates/shengmo/tests/self_governance.rs`
- `crates/kanhe/tests/law_restatement.rs`
- `crates/kanhe/src/restatement.rs`
- `AGENTS.self-law.md`

## Requirements
### Requirement: Self-law projection is generated from the enforced self-constitution

The constitution this projection is generated from SHALL be **library code**, not a function inside a test
file. It is written with the product's own declaration API — the capability applied to its own author — and a
declaration living among a dozen `#[test]` functions reads as a test rather than as the repository's law, which
is how a governance document came to describe the reactions beside it as something they are not.

Authored text SHALL NOT restate a declared dependency allowlist, and the reaction holding that SHALL read
**every** declared allowlist against **every** tracked governance document, not one crate's line comments
against one dimension's. Measured before this change, the reaction read only Rust line comments under the
`tianheng` shell and only the shell's allowlist, while `PROJECT.md` named every member of `guibiao`'s live
allowlist — the same second source of truth, in a file class nothing scanned. A rule enforced at one site and
not its neighbour is a rule about the site.

What a declaration cannot carry SHALL stay prose: why a boundary exists, what it protects, the narrative of
the family. What it can carry — the membership — belongs to the declaration and its projection, and the repair
for a restatement is a pointer to `AGENTS.self-law.md`.

#### Scenario: A governance document names every member of a declared allowlist

- **WHEN** a tracked governance document names every member of any live `restrict_dependencies_to` allowlist
- **THEN** the reaction fails, naming the document, the crate whose allowlist was copied, and the members —
  and directs the repair to the projection rather than to a rewording

#### Scenario: The law is reached as a library

- **WHEN** the projection is generated and the reaction runs the constitution
- **THEN** both read the same exported declaration, so a projection cannot be generated from one definition
  while the reaction evaluates another

### Requirement: Observation bounds

The comment reactions above read **authored text**, and what they cannot read SHALL be a limit this capability
declares rather than one left for the author who trips it. Both are over-reactions, which is the safe direction here: a
false positive costs a sentence rewritten, while the false negative would be a restated declaration that no
reaction governs. Neither SHALL be closed by teaching a recognizer to read intent — that is a heuristic over
prose, which this repository has measured and rejected.

#### Scenario: A doc example of the dependency DSL is refused — a stated bound

- **WHEN** a line comment under the shell names `restrict_dependencies_to(` in order to teach the re-exported
  DSL rather than to restate this shell's own declaration
- **THEN** the reaction refuses it anyway, a stated bound: it reads a comment's text and never its purpose, and
  the shell publishes that DSL, so the shape is live even with no instance in the tree today
- **PINNED-BY** `a_doc_example_of_the_dependency_dsl_is_refused`

#### Scenario: A comment naming every member for another reason is refused — a stated bound

- **WHEN** one contiguous line-comment block names every current allowlist member for a purpose other than
  copying the declaration — a crate-level note on what the shell composes, say
- **THEN** the reaction refuses it anyway, a stated bound: it asks whether the members all appear and never why,
  so a block naming them for another reason reads the same as a copied census
- **PINNED-BY** `a_comment_naming_every_member_for_another_reason_is_refused`

### Requirement: A staleness test reacts when the checked-in projection drifts

A test SHALL fail when the checked-in projection artifact differs, byte for byte, from the live projection generated from `tianheng_constitution()`. The comparison SHALL cover the **entire** artifact — both the generated boundary projection and any fixed preamble (the preamble being a generated constant, never hand-edited prose) — so no part of the artifact can drift unnoticed. The test SHALL follow the repository's existing repo-only discipline: it SHALL skip when run outside a workspace checkout (e.g. a packaged crate tarball), and SHALL fail loudly rather than skip when a workspace is expected but absent (the `TIANHENG_WORKSPACE_TESTS` signal). A one-command regeneration path SHALL overwrite the checked-in artifact from the live projection instead of asserting.

The byte-check reaction itself SHALL be a **reusable public helper** so an adopter can gate their own projected constitution with the same mechanism (the 潛移 adoption face) rather than hand-rolling it: given the **live** projection string, the artifact **path**, a **regenerate** command string, and a **bless** flag, the helper SHALL — when `bless` is true — overwrite the file with the live projection (creating any missing parent directories) and succeed; otherwise it SHALL compare the checked-in file to the live projection and **fail** when they differ, when the file is **missing**, or when it is **unreadable**, returning an actionable error that names **both the artifact path and the regenerate command**. A write failure under `bless`, or a read failure otherwise, SHALL be returned as an error, never a silent success. The helper SHALL NOT itself read the environment — the **caller** supplies `bless` (so the helper is a pure function of its arguments, with no process-global env dependency and no parallel-test hazard); Tianheng's own self-law staleness test reads its `BLESS` signal and passes it in, and is one caller of this helper.

#### Scenario: A stale checked-in projection fails the test

- **WHEN** the checked-in projection artifact no longer matches the live projection of `tianheng_constitution()`
- **THEN** the staleness test fails, naming the artifact and instructing to regenerate it

#### Scenario: Regeneration refreshes the artifact instead of asserting

- **WHEN** the regeneration signal is set and the staleness test runs
- **THEN** the checked-in artifact is overwritten with the live projection and the test does not assert staleness

#### Scenario: The test skips outside a checkout but fails loud when a workspace is expected

- **WHEN** the test runs where no workspace root is present
- **THEN** it skips if no workspace is expected, but fails loudly if `TIANHENG_WORKSPACE_TESTS` declares a workspace must be present

#### Scenario: The reusable gate helper reacts to drift, missing, and unreadable artifacts

- **WHEN** the gate helper is called with `bless = false` and the checked-in file differs from the live projection, or does not exist, or cannot be read
- **THEN** it returns an error naming the artifact path and the regenerate command, so the caller's `cargo test` reacts — a missing or unreadable projection is "cannot confirm fresh", never a silent pass — and returns success only when the file byte-matches the live projection

#### Scenario: The gate helper regenerates on bless, creating parent directories

- **WHEN** the gate helper is called with `bless = true`
- **THEN** it overwrites the artifact with the live projection (creating any missing parent directories) and succeeds, returning an error if the write itself fails — the caller supplies `bless`, so the helper touches no process-global environment

### Requirement: The Markdown projection is a human/agent-readable surface, not a machine-stable contract

The constitution-to-Markdown projection SHALL be produced by the **same renderer** as `list --format markdown` and SHALL add nothing of its own (no preamble, no trailing newline), so the agent-loaded artifact and the CLI projection cannot diverge. The public rendering helper SHALL document that its Markdown layout is intended for display, review, and LLM context, and **MAY evolve in any compatible release** to improve readability or imitability; consumers needing a stable, machine-parseable contract SHALL use the JSON projection instead. No automated test SHALL pin the helper's exact Markdown layout as a contract — that absence is deliberate, so evolving the layout (e.g. foregrounding the `reason`) is not a breaking change to a machine consumer. (The evolvability clause is held by the doc-comment, verified by review, not by an automated assertion — see design.md, Contract B.)

#### Scenario: The helper renders the same projection as the CLI, byte for byte

- **WHEN** a constitution is rendered through the public Markdown helper
- **THEN** the output equals, byte for byte, what the `list --format markdown` path projects for that same constitution — the helper prepends and appends nothing (one renderer, no parallel projection path)

#### Scenario: The Markdown format is documented as evolvable (review-verified)

- **WHEN** a reviewer reads the public Markdown helper's doc-comment
- **THEN** it states the layout is human/agent-readable and may evolve, and directs machine consumers to the JSON projection — and no golden/snapshot test fixes the helper's exact output as a contract

### Requirement: The preamble describes only how to read the projection, not crate-specific law

The artifact's fixed preamble SHALL describe only how to read and use the projection and the reaction loop it serves (declare intent in code; observe only what has an observation source; react with the 0/1/2 outcomes; repair toward the declared `reason`; never weaken the law to pass; 三儀 measure, 三司 administer). The preamble SHALL NOT make crate-specific architectural claims; any such claim SHALL appear only in the generated projection, where it traces to a boundary that actually reacts (no open-loop prose prescription).

#### Scenario: Crate-specific law appears only in the generated projection

- **WHEN** the preamble is read
- **THEN** it describes the reaction loop and how to read the projection, and makes no crate-specific architectural claim — such claims appear only in the generated boundary projection below it

### Requirement: A dimension's declared allowlist SHALL obey the law its reason quotes

The reaction over each dimension's declared allowlist SHALL assert **both** statements separately: that the clause is present in the reason, and that the allowlist **names no
sibling dimension**. They are different statements, and only the first was asserted.

That gap was a false negative, reproduced rather than reasoned about: widening a dimension's allowlist to
name a sibling left **every** test binary in this workspace green, and `AGENTS.self-law.md` regenerated to
print the widened membership **directly beneath** the reason that forbids it. The membership itself is not
written here — restating it is the second source of truth this capability forbids, and a requirement that
reproduces what it governs goes stale on a legitimate amendment with no reaction to say so.

Neither of the two reactions a reader would expect to catch it can. The staleness check pins the projection
against the declaration, so a blessed projection of a widened allowlist is *fresh* — freshness is not truth. And
the dependency reaction cannot fire on a **widened** allowlist, because permitting more than the tree uses
produces no violation. So this assertion is the sole guard, which is why it is a requirement rather than an
implementation detail.

The clause check SHALL remain, and every limit of this reaction SHALL be **declared**, with each extent read off
a run of that limit's own WHEN rather than off an argument for it. Three are declared below. Two belong to the
wording half and go in **opposite directions**: paraphrasing the clause makes the reaction *fire*, and a
`because` carrying the literal clause while *negating* the law passes. One belongs to the membership half's
enumeration: a rule variant the filter does not reach.

A fourth was declared here and is **retired**, which is recorded rather than quietly dropped: the hand-kept
dimension list. Its own text named its closing condition — *the dimension set derived from something that
enumerates it rather than typed beside it* — and that is what the reaction now does, reading workspace members
from cargo and comparing both ways. Retired against a run of its own WHEN on the post-change tree, not against
the argument that it should be: a dimension removed from the literal, and a dimension crate the literal never
named, are both refused now.

Declaring SHALL NOT be deferred for want of a pin. A bound may be declared **unpinned** against a tracker, and
that is the right form here: what a pin of these would need — the reaction run over a supplied declaration
rather than its predicate over a string — is work the tracker owns, and withholding the declaration until then
keeps a measured false negative out of the register that a reader is told to consult *before* reporting a
behaviour as a defect. A draft of this change confused the two acts and deferred all four; the same draft had
declared the first as a false *negative*, which one run of its WHEN falsified.

Membership is the structural half; wording is the half that is not.

#### Scenario: A reason that paraphrases the law is refused — a stated bound

- **WHEN** a dimension's `because` states the mutual-independence law in different words and does not carry the
  literal clause
- **THEN** the reaction **fires**, refusing a reason that genuinely states the law. Measured by writing that
  WHEN into the tree: paraphrasing `guibiao`'s clause produces *"dimension boundary for `guibiao` dropped the
  `三儀 ⊥ 三儀` clause"*. The direction is the safe one — an author meets a refusal to argue with — and closing
  it needs the check to decide two wordings state one law, a judgement over prose measured and rejected here
- **UNPINNED** `BACKLOG.md` — *four limits of the mutual-independence check*

#### Scenario: A reason carrying the clause while negating the law is not observed — a stated bound

- **WHEN** a dimension's `because` quotes `三儀 ⊥ 三儀` and then states that it does not bind that dimension
- **THEN** nothing reacts, and `AGENTS.self-law.md` projects the negation to every agent that loads it. This is
  the serious direction of the pair: the teaching surface can carry the law's opposite while satisfying the
  check that exists to keep the law taught. Measured, with the projection blessed and the whole suite green
- **UNPINNED** `BACKLOG.md` — *four limits of the mutual-independence check*

#### Scenario: A workspace-dependency allowlist is not examined — a stated bound

- **WHEN** a dimension declares the law through `restrict_workspace_dependencies_to` rather than
  `restrict_dependencies_to`
- **THEN** the reaction never examines it, though that rule governs workspace-member edges specifically and is
  the more natural one for this law. Measured: a second `guibiao` boundary of that variant naming `hunyi` is
  green, and set coverage still reads the three dimensions
- **UNPINNED** `BACKLOG.md` — *four limits of the mutual-independence check*

#### Scenario: A dimension's allowlist names a sibling

- **WHEN** a dimension's `restrict_dependencies_to` allowlist contains another dimension's package name
- **THEN** the reaction fails, naming the target, the sibling, and that the boundary now permits exactly what
  its own reason forbids

#### Scenario: The projection of a widened allowlist is fresh

- **WHEN** an allowlist is widened and the projection is regenerated
- **THEN** the staleness check is clean, because it compares the projection with the declaration — so freshness
  cannot stand in for this assertion

### Requirement: The Three-Layer Agent Law structure is formalized and documented

The repository documentation SHALL define and teach the **Three-Layer Agent Law** structure: Layer 1 (Universal Preamble), Layer 2 (Generated Projection Body from `constitution_markdown()`), and Layer 3 (Rust Law Source governed by `CODEOWNERS`). The preamble discipline SHALL be explicitly documented in [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md), restricting preambles to universal meta-instructions and vocabulary while forbidding crate-specific architectural claims.

#### Scenario: COOKBOOK documents the Three-Layer Agent Law recipe

- **WHEN** an adopter reads [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md)
- **THEN** it contains a dedicated recipe teaching how to assemble the preamble, generate the projection body, gate staleness with `GovernanceTest`, and update the law with `BLESS=1`

### Requirement: The preamble SHALL say what each governance name does

The universal preamble SHALL introduce no governance name without saying what that name does, and SHALL NOT
present the governance surfaces as a set whose members share a shape. The preamble is loaded by every agent,
and a bare set — *三司 (垂象 · 實錄 · 校讎) administer* — hands a reader three handles and no referent, which
reads as three things to be found rather than three descriptions of surfaces that already exist.

Measured: none of the three names appears in any shipped public item, crate name, manifest, `description`, or
adopter-facing document. Their referents are `crates/guibiao/src/projection.rs` with
`crates/tianheng/src/runner/render.rs`, `crates/xuanji/src/baseline.rs`, and `.github/CODEOWNERS` with
`crates/tianheng/src/constitution.rs` — one of which is not under `crates/` at all.

**Nothing observes this requirement.** Deciding that a name lacks a referent, or that a sentence describes
rather than groups, is a judgement over prose — the instrument `AGENTS.md` records as designed, measured three
times and rejected. It is stated here with that absence beside it rather than left for a reader to discover,
which is what this repository does with a rule it cannot react to. The preamble is a hand-written constant and
this property can fail.

#### Scenario: The preamble names a governance surface

- **WHEN** the universal preamble mentions 垂象, 實錄 or 校讎
- **THEN** it says what that surface does, and does not present the three as a set with a common shape.
  What stands in place of a reaction is the generated projection: `AGENTS.self-law.md` is regenerated and
  never hand-edited, and `self_law_projection_is_fresh` holds — which pins the projection to the declaration
  and is **freshness, not truth**, so this scenario is evidenced by review rather than by a guard
