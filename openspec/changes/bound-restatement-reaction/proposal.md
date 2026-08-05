## Why

The observation-bound register made a restatement measurable on its first projection. Two behaviours are
declared as bounds in **three capabilities each**, all six declarations citing the same single test:
`an_impl_nested_one_level_further_stays_a_stated_bound` and `a_static_wrapped_impl_stays_a_stated_bound`
appear under `semantic-forbidden-marker`, `semantic-signature-coupling`, and
`semantic-trait-impl-locality`.

That is the shape `BACKLOG.md` records as a live `READY-PATCH`: an inherited bound restated by each
capability that inherits it, so one behaviour change leaves several specs stale at once. It has already
cost this repository twice — the `#[path]`-remap bound was stale in two capabilities simultaneously, and a
sync left a contradicting bound beside its own reacting scenario.

The entry named three candidate answers and framed them as alternatives. They are not: a reaction
**detects** the restatement and the repair **resolves** it, and only the reaction keeps the next one from
accumulating silently.

## What Changes

- A **reaction**: a pinning test cited by declared bounds in more than one capability fails the register.
  One capability declares the bound; the others reference it. This is checkable, and it is what turns
  "declare once" from a convention into a fact.
- The **repair** the reaction forces, on the two instances that exist: the declaration stays with
  `semantic-signature-coupling`, which its own spec already identifies as the owner — it states the
  anchor-and-item property "on their behalf" for every single-module-anchored semantic capability — and the
  other two capabilities carry `(bound: …)` references instead of parallel declarations.
- **A settled question is reopened and answered differently, deliberately.** The register's design settled
  that a shared bound is declared once per capability, each citing the same test, because declaring it once
  would leave the other specs silent about a bound they have. That reason does not survive the reference
  form, which did not exist when it was settled: a reference keeps the bound visible in every capability
  that has it while leaving one declaration to maintain. The superseded reasoning is replaced rather than
  left standing.

Not breaking: additive reaction, no adopter-visible surface, no reaction over adopter code changes.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `observation-bound-register`: gains the cross-capability restatement reaction, and its shared-bound rule
  changes from "declared once per capability" to "declared once, referenced elsewhere".

## Impact

- **`scripts/check_bound_register.sh`** gains one direction; **`scripts/test_bound_register.sh`** gains its
  failure and passing fixtures.
- **`openspec/specs/observation-bound-register/spec.md`** — the modified requirement.
- **Four spec files** lose a parallel declaration and gain a reference; `docs/observation-bounds.md` is
  regenerated and its bound count falls by four.
- **`BACKLOG.md`** — the `READY-PATCH` entry closes with its resolution recorded.
- No crate API, wire format, identity shape, or manifest change.
