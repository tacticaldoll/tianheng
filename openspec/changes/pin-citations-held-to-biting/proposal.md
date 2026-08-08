## Why

`observation-bound-register` decides that a `PINNED-BY` citation names a test that **runs** — resolved to one
`fn` under `crates/`, carrying `#[test]`, registered by the harness. It does not decide that the test **bites**.
A pin whose assertions are deleted, or whose subject is loosened back toward the rule it was written to refuse,
keeps running, keeps being registered, and keeps reading as coverage.

That is not hypothetical. In this window the retirement of the composition-body reaction deleted the only
assertions over `anchor_count` and `decline_reason`; the suite stayed green, and the rule they defend —
counting occurrences rather than trimmed-start lines — could be reverted with nothing refusing. It was found by
a reviewer reading the diff, which is the instrument this register exists to replace.

`AGENTS.md` already states the obligation — *"a guard is not a guard until it has been seen to fail"* — and
enforces it by asking an author to record the negative run in a PR body. Prose carrying a checkable property is
the drift class this repository has closed twice already, one level down each time.

## What Changes

- A **mutation** may be declared against a pinning citation: a tracked file, a `from` substring, and a `to`
  substring that perturbs the reaction at the point the citation is about.
- A new gate applies each declared mutation to a scratch tree built from **tracked content**, runs only the
  cited test, and requires it to **fail**. A test that survives its mutation is a citation occupying the place
  of a defence.
- Coverage is partial by construction and says so: the gate prints how many citations carry no mutation, in the
  same shape `docs/observation-bounds.md` already leads with its unpinned count. A register that reported the
  covered ones and stayed silent about the rest would be the reads-as-coverage failure one level up.

Not **BREAKING**. No crate, public surface, manifest, or package version is touched; this is a repository gate
over this repository's own governance tests.

## Capabilities

### New Capabilities

None. `observation-bound-register` already owns the question *what defends this bound*; this change makes the
register's answer observable instead of nominal.

### Modified Capabilities

- `observation-bound-register`: one requirement added — a pinning citation may declare the mutation it dies
  under, every declared mutation must kill its citation, and the uncovered remainder is disclosed rather than
  implied.

## Impact

- `scripts/check_pin_bites.sh` — the new gate; enters `gate-shape-contract`'s surface the moment it is tracked.
- `scripts/test_pin_bites.sh` — its twin, holding the five matrix properties that surface requires.
- `scripts/lib/pin_mutations.tsv` — the declared mutations, cross-checked against the register's citations in
  both directions so a renamed test breaks loudly rather than silently losing its mutation.
- `openspec/specs/observation-bound-register/spec.md` — the added requirement, at sync.
- `AGENTS.md` — the new gate joins the Definition of Done list; `check_dod_coherence.sh` requires CI to run it
  too, so `.github/workflows/ci.yml` gains it.
- `CHANGELOG.md` — an `[Unreleased]` entry.
- `BACKLOG.md` — the entry recording the uncovered remainder and what closing it costs.

Two measurements taken while exploring belong in the design, because each rules out an implementation that
looks obviously right:

- Reusing the repository's own `target/` for a scratch tree **silently reports every pin as biting**. Cargo
  resolved the fingerprint against the original worktree's sources, so a mutated scratch tree ran a binary built
  from unmutated code and finished in 0.01s claiming `Finished`. The gate therefore owns an isolated target
  directory, and pays 5.2s to warm it once rather than accepting a false clean.
- A mutation that fails to perturb the pinned point makes the gate report a pin that does bite as one that does
  not. That is the safe direction — a false alarm an author resolves by writing a better mutation, never a
  silent pass — and it is why the requirement is written about the *declared* mutations rather than about a
  coverage percentage.
