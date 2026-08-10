# Design

## Context

`kanhe::refusal` started as one shared typed result for repository judgements. It later gained caller-location recording and runtime mutation. Supporting that instrumentation added `refusal_sites`, `refusal_exemptions`, `refusal_bites`, a census, workflow wiring, and product-visible observation-bound declarations.

The focused gate matrices already assert the result kind and actionable message for their externally meaningful shapes. The sweep instead treats every constructor call as an independent contract, including internal propagation branches and machine failures no fixture can schedule. That is mutation coverage over an implementation detail, not a product or repository boundary.

## Goals / Non-Goals

**Goals:**

- Restore `Refusal` to a pure repository-check result.
- Remove support code whose only consumer is constructor-site mutation.
- Keep disagreement and unverifiable inputs structurally distinct.
- Remove repository-only declarations from the published product catalog and prevent their return.

**Non-Goals:**

- Rename the result vocabulary; reaction taxonomy prose is a later independent change.
- Reduce focused gate behavior matrices.
- Change any product evaluator, exit code, report, or architecture law.
- Generalize a heuristic that decides which future bound is "repository-only".

## Decisions

### Retain the typed result, delete its instrumentation

`Kind`, `Refusal`, `violation`, and `cannot_judge` remain. The constructors directly build values. `cannot_judge_out_of_reach` is replaced by `cannot_judge`; its slug existed only to join a constructor site to the retired exemption registry.

### Delete the sweep as one vertical slice

The scanner, registry, integration target, census entry, CI/DoD line, and mutation-variable scrub are one mechanism. Keeping any part would leave dead capability or a workflow claim with no implementation.

### Move the capability catalog before defending ownership

The surviving `rust-repository-reactions` declarations move from `tianheng::observation_bounds()` to `kanhe::bounds::observation_bounds()`. The model gate consumes both catalogs, then separately rejects this exact capability prefix from Tianheng's product catalog. This is deliberately narrow and observable: it repairs one capability ownership boundary without claiming that a string heuristic can infer arbitrary repository intent. Other repository-capability declarations still present in Tianheng are separate migration work, not silently absorbed here.

### Accept focused matrices as the repository-check evidence level

Each public repository judgement keeps fixture directions that assert the kind and message users act on. We deliberately stop requiring mutation coverage of every internal constructor site. A site may be refactored without creating a governance identity, and an unconstructible machine-error branch no longer needs a declared product bound.

## Risks / Trade-offs

- An internal constructor's kind or message can change without a dedicated test if no focused behavior direction reaches it. That is accepted: unobserved internal sites are implementation coverage, not independently declared contracts.
- Public catalog membership loses every `rust-repository-reactions` entry. The changelog marks this breaking and gives the exact migration.
- Removing mutation environment handling means old local invocations setting those variables become inert; no supported user workflow promised them.

## Verification

- Before relocation, the new product-catalog ownership check fails on the `rust-repository-reactions` ids.
- Controlled kind and message perturbations make existing focused gate directions fail, demonstrating the evidence retained after the sweep is gone.
- After removal, ordinary and env-poisoned focused gates agree because retired mutation variables have no effect.
- Bound register/model projections regenerate and remain bijective while reading the relocated Kanhe catalog.
- Full repository Definition of Done passes with the retired command removed from both AGENTS and CI.
