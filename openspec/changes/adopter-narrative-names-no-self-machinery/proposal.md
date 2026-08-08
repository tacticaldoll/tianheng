# Adopter narrative names no self-governance machinery

## Why

`scripts/` ships in **zero** packages:

```
cargo package -p <every member> --list  →  0 files under scripts/
```

It is self-governance: 26 files, 3,793 non-comment lines, judging *this repository* and reaching no
adopter. Yet it is priced as product at four surfaces:

| surface | measured |
|---|---|
| `CHANGELOG.md` | **11 entries** whose subject is a gate, under `### Added` / `### Changed` / `### Fixed` — every heading an adopter's vocabulary, and no heading that is not |
| `openspec/specs` | **4 of 36** capabilities (1,291 lines, 16%) have their subject inside this repository, in the same directory and lifecycle as the 32 that describe what adopters get |
| twin obligation | 2,171 lines of `test_*.sh`, required by `gate-shape-contract` for something that ships to nobody |
| Definition of Done | every gate behind the same wall as the product suite |

An adopter reading `[Unreleased]` reads about `check_pin_bites.sh`, a file they can never run.

The cost is the coupling this creates. Across the 134 commits of the 0.5.0 window, **51 touched
`scripts/`, and 40 of those 51 (78%) were forced to also touch `openspec/` or `CHANGELOG.md`.** A
change to a script that ships to nobody is not a script edit — it is a four-file transaction, and
prose is where this window's defects have overwhelmingly been.

## What this change does NOT claim

**Self-governance deserves full rigour.** Dogfooding is this project's thesis; `governance-dogfood` is
a capability and 潛移 is the design principle. Not one reaction, twin, observation bound or exit-contract
obligation is removed or weakened here.

What self-governance does not deserve is **adopter-facing publication**. Rigour and publication have
been fused, and only publication is wrong. The distinction is testable: deleting an adopter-facing
CHANGELOG entry for `check_pin_bites.sh` changes nothing about that gate's rigour.

## The first rule drafted here was falsified before it was written down

The draft was *an adopter-facing entry SHALL NOT cite a path that ships in no package*, on the reasoning
that "ships" is what separates adopter surface from housekeeping. Enumerating what `[Unreleased]`
actually cites killed it:

```
15 distinct repository paths cited — 15 ship in no package, 0 ship
  COOKBOOK.md · docs/observation-bounds.md · docs/gate-shape-contract.md · Cargo.toml
  AGENTS.md · BACKLOG.md · PROJECT.md · scripts/publish.sh · scripts/lib/capture.sh · …
```

`COOKBOOK.md` and `docs/*.md` are documentation an adopter reads on the repository page; they ship in no
package and are adopter surface anyway. **"Ships in a package" is not a proxy for "the adopter can see
it"**, and a rule built on that proxy would have fired on all fifteen. It is recorded here so the next
author does not re-derive it.

## What changes

An entry under an adopter-facing heading in `[Unreleased]` SHALL NOT name **this repository's own
machinery** — a path under `scripts/`, or the bare basename of a tracked file there. That is not a
proxy: `scripts/` is the executable machinery of this repository's own Definition of Done, which is
already how `gate-shape-contract` enumerates the gate surface. Entries whose subject is that machinery
go under a `### Self-governance` heading, where naming it is exactly what belongs.

The rule sits on the **decidable** side of the line `release-coherence` already draws for itself —
grammar and references are decidable, claims are not. A path citation is a reference, and
`check_reference_integrity.sh` already resolves references over `CHANGELOG.md` mechanically.

Measured on the current `[Unreleased]`: the rule reaches **9 entries** and leaves the **11**
`COOKBOOK.md` / `docs/*.md` citations untouched, which is the discriminator working.

**Dated sections are record, not subject.** Five of the eleven entries sit in the released `[0.4.0]`;
rewriting them would falsify what was true at 0.4.0, exactly as `docs/history/` is left alone. The
requirement binds `[Unreleased]`, and a section becomes record by being dated.

## What the rule forces, measured on a real entry

`CHANGELOG.md:134` reads:

> The crates.io publish now runs through a source gate (`scripts/check_publish_source.sh`, reached via
> `scripts/publish.sh`) that refuses any source other than the signed-and-annotated-tagged
> `release: X.Y.Z` commit at the live tip of `main`.

This looks like a false positive, because the adopter-relevant fact is genuinely present: the provenance
of what they install. It is not one. The rule forces that entry to be **rewritten in adopter terms** —
state the guarantee, drop the filenames — rather than moved. If a fact matters to an adopter, state the
fact; if it can only be stated by naming a file they will never see, it was not an adopter fact.

## What the rule does not reach

An entry whose subject is this repository's own governance but which names no path under `scripts/` is
invisible to it. That residual is a **judgement over the entry's subject**, which is the instrument
`AGENTS.md` records as designed, measured three times and rejected. It is declared as an observation
bound rather than approximated.

## Out of scope

The 4 capabilities whose subject is this repository (`gate-shape-contract`,
`observation-bound-register`, `projection-register`, `self-law-projection`) are the same mispricing at a
second surface. They are not touched here — that is a separate change, filed in `BACKLOG.md`, so this
one stays closable.
