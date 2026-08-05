## Context

An observation bound is a claim that a reaction **stops** at a certain shape. It is the one claim class
no reaction defends, and this tree carries roughly a hundred of them across four unlinked surfaces:
`openspec/specs/*` prose (43 occurrences), `crates/*/src` rustdoc (69), `BACKLOG.md` ACCEPTED DEBT (3),
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

**1. A register entry is a `#### Bound: <id>` block inside the requirement it bounds.**
Verified empirically, not assumed: a throwaway spec carrying such a heading alongside a `#### Scenario:`
passes `openspec validate --strict`, and all 29 existing specs pass it today, so the syntax neither
breaks validation nor needs schema work. The alternative shapes were rejected — an HTML comment hides
the claim from the human reader, which defeats surfacing it; a separate register file re-creates the
unlinked-surface problem this change exists to end; a fenced code block reads as sample code rather than
as a requirement's own qualification.

Each block carries `- **statement**:` and exactly one of `- **pinned-by**: \`<test fn name>\`` or
`- **unpinned**: <tracker>`.

**2. Ids are `<capability>/<slug>`.** Globally unique and self-locating, so a violation message or a
`BACKLOG.md` entry can cite a bound without a path. A bare slug would collide across 29 capabilities;
an opaque number would need its own allocation ledger, which is another hand-maintained surface.

**3. The reaction is a shell gate plus its own failure matrix**, `scripts/check_bound_register.sh` and
`scripts/test_bound_register.sh`, wired into `AGENTS.md`'s Definition of Done and mirrored verbatim into
CI. This follows the established pair for a repo-wide text property (`check_release_coherence.sh` /
`test_release_coherence.sh`, and the publish-source gate). A Rust test was considered: it is right for a
property of Rust semantics, which this is not — the subject is Markdown and test-name existence across
the tree, and `check_dod_coherence.sh` already enforces the DoD/CI mirroring a shell gate needs.

**4. A cited test must resolve to exactly one definition.** Zero fails (a renamed or deleted test must
not read as coverage). **Two also fails**: a name defined twice makes the citation ambiguous, so the
register would point at a set rather than a reaction. Matching is on the definition form (`fn <name>(`),
never a bare mention, so a citation satisfied by a comment is not possible.

**5. The completeness floor is a prose scan, and it is a floor rather than a proof.** Every
bound-prose occurrence in `openspec/specs/*` outside a register block fails the gate. This makes the 43
occurrences the register's mandatory minimum while leaving a bound worded outside the pattern
undetectable — which the projection states in its own header rather than letting the index imply
totality. `PROJECT.md`'s rule applies to the index as much as to a dimension: claim exactly the
guarantee held.

**6. An unpinned bound is representable, deliberately.** The back-fill will find bounds with no pinning
test; requiring one would make the gate block on precisely what it exists to discover, and the practical
result of that is a smaller register rather than more tests. So `- **unpinned**: <tracker>` is a legal
entry, the projection surfaces the unpinned count as its headline number, and that count is the audit
backlog. This mirrors `violation-baseline`'s own settled design — record what is accepted, gate on new
drift — and inherits its discipline: an unpinned entry names a tracker, never merely asserts.

**7. The projection is `docs/observation-bounds.md`, generated and staleness-checked.** Regenerated by
the gate under an environment flag and compared byte-for-byte otherwise, exactly as
`self_law_projection_is_fresh` treats `AGENTS.self-law.md`. It is not placed at the repository root: the
root projection is the self-law an agent loads as its idiom, and a second root document dilutes that.
`AGENTS.md` points at it, the way `BACKLOG.md` points at `docs/history/`.

## Risks / Trade-offs

- **A prose-pattern floor produces false positives** — a sentence containing "documented bound" that
  declares nothing would demand a register entry. → No exemption marker is added, because an exemption
  list rots into the thing it was meant to avoid; the author registers the bound or rephrases the
  sentence. The failure is loud and local either way.
- **The floor is blind to a bound worded outside the pattern** → stated in the projection header and in
  the capability's own spec, as its own registered bound. The index having a bound is not irony; a
  surface that claimed otherwise would be the problem.
- **The back-fill is all-or-nothing across the 43 occurrences**, because the floor fails on any that
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
