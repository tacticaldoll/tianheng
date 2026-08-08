## Why

`三儀 ⊥ 三儀` — a dimension never learns from a sibling — is this family's foundational law. Each dimension's
`restrict_dependencies_to` boundary quotes it in its `because`, and a reaction asserts the quote is there.

**Nothing asserted that the allowlist obeys it.** Reproduced: widening `guibiao`'s allowlist to name `hunyi`
left every test binary in this workspace green, and `AGENTS.self-law.md` regenerated to print
`only: serde_json, xuanji, xingbiao, hunyi` directly beneath the reason that forbids it.

Neither reaction a reader would expect to catch it can. The staleness check pins projection against
declaration, so a blessed projection of a widened allowlist is **fresh** — and freshness is not truth. The
dependency reaction cannot fire on a *widened* allowlist, because permitting more than the tree uses produces no
violation.

## How this was found

Three attempts at one `PROJECT.md` paragraph were withdrawn. The third tried to stop *restating* the law and
**cite** the generated projection instead — and review showed the citation bought freshness, not truth, because
the projection carries the allowlists without anything holding them to the law. The paragraph kept coming out
wrong because the law it was describing was only half reacted to. That is `AGENTS.md`'s *A repair loop is a
diagnosis, not a schedule* arriving at its own conclusion: the shape was wrong, not the wording.

## What Changes

- The reaction asserts allowlist **membership** as well as the clause's presence, naming the sibling it found.
- `self-law-projection` states the obligation, why this assertion is the sole guard, and that the clause check's
  text-only limit remains a declared bound.

Not **BREAKING**: a `tests/` reaction of this repository, shipping in no crate.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `self-law-projection`: one requirement added.

## Impact

- `crates/tianheng/tests/self_governance.rs`, `openspec/specs/self-law-projection/spec.md` at sync, a
  `CHANGELOG.md` entry.

## What this deliberately does not do

It does not touch the `PROJECT.md` paragraph. Three attempts at it were withdrawn; with the law now reacted to
in the direction that matters, a citation from that file would buy truth rather than freshness — but that is a
separate change, and writing it inside this one would repeat the pattern of fixing prose while the reaction was
the subject.
