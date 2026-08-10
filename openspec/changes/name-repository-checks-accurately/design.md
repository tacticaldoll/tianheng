# Design

## Context

The repository already enforces the structural product boundary: only publishable crates are product, Tianheng has no repository-bound catalog, Kanhe and Shengmo are `publish = false`, and shell wrappers carry no verdict. Live prose still uses “reaction” for Kanhe tests, CI jobs, shell gates, and the `rust-repository-reactions` capability. That vocabulary contradicts the structure and has already caused task-scope drift toward strengthening `.sh` as though it implemented product behavior.

## Goals / Non-Goals

**Goals:**

- Give the repository-check capability an identity matching its actual owner and subject.
- Make high-authority prose teach one four-way vocabulary consistently.
- Preserve product use of reaction and Shengmo dogfooding of real product reactions.
- Keep every repository check and workflow behavior unchanged.

**Non-Goals:**

- Rename product reaction types, rules, outcomes, reports, or product capability specs.
- Remove every lexical occurrence of “reaction” from Kanhe or Shengmo; those crates legitimately discuss product reactions they inspect or execute.
- Add a Tianheng boundary over prose. The forge-law audit classified this as API explanation/process vocabulary, not an observable Rust architecture fact.
- Add a general prose detector.

## Decisions

### Use four terms by owner and effect

- **Product**: crates whose manifests permit publication.
- **Reaction**: observable product behavior from those crates—observation, structured outcome/report, exit class, or runtime event.
- **Gate/check**: an unpublished Rust test or judgement that verifies this repository, including a Shengmo gate that invokes product reactions for dogfood.
- **Workflow/orchestration**: shell and CI sequencing that invokes gates or irreversible commands and owns no verdict.

The term is selected by what produces the verdict, not by file extension alone. A shell wrapper may invoke a Rust gate; it does not thereby become a reaction.

### Rename the capability and its ids atomically

The main spec directory moves to `repository-checks`, its title changes, and every `rust-repository-reactions/` bound id moves to `repository-checks/`. The declarations' extent, owner, reason, and defence stay unchanged except where a reason incorrectly calls the Kanhe check a reaction. Generated projections are regenerated from the renamed declarations.

### Correct authoritative and self-descriptive prose, preserve history

Live contracts, contributor instructions, crate READMEs, generator templates, workflow comments, and source comments describing their own repository gate are corrected. Dated changelog/history statements remain records unless they make a current unanchored claim. Product specifications retain reaction vocabulary.

### Verify with existing joins and a retired-term sweep

Capability-subject and bound-model gates hold the rename structurally. Projection freshness holds generated text. A whole-file grep over every touched live document/spec/source refuses the retired capability term, satisfying the repository's vocabulary-identity review rule without creating a general prose-intent detector.

## Risks / Trade-offs

- Some comments require judgement because Kanhe legitimately discusses product reactions. Review must inspect each changed sentence rather than mass-replace the token.
- Renamed bound ids are repository identities. Their change is intentional but still requires generated projections and citations to move together.
- Historical records may retain the old term, so a repository-wide zero-match rule would falsify provenance. The sweep is scoped to live surfaces and every touched file is read whole.

## Verification

- Before rename, a focused retired-term sweep names the old spec, ids, fixtures, and generated projections.
- Capability-subject tests fail if the renamed spec and proposal accounting diverge.
- Bound register/model tests fail if any renamed id, citation, extent, or generated projection is stale.
- Projection-register freshness fails before regenerated “checks holding them” text is committed.
- Full-file review confirms changed `reaction` uses are product-facing and changed `check/gate/workflow` uses are repository-facing.
- Complete Definition of Done passes.
