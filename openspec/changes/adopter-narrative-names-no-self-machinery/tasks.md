# Tasks

## Reaction

- [ ] `scripts/check_release_coherence.sh`: `changelog_sections()` emits a `CITATION` record carrying
      section, heading in force, and the machinery path named. Attribution is list-item → heading →
      section, all grammar.
- [ ] Machinery is recognised as a token: a path under `scripts/`, or a bare basename that
      `git ls-files scripts/` resolves. The enumerator is the authority; no list of gate names.
- [ ] Adopter-facing = every `### ` heading except `Self-governance`, so an unanticipated heading reacts.
- [ ] Scope is `[Unreleased]`; a dated section is record.
- [ ] Refuse to judge (exit 2) when `git ls-files scripts/` cannot be read, rather than reporting clean
      against an empty enumerator.

## Twin matrix — `scripts/test_release_coherence.sh`

Each direction asserts an exit **code**.

- [ ] adopter heading names a `scripts/` path → 1
- [ ] same entry under `### Self-governance` → 0
- [ ] adopter heading names a bare basename the enumerator resolves → 1
- [ ] bare basename resolving to no tracked file under `scripts/` → 0
- [ ] dated section names a gate → 0
- [ ] unquoted basename in prose → 0
- [ ] enumerator unreadable → 2

## Bounds

- [ ] `crates/tianheng/src/bounds.rs`: three `BoundDecl` for this reaction — dated-section scope and
      unquoted-prose recognition **pinned**, the subject residual **unpinned** against a tracker.
- [ ] `crates/tianheng/tests/gate_shape_contract.rs`: `a_dated_section_naming_a_gate_is_a_stated_bound`
      and `an_unquoted_basename_in_prose_is_a_stated_bound`, each run against a fixture and each seen to
      fail before the reaction exists.
- [ ] `BACKLOG.md`: the tracker *the self-governance residual is a judgement over an entry's subject*.

## The document itself

- [ ] `CHANGELOG.md` `[Unreleased]` grows `### Self-governance`; the nine entries naming machinery move
      there, **except** the publish-provenance entry, which is rewritten in adopter terms with the gate
      filenames dropped.
- [ ] `[0.4.0]`'s five entries are untouched — they are record.
- [ ] The `[Unreleased]` entry for this change itself goes under `### Self-governance`, which is the
      rule holding on its own introduction.

## Verification

- [ ] Each new bound pin run against the tree **without** the reaction, failure recorded in the PR
      `## Verification`.
- [ ] The reaction run against the current `[Unreleased]` *before* the entries move: it must name the
      nine, which is the guard seen to fail on the real document rather than on a fixture only.
- [ ] `bash scripts/check_bound_register.sh` census agrees across `BACKLOG.md`,
      `docs/observation-bounds.md` and `docs/observation-bound-extents.md`.
- [ ] Full `AGENTS.md` Definition of Done in order.
