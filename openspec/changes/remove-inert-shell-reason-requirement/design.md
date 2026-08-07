## Context

The projected shell dependency reason was corrected from an allowlist restatement to bounded architectural direction. That correction is valid Layer 2 gravity, but the accompanying spec requirement claims two properties that the repository does not react to: prose quality and outcome stability when only diagnostic text changes. Projection freshness compares the generated artifact with its generator and therefore cannot judge either property.

## Goals / Non-Goals

**Goals:**

- Remove the false impression that shell-specific reason wording has an automated backstop.
- Preserve the corrected live reason, its generated projection, and the general forward-reason policy.
- Keep specification scenarios limited to behavior with an observation capable of refusing.

**Non-Goals:**

- Change the `tianheng` dependency boundary, allowlist, reason text, or projection renderer.
- Add a lexical policy detector for reason prose.
- Address the separate observer delegation scenario or the wider un-reacted-`SHALL` backlog item.

## Decisions

- Remove the complete shell-specific requirement rather than weakening its wording. Recasting the second scenario as a construction note would still add no reaction, while its useful fact already follows from `reason` being diagnostic metadata rather than identity or verdict input.
- Leave the corrected reason in `self_governance.rs`. Layer 2 prose can guide continuations without pretending to be a Layer 1 reaction, and the existing freshness test continues to ensure the checked-in projection matches that source.
- Do not add a crate-name lexical assertion. Such a check would enforce one wording tactic rather than the observable dependency boundary and would create a second allowlist-shaped source to maintain.

## Risks / Trade-offs

- **A future edit can make the reason repetitive again** → Adversarial law review remains the honest control for prose quality; the dependency boundary still reacts to the actual architecture.
- **Removing normative text can look like weakening law** → The product law, generated projection, and all reactions remain byte-for-byte unchanged; only an unobservable claim about wording is retired.
