## Why

`PROJECT.md` names three governance offices — 三司: 垂象 (the reaction surface), 實錄 (the baseline), 校讎 (the
amendment flow) — and closes the paragraph with a rule of its own:

> …never named before their reaction exists.

Measured against that rule, one of the three fails it. 垂象 has `crates/tianheng/src/runner/render.rs` and the
report surfaces; 實錄 has `crates/xuanji/src/baseline.rs` and the `violation-baseline` capability. **校讎 has no
code, no capability, and no reaction** — it names the amendment flow, which exists as process. Its only uses in
the tree are as an adjective: two doc comments calling a reaction "潛移/校讎-adjacent".

The same paragraph carries a second sentence that is false as measured:

> Both are crate-or-convention as their nature dictates.

That reads as an open question, and it is the sentence that keeps 校讎 alive as a crate candidate. The
measurement answers it: 垂象 is a module of the facade, 實錄 is in the kernel — where it must be, since every
dimension baselines its verdicts.

## What Changes

- **校讎 is retired as a name.** The self-law preamble line loses it, its two adjectival uses are reworded to
  name what they actually describe, and `PROJECT.md` stops listing it as an office. What it referred to — the
  amendment flow — is the OpenSpec lifecycle, which `AGENTS.md` already owns and names plainly.
- **`crate-or-convention as their nature dictates` is replaced by where the two survivors are.** 三儀 are
  orthogonal, so each needs a boundary the self-law can react to, so each is a crate. The governance surfaces
  are not orthogonal — a record is rendered, a rendering reads a record — so a boundary between them would be
  one crossed immediately, which is a name with no reaction. The asymmetry follows from the topology, not from
  importance.
- `self-law-projection`'s requirement prescribes the preamble's content, including the retired name, so this is
  a **spec change** rather than a documentation edit.

`docs/history/0.1.0-0.3.0-built-ledger.md` records 三司 as it stood at 0.1.0–0.3.0 and is **not** edited: it is
a record of what was true then, and changing it would falsify the record rather than update it.

Not **BREAKING**. No crate, public surface, manifest, or package version is touched; `AGENTS.self-law.md` is a
generated projection of this repository's own law.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `self-law-projection`: the requirement fixing the preamble's content drops the retired name and states what
  the surviving two are, so the preamble stops carrying a name with no referent.

## Impact

- `crates/tianheng/tests/self_governance.rs` — the preamble line and two adjectival uses.
- `AGENTS.self-law.md` — regenerated, never hand-edited.
- `PROJECT.md` — the observatory-vocabulary paragraph.
- `openspec/specs/self-law-projection/spec.md` — the modified requirement, at sync.
- `CHANGELOG.md` — an `[Unreleased]` entry.

## What this does not do

It does not decide that the amendment flow needs no register. `BACKLOG.md` carries a WATCH entry for rejected
observation points reaching no agent, with its trigger recorded as **not fired**; retiring the name it was
filed under changes neither the entry nor the trigger. What retiring removes is a handle with no referent —
the thing that made a reader ask whether it should become a crate.
