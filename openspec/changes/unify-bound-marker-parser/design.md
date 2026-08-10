# Design

## Context

`bound_register_parse::marks_a_bound` defines the grammar used by the register. `observation_bound_model.rs` copied that rule, then widened it independently. Because each gate begins from its own enumeration, later set comparisons cannot prove they started from the same declared population.

## Goals / Non-Goals

**Goals:**

- Give declaration recognition one implementation.
- Preserve the register's bare singular marker grammar.
- Retain independent slug derivation as a useful cross-check.

**Non-Goals:**

- Change existing bound ids or defence declarations.
- Redesign the observation-bound model.
- Reclassify projections or repository gates as product reactions.

## Decisions

### Reuse the register predicate

The model imports `kanhe::bound_register_parse::marks_a_bound` and deletes its local predicate. This is construction, not a second agreement test: all spec-side consumers now answer declaration membership through the same function.

### Pin the grammar at the source

A focused unit table asserts the two accepted bare singular markers and rejects plural, article-less, and qualified lookalikes. The negative run temporarily restores the widened predicate in the model; a plural synthetic heading must then enter the model population and fail the bijection.

### Keep slug derivation independent

Slug derivation remains duplicated because the model compares its derived ids with the register projection. Sharing that function would collapse the comparison to the same implementation on both sides; marker membership has no analogous independent comparison and therefore gains nothing from duplication.

## Risks / Trade-offs

- A future grammar amendment must change the shared predicate and its table. That is intentional: consumers cannot silently choose different grammars.
- Importing a Kanhe library helper from a Kanhe integration test adds no new crate dependency or product coupling.

## Verification

- Focused predicate table passes.
- Controlled widened-predicate run makes the model gate fail on a plural near-miss.
- Ordinary bound-register and observation-bound-model gates pass.
- Full repository Definition of Done passes.
