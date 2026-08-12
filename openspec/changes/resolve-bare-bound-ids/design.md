## Context

Four kinds of reference live in this repository, and three of them resolve:

| reference | resolver |
|---|---|
| a path | `crates/kanhe/tests/reference_integrity.rs` |
| a `--exact` identifier | `crates/kanhe/src/gate_identity.rs` |
| `(bound: <capability>/<slug>)` in a spec | `crates/kanhe/src/bound_register_parse.rs` |
| a **bare** `<capability>/<slug>` anywhere else | — |

The fourth is where both live defects sit, and they are the same mistake made twice in two crates: the id was
derived from a requirement's prose rather than from the declaring scenario's heading.

## Goals / Non-Goals

**Goals:**

- A bound id resolves wherever it is written, because resolution belongs to the id.
- Recognition stays mechanical and enumerated, so a capability added later is covered without an edit here.

**Non-Goals:**

- Deciding whether the prose *around* a bare id describes the bound it names. The wrapped form already
  declares that limit and this inherits it verbatim.
- Any judgement over sentences. The instrument this repository designed, measured three times and rejected is
  a detector over *prose*; this resolves an identifier with a fixed shape into a produced set.
- Widening the corpus beyond tracked Rust and Markdown. A bound id in a shell comment or a manifest is not a
  shape anyone writes today, and admitting a corpus nothing exercises would be a name without a reaction.

## Decisions

**Recognize by maximal run of path characters, then require exactly one slash.** This is the rule the
adopter-narrative reaction already uses, adopted for the same reason: a substring match would read
`repository-checks/` out of the middle of `openspec/specs/repository-checks/spec.md` and refuse a path for
resembling a reference. Reading the whole run makes that run *not* a `<capability>/<slug>` pair, so the path is
excluded by construction rather than by an exception list.

**The left side must be an enumerated capability, read from `openspec/specs/`.** This is what makes the
matcher precise rather than a false-positive machine — the discriminator is not "looks like two kebab words
around a slash" but "names a capability this repository declares". Enumerating rather than listing is the
register's own prohibition, and it means a capability added later is recognized for free.

**Measured before designing, not argued.** Across tracked Rust and Markdown the shape matched a few hundred
tokens and all but three resolved; those three were the defects. The precision is a property of requiring the
capability set on the left, and it was measured rather than hoped for — the figures are anchored to that
moment and deliberately not written into the spec as a live claim, because no reaction produces them.

**Corpus is tracked Rust and Markdown, and the capability's `## Subject` is deliberately not widened.** The
register's subject names its own implementation and the specs it reads; a reaction may read a corpus wider than
its capability's subject, as `census.rs` already does over tracked Markdown. Widening the subject to
`crates/**/*.rs` would file every Rust change in the repository under this capability, which is the opposite of
what a subject is for.

**The census repair rides along and is prose only.** `observation_bound_model.rs` saying *"the family's four
sets"* over a chain of five is the class *A census is produced, never typed*, but it lives in a Rust doc
comment and the census reaction reads tracked Markdown. So it is repaired by removing the number, not by
extending that reaction — and this is said rather than left for a reader to notice the asymmetry.

## Risks / Trade-offs

**A bound id inside a code fence or an example could be refused** → It would have to be a well-formed
`<capability>/<slug>` naming an undeclared bound, which is exactly the shape worth refusing wherever it sits.
If a legitimate example ever needs one, the register's existing vocabulary for that is to declare the bound.

**The corpus grows as the tree does** → Every new Rust or Markdown file is scanned. That is the point, and it
is why the recognizer must stay cheap: one regex over a maximal run, one set membership, one map lookup.

**Two resolvers for one syntax** → The `(bound: …)` form and the bare form must not disagree about what a
valid id is. The bare recognizer therefore resolves into the **same** produced set the wrapped one uses rather
than deriving its own, so a disagreement is not expressible.

## Migration Plan

None. Nothing published moves; the `xuanji` edit is a `#[cfg(test)]` doc comment. The three stale citations are
corrected in the same change that makes them observable, so the reaction lands green rather than red.

## Open Questions

- **Should the corpus eventually include shell and TOML?** Not now: no bound id is written in either today, and
  a corpus nothing exercises is a name without a reaction. Named so the omission is deliberate.
