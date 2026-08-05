## Context

`scripts/check_bound_register.sh` shipped into `release/0.4.1` with 41 declared bounds across 15
capabilities and a 26-direction failure matrix. An adversarial review found four directions where the
reaction is weaker than the requirement it enforces. Two of them are the register's own forbidden class —
a citation that reads as coverage while defending nothing — reachable inside the mechanism built to end
exactly that.

The constraint that shapes every decision below: this register governs a suite it does not own. It may
require the **form** of what a spec author writes (a scenario heading, a citation bullet), because that is
authored in the spec. It may not require a convention of a pre-existing test suite — the measurement that
settled this is on record, since the bound-pinning tests follow at least three naming variants and some
carry no "bound" in the name at all, so a name-keyed register would have reported pinned bounds as
unpinned.

Both tightenings were verified against the tree **before** being specified, not after: all 36 `PINNED-BY`
citations resolve to a function whose attribute run contains `#[test]`, and all 5 `UNPINNED` citations are
`BACKLOG.md READY-PATCH "declared bounds with no pinning test"`. Neither direction is a migration, and
neither would have been safe to specify on the assumption that they were.

## Goals / Non-Goals

**Goals:**

- A `PINNED-BY` citation cannot be satisfied by a function that never runs as a test.
- An `UNPINNED` citation cannot be satisfied by a sentence that names no owner.
- Regeneration's exit code carries the family's contract rather than "the file was written".
- The shared-bound requirement claims only the shape its reaction observes, and the residual is stated
  where a register reader sees it.

**Non-Goals:**

- Detecting a restatement that cites two different tests. This is not deferred work; it is out of reach.
  See the decision below.
- Checking which *section* of a tracker document owns an unpinned bound.
- Requiring anything of a pinning test's name.
- Any crate behaviour change. Every edit is to a repository gate, its matrix, its spec, and the generated
  projection.

## Decisions

**1. Test-ness is read from the attribute run, not from the preceding line.**

The obvious check — "the line above the definition is `#[test]`" — is wrong in this tree, and the evidence
is three sites where `#[should_panic]` sits between the attribute and the `fn`
(`crates/louke/tests/install_rejects_duplicates.rs:8`, and `crates/hunyi/src/finding/fact.rs:1698`). None
of those is cited today, so the naive check would pass the matrix and refuse a real test the first time
someone pinned a bound with a panicking test. The reaction therefore walks upward from the definition
while the line is an attribute, a comment, or blank, and looks for `#[test]` in that run.

Alternative considered: `cargo test --list`. Rejected — it needs a compiled workspace, which turns a
text-scanning gate that runs on a fixture repository into one that cannot. The whole failure matrix is
built on throwaway repositories with one `lib.rs` and no `Cargo.toml`.

Alternative considered: requiring the definition to sit under `#[cfg(test)]` or in `tests/`. Rejected —
it is a weaker signal (a helper inside `mod tests` would pass) and a coarser one.

**2. The tracker check is "names a tracked path", nothing more.**

"Names an owner" has a checkable part and an unreadable part. The checkable part is that some token in the
citation is a path the repository tracks: `git ls-files` answers it as a fact. The unreadable part is
whether the named section still describes the debt — prose. The reaction takes the first and refuses the
second, and the spec says so, because a gate that guesses at the second would produce the false positives
that get gates disabled.

This also gives the direction its second refusal for free: a tracker naming `NOSUCH.md` fails, which is
the same class as a `PINNED-BY` naming a deleted test — a citation pointing at something unreadable.

**3. Regeneration writes, then judges. Cannot-judge precedes writing.**

Three orderings were possible:

- *Render to a temp file, judge, only then move it into place.* Rejected: it conflates two failures the
  author needs to tell apart. If the register is invalid, the author's next move is to fix the register —
  and having the regenerated projection on disk is how they see what the register now says. Withholding it
  makes the failure harder to repair, not safer.
- *Write and exit 0* (the current behaviour). Rejected: exit 0 is the family's "clean", and CI's green is
  built on that meaning.
- *Write, then fall into the same verdict.* Chosen. One code path produces the verdict, so regeneration
  and judgment cannot disagree about what an offense is.

The `cannot judge` condition is the exception, and it moves **ahead** of writing: with zero declared bounds
parsed, the heading form has changed and the register is not there to be projected. Writing a `0 of 0`
document first would leave behind a projection that reads as a complete register of a repository with no
bounds — the flattering direction this gate's subject makes easy.

**4. The shared-bound residual is stated, not declared as a bound.**

The reviewer's first proposed correction was to introduce an observable shared bound identity. It was
rejected on evidence rather than on principle: `semantic-dyn-trait-operand-boundary` and
`semantic-impl-trait-operand-boundary` both declare `A genuinely unresolvable bare principal is a
documented bound` — identical heading, distinct `WHEN` (`dyn Frobnicate` versus `impl Frobnicate`),
distinct pinning tests, because the code paths differ. Any key over heading text or statement similarity
fires on that pair, and the only repair it would accept is dissolving a symmetry `三儀 ⊥ 三儀` requires.
A gate whose first live finding is a false positive over a constitutional requirement is worse than the
gap it closes.

So the claim narrows to what is observed, and the residual joins the undeclared-prose floor in the
projection's header. It is **not** declared as a bound of this capability, following the rule this
capability already set for its own prose floor: a declaration nothing can observe is the
name-without-a-reaction `PROJECT.md` forbids, and the register must not make itself the exception.

The record of the two historical restatements is corrected in the same edit, because it was doing the
overclaiming: the `#[path]`-remap bound was **prose** in `external-crate-confinement` and a **scenario** in
`inline-symbol-path-confinement`, so the undeclared-prose direction is what reaches that shape — this
requirement's own direction never would have.

## Risks / Trade-offs

- **[The `#[test]` check refuses a legitimately-cited test whose attribute form this tree does not yet
  contain]** → the tree carries no `#[tokio::test]`, `#[rstest]`, or other test macro (verified by grep), so
  the check's blind spot is empty today. It fails **loudly** rather than silently if that changes: the
  citation is refused, which is a false positive an author sees immediately, never a false negative. The
  matrix pins both the `#[should_panic]` interleaving and the plain-function refusal so the direction of
  the error is fixed by a fixture.
- **[The tracker check refuses a legitimate tracker written without a path]** → all five citations today
  name `BACKLOG.md`; a tracker with no document is the anonymous debt the requirement already forbids, so
  there is no legitimate form being refused.
- **[Regeneration now exits nonzero in a state where authors were used to 0]** → the projection is still
  written, and the message names the offense and the remedy. The failure matrix's fixture helper stops
  asserting `exit 0` from blessing and asserts the projection **exists** instead, which is what it actually
  needed.
- **[The narrowed shared-bound claim reads as a weaker register]** → it is the same register with an honest
  header. The alternative on offer was a wider claim than the mechanism, which is the precise failure
  (`#[path]`, twice) that motivated building this capability.

## Migration Plan

None. No published artifact, crate API, or adopter-visible behaviour changes; both tightened directions are
already satisfied by every citation in the tree. `docs/observation-bounds.md` is regenerated in the same
change so the projection cannot land stale.

## Open Questions

None.
