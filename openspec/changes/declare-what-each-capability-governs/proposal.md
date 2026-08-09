## Why

**Which capability a requirement belongs to is chosen in a proposal and checked by nothing.** A wrong choice
survives review, sync, and archive — and then teaches the next reader.

It has already happened here twice in one window. A requirement about what `scripts/publish.sh` must do before
`cargo publish` was filed under a capability whose own Purpose says `publish.sh` "is a wrapper rather than a
gate"; and the member built to hold this repository's governance was first filled using one criterion — does
it ship — which says where a reaction must *not* live and nothing about where it belongs. Both were caught by
a reader, not by a reaction.

**The capability's name still describes the wrong population.** `rust-self-governance-gates` holds the
requirements governing this repository's own reactions. Self-governance is Tianheng governing itself **with
the capability it ships** — what `crates/shengmo/` now holds — while the reactions this capability actually
governs mostly collate a record and reach no product contract at all. The name was the reason a governance
document could describe twenty of them as running Tianheng's own reactions against the workspace.

## What Changes

- **`rust-self-governance-gates` becomes `rust-repository-reactions`.** The existing requirement set moves
  **verbatim**, so review reads a rename rather than a rewrite. The `rust-` prefix carries weight: the subject
  is Rust reactions, which is exactly the constraint the misfiling broke.
- **Every capability declares a `## Subject`**: the tracked-path globs it governs, resolved by
  `git ls-files -- <glob>` so membership is a produced set rather than a text model of one.
- **A reaction joins a change's produced diff to what its proposal claims.** For each active change, the files
  it actually touches come from `git diff`; a touched file owned by some capability's subject requires that
  capability to be named in the proposal's Capabilities section. The filing decision is checked where it is
  made.
- **The two governance members' identities are declared, and the absence of a mechanical discriminator is
  declared with them.** Whether a reaction belongs to 繩墨 or 勘合 is a judgement about what it judges;
  position is the declaration, and two attempts at a mechanical rule were each measured unreliable.

## Capabilities

### New Capabilities

None. One capability is renamed and two gain requirements.

### Modified Capabilities

- `rust-self-governance-gates` → **renamed** `rust-repository-reactions`: requirement set moved verbatim, plus
  the requirement that a capability declares its subject and that a change's proposal is joined to it.
- `observation-bound-register`: the new bound — that no mechanical rule separates the two governance members —
  is declared where declared bounds live.

Two further capabilities' subjects are touched and their requirements are **not** changing, which is stated
here rather than left for a reader to infer — the join below requires a proposal to account for every
capability whose subject it touches, by listing it or by saying why it is not listed:

- `observation-bound-model` claims `crates/*/src/bounds.rs` and `docs/observation-bound-extents.md`. Both
  carry bound ids derived from the renamed capability, so their **content** moves while the requirement that
  an id is derived from its capability stays exactly as it is — that requirement is why they move at all.
- `release-coherence` claims `CHANGELOG.md`, which records the rename under `[Unreleased]`. Recording a
  change is what that capability requires; nothing about the requirement changes.

## Impact

- `openspec/specs/rust-self-governance-gates/` → `openspec/specs/rust-repository-reactions/`; every capability
  spec gains a `## Subject` section.
- Four published bound ids beginning `rust-self-governance-gates/` change value. They are exported through
  `pub use bounds::observation_bounds` and were published in 0.4.0, so this is data an adopter can observe.
  Keeping the old strings was considered and rejected: the register requires a bound id to be **derived**,
  `<capability>/<scenario-slug>`, and pinning them would convert every one into an assignment.
- `crates/kanhe/` — one new reaction with its failure matrix; its refusal sites join the enumeration
  `refusal_bites` perturbs.
- No public API changes. No version change.
