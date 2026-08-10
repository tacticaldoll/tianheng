# Change: Name repository checks accurately

## Why

Tianheng uses **reaction** for the observable product behavior emitted by publishable crates. Kanhe's tests instead compare this repository's records, and shell/CI files only orchestrate those checks. Calling all three “reactions” collapses the product boundary the repository now enforces structurally and causes agents to assign product semantics to `.sh`, CI, and unpublished checks.

The remaining strongest source of drift is the capability identity `rust-repository-reactions` itself. Even after its declarations moved to Kanhe, the name and its live prose continue teaching the retired classification.

## What Changes

- Rename `rust-repository-reactions` to `repository-checks`, including the spec path, bound ids, citations, fixtures, and generated projections.
- Define the live vocabulary: product means publishable crates; reaction belongs to their observable boundary behavior; Shengmo gates dogfood those reactions; Kanhe provides repository checks; shell and CI orchestrate gates and carry no verdict.
- Correct `PROJECT.md`, `AGENTS.md`, Kanhe/Shengmo READMEs, workflow/script comments, and repository-check source comments that classify their own gate as a reaction.
- Keep product specifications and comments using reaction where they actually describe product observation, verdict, report, exit, or runtime behavior.
- Regenerate projection-register text so it names the checks holding projections fresh rather than “reactions”.

## Capabilities

### New Capabilities

- `repository-checks`: the renamed repository-only capability owns Rust checks over repository records and the vocabulary boundary separating them from product reactions and workflow orchestration.

### Retired Capabilities

- `rust-repository-reactions`: retired as a misleading identity; every requirement and declared bound moves to `repository-checks` without behavioral change.

### Modified Capabilities

- `projection-register`: generated prose calls its Rust freshness holders checks, while correspondence and freshness behavior remain unchanged.

### Existing Capabilities

- `observation-bound-model`: requirements do not change; renamed repository bound ids remain in the same combined bijection.
- `governance-dogfood`: behavior does not change; Shengmo prose distinguishes its gates from the product reactions they execute.
- `self-law-projection`: law and projection content do not change; surrounding prose names the self-governance test as a gate.
- `release-coherence`: behavior does not change; its AGENTS/CI/CHANGELOG subjects receive vocabulary corrections.
- `publish-source-integrity`: behavior does not change; the shell wrapper is orchestration and its Rust judgement remains a repository gate.
- `adopter-surface`: product reaction vocabulary remains unchanged and product still means crates that can be published.

## Impact

- Changes unpublished repository capability and bound ids from `rust-repository-reactions/*` to `repository-checks/*`.
- Changes no product API, evaluator, report, exit class, package manifest, or architecture law.
- Requires no released adopter action; Kanhe and Shengmo are unpublished and the renamed ids are repository records.
