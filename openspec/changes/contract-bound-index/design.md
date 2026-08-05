## Context

An observation bound is a claim that a reaction **stops** at a certain shape. It is the one claim class
no reaction defends, and this tree carries roughly a hundred of them across four unlinked surfaces:
`openspec/specs/*` (55 lines naming a bound, of which 25 are scenario headings declaring one and 29 are
prose or bare THEN clauses), `crates/*/src` rustdoc (69), `BACKLOG.md` ACCEPTED DEBT (3),
and a `*_is_a_stated_bound` / `*_is_a_documented_bound` test-naming convention (21 tests). The sets do
not correspond — the symlinked-directory bound's pinning test is cited by name in `BACKLOG.md` and is
absent from the 21 — so no existing surface can be promoted to the index by itself.

Two constraints come from the repository rather than from the problem. `AGENTS.md` names
`openspec/specs/*` as the per-capability requirement truth, so a bound's declaration belongs there and
nowhere new. And `PROJECT.md`'s drift law forbids a hand-maintained structural document: the index must
be **generated and staleness-checked**, the way `AGENTS.self-law.md` is, or it becomes the largest prose
drift liability in the tree instead of the cure.

## Goals / Non-Goals

**Goals:**
- One declaration site per bound, carrying its statement and the name of the test that pins it.
- A reaction in both directions: a citation that no longer resolves fails; a bound stated in spec prose
  and left unregistered fails.
- A generated projection an auditor and an agent can read instead of re-reading 29 specs.
- Make the register's own incompleteness explicit, so the index cannot be mistaken for a totality proof.
- Represent a bound that has **no** pinning test, so the register is not blocked by the very gaps it
  exists to surface.

**Non-Goals:**
- Bounds stated only in rustdoc, and `BACKLOG.md`'s ACCEPTED DEBT entries. Registering those moves a
  claim between documents, changing a capability's requirement surface; each earns its own change.
- The full scenario-to-reaction coverage program over 804 SHALL statements and 1020 scenarios. That is a
  multi-window program; this change builds the instrument it would need, not the program.
- A CLI subcommand. `tianheng list` projects an **adopter's declared** constitution; the register is
  about *this project's own* observation contract, a different subject with a different audience. Fusing
  them would put Tianheng's internal bounds in every adopter's law projection.
- Renaming any test to satisfy a convention. The register cites tests by their existing names.

## Decisions

**1. A bound is declared where this repository already declares them: as a `#### Scenario:` whose heading
marks it a bound, under the requirement it qualifies.** The first apply pass said "under the capability's
`Observation bounds` requirement" and the enumeration refuted it: 21 of 24 sit under the requirement they
qualify, and hoisting them into a common section would separate each bound from the reaction it limits.
That requirement, which three specs carry, is a place bounds are gathered — not the definition of one. This reverses the shape proposed before apply,
because the convention already exists and is further along than the proposal assumed. Three specs carry
that requirement today — `inline-symbol-path-confinement` ("Observation bounds are stated, not silent"),
`semantic-unsafe-confinement`, and `semantic-visibility-boundary` — and 25 scenarios across more specs
already name themselves `… is a stated bound` / `… is a documented bound`, each with its own WHEN/THEN.
One spec even cross-references the section by name.

Introducing a `#### Bound:` block would therefore be a **third** convention competing with two that
exist, and for the 25 already-declared bounds it would state the same bound twice — the drift this
register exists to end. Adopting the existing shape also means each bound arrives with a WHEN/THEN
already written, which is what makes a pinning test findable at all.

**2. The register's added element is a citation bullet inside the bound scenario**: exactly one of
`- **PINNED-BY** \`<test fn name>\`` or `- **UNPINNED** <tracker>`, beside its WHEN/THEN. Verified rather
than assumed: a throwaway spec carrying both forms passes `openspec validate --strict`, which all 29
specs pass today, so the syntax costs no schema work. The alternatives were rejected for the reasons
above — a separate register file re-creates the unlinked surfaces, and an HTML comment hides the claim
from the human reader it exists to serve.

**3. A bound's id is derived from its location, not allocated.** `<capability>/<scenario-slug>` comes
from the spec directory and the scenario name, so nothing hand-assigns an id and no allocation ledger is
needed — a ledger being another hand-maintained surface. A renamed scenario changes the id, which is
correct: the citation follows the declaration rather than outliving it.

**4. A spec that states a bound in prose but carries no Observation-bounds requirement fails.** This is
the floor's real force, and its size is measured: 3 of 29 specs carry that requirement today, while 11
more state bound prose without one. The 29 non-heading occurrences are mostly THEN clauses whose scenario
heading does not mark the bound, so the floor's effect is to migrate them into declared bound scenarios
rather than merely to demand a block somewhere. Migrating a bound *into* a
scenario is a strict improvement independent of this register: a bound stated only in prose has no test
case by construction.

**5. The reaction is a shell gate plus its own failure matrix**, `scripts/check_bound_register.sh` and
`scripts/test_bound_register.sh`, wired into `AGENTS.md`'s Definition of Done and mirrored verbatim into
CI. This follows the established pair for a repo-wide text property (`check_release_coherence.sh` /
`test_release_coherence.sh`, and the publish-source gate). A Rust test was considered: it is right for a
property of Rust semantics, which this is not — the subject is Markdown and test-name existence across
the tree, and `check_dod_coherence.sh` already enforces the DoD/CI mirroring a shell gate needs.

**6. A cited test must resolve to exactly one definition.** Zero fails (a renamed or deleted test must
not read as coverage). **Two also fails**: a name defined twice makes the citation ambiguous, so the
register would point at a set rather than a reaction. Matching is on the definition form (`fn <name>(`),
never a bare mention, so a citation satisfied by a comment is not possible.

**7. The completeness floor is a prose scan, and it is a floor rather than a proof.** Every
bound-prose occurrence outside a declared bound scenario, and carrying no resolving reference, fails the
gate. This makes the 55 measured
occurrences the register's mandatory minimum while leaving a bound worded outside the pattern
undetectable — which the projection states in its own header rather than letting the index imply
totality. `PROJECT.md`'s rule applies to the index as much as to a dimension: claim exactly the
guarantee held.

**8. An unpinned bound is representable, deliberately.** The back-fill will find bounds with no pinning
test; requiring one would make the gate block on precisely what it exists to discover, and the practical
result of that is a smaller register rather than more tests. So `- **UNPINNED** <tracker>` is a legal
citation, the projection surfaces the unpinned count as its headline number, and that count is the audit
backlog. This mirrors `violation-baseline`'s own settled design — record what is accepted, gate on new
drift — and inherits its discipline: an unpinned entry names a tracker, never merely asserts.

**9. The projection is `docs/observation-bounds.md`, generated and staleness-checked.** Regenerated by
the gate under an environment flag and compared byte-for-byte otherwise, exactly as
`self_law_projection_is_fresh` treats `AGENTS.self-law.md`. It is not placed at the repository root: the
root projection is the self-law an agent loads as its idiom, and a second root document dilutes that.
`AGENTS.md` points at it, the way `BACKLOG.md` points at `docs/history/`.

**10. Prose may reference a declared bound, and a reference is checked.** Written
`(bound: <capability>/<slug>)`, resolving to exactly one declared bound — none fails as a dangling
pointer, two fails and thereby checks the derived id's uniqueness instead of assuming it.

This was not in the design before the gate ran; the gate's first pass produced it. Of the 22 prose
occurrences it flagged, some are not declarations at all but sentences legitimately **pointing at** a bound
declared elsewhere — `inline-symbol-path-confinement` says "a stated bound (see \"Observation bounds\")",
and `runtime-origin-assertion` says "the same stated bound as the semantic dimension". The design's original
answer was "register the bound or rephrase the sentence", and both options are wrong for those: rephrasing
degrades prose that is doing its job, and registering restates a bound that already exists somewhere else —
which is the drift now filed as a live `READY-PATCH` item. A reference is the third option, and it is the
same mechanism that entry's promotion trigger was waiting for, arriving early because the gate's own false
positives demanded it.

## Risks / Trade-offs

- **A prose-pattern floor produces false positives** — a sentence containing "documented bound" that
  declares nothing would demand a bound scenario. → No exemption marker is added, because an exemption
  list rots into the thing it was meant to avoid; the author registers the bound or rephrases the
  sentence. The failure is loud and local either way.
- **The floor is blind to a bound worded outside the pattern** → stated in the projection header and in
  the capability's own spec, as its own registered bound. The index having a bound is not irony; a
  surface that claimed otherwise would be the problem.
- **The back-fill is all-or-nothing across the 55 measured occurrences**, because the floor fails on any that
  remain unregistered. → That is the feature that prevents a half-linked index from reading as complete,
  but it means this change cannot land partially. The first task is enumeration, so the real size is
  known before any registration is written rather than discovered mid-way.
- **`unpinned` could become a dumping ground** → the projection makes the count its headline rather than
  a footnote, and each entry names a tracker, so growth is visible instead of quiet.
- **A future bound added without a register entry passes if its wording avoids the pattern** → the same
  residual as any prose-derived floor. It is bounded by the floor, not closed by it, and closing it
  would require every bound to originate as a register entry — a discipline this change can state but
  cannot react to.

## Migration Plan

Additive; nothing to roll back beyond reverting the change. Order matters within apply: enumerate the
occurrences first, so the scope is measured rather than assumed; write the gate and prove each failure
direction against a fixture before registering anything, so the register is written against a reaction
that already bites; then register, generate the projection, and wire the gate into the Definition of
Done and CI in the same change that makes it pass.

## Open Questions

- Whether a bound that belongs to two capabilities (an agreement pinned by a cross-dimension conformance
  test) is registered once in the capability that owns the reaction, or once per capability with the same
  pinning test. The gate's exactly-one-definition rule permits either; the projection reads better with
  one entry, and a cross-dimension bound is the case that argues for two.
- Whether the gate should also require that a registered bound's `statement` appear nowhere else in the
  specs, which would prevent a bound from being restated and drifting — valuable, and possibly too
  strict to satisfy on the first pass.
