# self-law-projection

## ADDED Requirements

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
- **THEN** it says what that surface does, and does not present the three as a set with a common shape
- **UNPINNED** `BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*
