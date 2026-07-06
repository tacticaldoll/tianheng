# self-law-projection Specification

## Purpose

Put Tianheng's **own enforced self-law** into an agent's context as a faithful, imitable,
staleness-checked Markdown artifact. The published binary's `list` projects a *demo*
constitution, so an agent working on this repo never naturally sees the self-law that
actually reacts (`tianheng_constitution()` in `self_governance.rs`). This capability closes
that entry-point gap — the first dogfood of the 潛移 (gravity) face (see `PROJECT.md`): the
declared law, rendered where an agent reads it, so its continuations imitate the real shape
rather than the demo. Two contracts are kept distinct: the **repo artifact** must not drift
from the enforced law (a test reacts), and the **public renderer's Markdown layout** is a
human/agent surface that may evolve (a doc contract, never frozen — JSON remains the machine
contract).
## Requirements
### Requirement: Self-law projection is generated from the enforced self-constitution

The repository SHALL carry an agent-readable Markdown artifact projecting Tianheng's self-governance law. The projection SHALL be derived from the **same** constitution object the self-governance gate reacts against (`tianheng_constitution()`), never a hand-written restatement, so the projected law and the enforced law cannot diverge into two sources of truth. The projection SHALL cover every boundary the self-constitution declares, each with its target, rule, and declared `reason`.

#### Scenario: The projection carries the enforced self-law

- **WHEN** the self-law projection is generated
- **THEN** it contains every boundary `tianheng_constitution()` declares — each crate boundary with its rule and declared `reason`, including 圭表's two (its dependency allowlist and its forbid-dependency-on-shell)

#### Scenario: Adding a boundary to the self-law changes the projection

- **WHEN** a boundary is added to or amended in `tianheng_constitution()` and the projection is regenerated
- **THEN** the regenerated projection reflects that boundary, because it is rendered from that same object

### Requirement: A staleness test reacts when the checked-in projection drifts

A test SHALL fail when the checked-in projection artifact differs, byte for byte, from the live projection generated from `tianheng_constitution()`. The comparison SHALL cover the **entire** artifact — both the generated boundary projection and any fixed preamble (the preamble being a generated constant, never hand-edited prose) — so no part of the artifact can drift unnoticed. The test SHALL follow the repository's existing repo-only discipline: it SHALL skip when run outside a workspace checkout (e.g. a packaged crate tarball), and SHALL fail loudly rather than skip when a workspace is expected but absent (the `TIANHENG_WORKSPACE_TESTS` signal). A one-command regeneration path SHALL overwrite the checked-in artifact from the live projection instead of asserting.

The byte-check reaction itself SHALL be a **reusable public helper** so an adopter can gate their own projected constitution with the same mechanism (the 潛移 adoption face) rather than hand-rolling it: given the **live** projection string, the artifact **path**, a **regenerate** command string, and a **bless** flag, the helper SHALL — when `bless` is true — overwrite the file with the live projection (creating any missing parent directories) and succeed; otherwise it SHALL compare the checked-in file to the live projection and **fail** when they differ, when the file is **missing**, or when it is **unreadable**, returning an actionable error that names **both the artifact path and the regenerate command**. A write failure under `bless`, or a read failure otherwise, SHALL be returned as an error, never a silent success. The helper SHALL NOT itself read the environment — the **caller** supplies `bless` (so the helper is a pure function of its arguments, with no process-global env dependency and no parallel-test hazard); Tianheng's own self-law staleness test reads its `BLESS` signal and passes it in, and is one caller of this helper.

#### Scenario: A stale checked-in projection fails the test

- **WHEN** the checked-in projection artifact no longer matches the live projection of `tianheng_constitution()`
- **THEN** the staleness test fails, naming the artifact and instructing to regenerate it

#### Scenario: Regeneration refreshes the artifact instead of asserting

- **WHEN** the regeneration signal is set and the staleness test runs
- **THEN** the checked-in artifact is overwritten with the live projection and the test does not assert staleness

#### Scenario: The test skips outside a checkout but fails loud when a workspace is expected

- **WHEN** the test runs where no workspace root is present
- **THEN** it skips if no workspace is expected, but fails loudly if `TIANHENG_WORKSPACE_TESTS` declares a workspace must be present

#### Scenario: The reusable gate helper reacts to drift, missing, and unreadable artifacts

- **WHEN** the gate helper is called with `bless = false` and the checked-in file differs from the live projection, or does not exist, or cannot be read
- **THEN** it returns an error naming the artifact path and the regenerate command, so the caller's `cargo test` reacts — a missing or unreadable projection is "cannot confirm fresh", never a silent pass — and returns success only when the file byte-matches the live projection

#### Scenario: The gate helper regenerates on bless, creating parent directories

- **WHEN** the gate helper is called with `bless = true`
- **THEN** it overwrites the artifact with the live projection (creating any missing parent directories) and succeeds, returning an error if the write itself fails — the caller supplies `bless`, so the helper touches no process-global environment

### Requirement: The Markdown projection is a human/agent-readable surface, not a machine-stable contract

The constitution-to-Markdown projection SHALL be produced by the **same renderer** as `list --format markdown` and SHALL add nothing of its own (no preamble, no trailing newline), so the agent-loaded artifact and the CLI projection cannot diverge. The public rendering helper SHALL document that its Markdown layout is intended for display, review, and LLM context, and **MAY evolve in any compatible release** to improve readability or imitability; consumers needing a stable, machine-parseable contract SHALL use the JSON projection instead. No automated test SHALL pin the helper's exact Markdown layout as a contract — that absence is deliberate, so evolving the layout (e.g. foregrounding the `reason`) is not a breaking change to a machine consumer. (The evolvability clause is held by the doc-comment, verified by review, not by an automated assertion — see design.md, Contract B.)

#### Scenario: The helper renders the same projection as the CLI, byte for byte

- **WHEN** a constitution is rendered through the public Markdown helper
- **THEN** the output equals, byte for byte, what the `list --format markdown` path projects for that same constitution — the helper prepends and appends nothing (one renderer, no parallel projection path)

#### Scenario: The Markdown format is documented as evolvable (review-verified)

- **WHEN** a reviewer reads the public Markdown helper's doc-comment
- **THEN** it states the layout is human/agent-readable and may evolve, and directs machine consumers to the JSON projection — and no golden/snapshot test fixes the helper's exact output as a contract

### Requirement: The preamble describes only how to read the projection, not crate-specific law

The artifact's fixed preamble SHALL describe only how to read and use the projection and the reaction loop it serves (declare intent in code; observe only what has an observation source; react with the 0/1/2 outcomes; repair toward the declared `reason`; never weaken the law to pass; 三儀 measure, 三司 administer). The preamble SHALL NOT make crate-specific architectural claims; any such claim SHALL appear only in the generated projection, where it traces to a boundary that actually reacts (no open-loop prose prescription).

#### Scenario: Crate-specific law appears only in the generated projection

- **WHEN** the preamble is read
- **THEN** it describes the reaction loop and how to read the projection, and makes no crate-specific architectural claim — such claims appear only in the generated boundary projection below it

