## Context

`docs/observation-bounds.md` shows two behaviours declared as bounds in three capabilities each, every
declaration citing the same test. Nothing reacts to that today: the register checks that each declaration
names a defence, not that a behaviour is declared once.

## Goals / Non-Goals

**Goals:**
- Detect a bound restated across capabilities, so the next one cannot accumulate silently.
- Keep the bound visible in every capability that has it, which is why the earlier "declare per capability"
  rule was chosen and is the property a naive "declare once" would lose.

**Non-Goals:**
- Raising a new capability for the shared surface. `semantic-signature-coupling` already states the property
  on the other capabilities' behalf, so ownership exists and a new capability would add a name without
  adding an observation.
- Detecting a restatement between a bound and rustdoc or `BACKLOG.md` prose. The register scans specs; the
  wider surfaces are their own change, as `observation-bound-register` already states.

## Decisions

**1. The reaction keys on the cited test, not on the statement text.** Two declarations of one behaviour
will not have identical prose — the three instances here differ in wording — but they cite the same test,
because one behaviour has one defence. Text similarity would be a heuristic; a shared citation is a fact.

**2. The owner is the capability that already claims the property on the others' behalf**, which
`semantic-signature-coupling` does in as many words. Where no such claim exists, the reaction names the
capabilities and leaves the choice to the author: ownership is a judgment the register can demand but not
compute.

**3. A reference keeps per-capability visibility, which is what makes "declare once" acceptable now.** The
earlier settlement — declare once per capability — was correct given no reference form. It is superseded,
and the spec says so rather than quietly changing.

**4. The reaction fires on two or more capabilities, never on repeated citation within one.** A bound whose
heading covers two shapes legitimately cites two tests, and one capability may cite one test from two
bounds; neither is a restatement. Only the same defence claimed by two capabilities is.

## Risks / Trade-offs

- **A shared test that genuinely defends two distinct bounds would false-positive** → then the two bounds
  differ and their defences should too; the reaction naming both is the prompt to split the test or the
  bound. Recorded as the intended reading rather than left to be discovered.
- **The reaction cannot say which capability should own the bound** → it names them all and demands a
  choice. Computing ownership would require modelling which capability's behaviour a test exercises, which
  is exactly the judgment the drift law keeps out of a reaction.
