# observation-bound-model (delta)

## MODIFIED Requirements

### Requirement: The specs' declarations and the code's SHALL be held in bijection

A reaction SHALL assert that the set of bound ids declared in `openspec/specs/*/spec.md` **equals** the set
declared in code, and SHALL name every id on either side that has no counterpart. Ids SHALL be asserted
duplicate-free before the sets are compared, since two declarations collapsing onto one id would satisfy an
equality that proves nothing.

The id SHALL be the `<capability>/<scenario-slug>` form the register already derives, so this reaction
introduces no second naming scheme and no lookup table.

Both directions are required for the same reason the register requires both of its own: a spec bound with no
declaration is an unclassified claim, and a declaration with no spec bound is a classification of something no
reader can find.

**A dimension's declarations SHALL be read through `Observer::bounds`**, not through its exported free function.
The protocol requires a participant to declare what it does not observe, and a required method nothing reads is
answered into a void: measured, `bounds()` had no call site anywhere outside a comment, so a dimension could have
answered anything without moving a verdict. Reading the bijection through it makes the register the method's
consumer, and a dimension returning the wrong set now fails here.

The **shell's** own declarations SHALL keep coming from its free function, because the shell composes dimensions
rather than being one and implements no observer. That asymmetry is stated so it does not read as the same gap
this requirement closes.

#### Scenario: A spec declares a bound with no typed declaration

- **WHEN** a bound scenario is added to a spec and no declaration is added in code
- **THEN** the reaction fails, naming the id, because the qualifier slot it used to carry is gone and an
  unclassified bound would otherwise pass silently

#### Scenario: Code declares a bound no spec states

- **WHEN** a declaration exists whose id matches no bound scenario
- **THEN** the reaction fails, naming the id, because a classification a spec reader cannot find is a fact
  recorded where nobody looks

#### Scenario: Two declarations collapse onto one id

- **WHEN** two declarations carry the same id
- **THEN** the reaction fails before comparing the sets, because set equality would hold while one bound went
  unclassified

#### Scenario: An observer answers the bounds method with the wrong set

- **WHEN** a dimension's `Observer::bounds` returns a set other than that dimension's declarations
- **THEN** the bijection fails, naming every id left unclassified, because the register reads the answer rather
  than reading past it
