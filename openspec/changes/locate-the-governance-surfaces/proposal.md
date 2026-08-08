## Why

`PROJECT.md` names three governance surfaces — 垂象 (the reaction surface), 實錄 (the baseline), 校讎 (the
amendment flow) — and then leaves their shape an open question:

> Both are **crate-or-convention as their nature dictates**, never named before their reaction exists.

That sentence is why a reader asks whether one of them should become a crate. It also gives no referent, so the
three read as three brandable things rather than as surfaces that already exist somewhere. The self-law
preamble every agent loads says only "三司 administer", which is the same absence in the file with the widest
reach.

Measured, none of the three has a single home, and that is the answer:

| surface | where it is |
|---|---|
| 垂象 | `crates/guibiao/src/projection.rs` assembles the report and constitution documents; `crates/tianheng/src/runner/render.rs` renders text and SARIF; `crates/xuanji` serializes a `Violation` |
| 實錄 | `crates/xuanji/src/baseline.rs` holds the model, and `guibiao`, `hunyi` and `tianheng` all consume it |
| 校讎 | `.github/CODEOWNERS` routes an amendment to the steward, `AGENTS.md` owns the OpenSpec lifecycle, and `crates/tianheng/src/constitution.rs` names the routing in shipped source |

**A crate is a boundary. None of these three has one to be** — each crosses every crate it touches, and one of
them lives outside `crates/` entirely. That is a stronger reason than "as their nature dictates", and unlike
that sentence it is checkable by reading the tree.

## What Changes

- `PROJECT.md` replaces the open question with where each surface is, and with the reason the asymmetry against
  三儀 exists: a dimension must never learn from a sibling, so each needs a boundary the self-law reacts to and
  a crate is that boundary; a surface that spans crates has no boundary to be, so a crate would name nothing.
- The self-law preamble names each surface **by what it does** rather than as a set, so an agent loading it
  meets three functions instead of three handles.
- `self-law-projection`'s requirement, which prescribes the preamble's content, says the preamble introduces no
  governance name without saying what it does.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `self-law-projection`: the preamble requirement gains the no-bare-handle rule and drops the set-shaped
  phrasing.

## Impact

- `PROJECT.md` — the observatory-vocabulary paragraph.
- `crates/tianheng/tests/self_governance.rs` — the preamble line.
- `AGENTS.self-law.md` — regenerated, never hand-edited.
- `openspec/specs/self-law-projection/spec.md` — the modified requirement, at sync.
- `CHANGELOG.md` — an `[Unreleased]` entry.

## What an earlier attempt got wrong, recorded so it is not retried

This change was first written as a **retirement** of 校讎, on the ground that it had "no code, no capability and
no reaction". Review falsified two thirds of that: `.github/CODEOWNERS` calls itself "the amendment reaction",
`crates/tianheng/src/constitution.rs` names it in shipped source, and `self-law-projection`'s own spec governs
"Layer 3 … governed by `CODEOWNERS`". The grep that produced the premise swept `crates/`, `docs/` and
`AGENTS.self-law.md` and never swept `.github/` — which is exactly where the referent lives, and where a
back-reference points at the very paragraph the retirement rewrote.

What survives of it is narrow: CODEOWNERS designation is advisory without branch protection, which `BACKLOG.md`
already records. That is a reason to describe the surface accurately, not to delete its name.
