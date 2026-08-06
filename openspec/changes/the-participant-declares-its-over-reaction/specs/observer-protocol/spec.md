# observer-protocol (delta)

## MODIFIED Requirements

### Requirement: A participant outside the family SHALL be demonstrated joining a run

A dogfood example SHALL exist in which a crate that is **not** part of the family implements `Observer`, is
composed into a run alongside the dimensions, and reacts. The protocol's claim is that a third party can take
part, and every implementor of it is a crate of this family, in this workspace, returning a literal list from its
own module — so the claim has never been executed.

The example's participant SHALL declare **computed** bounds: at least one id built at run time from what the
participant observed rather than written as a literal. `BoundId`'s owned-or-borrowed form exists precisely for an
implementor whose bounds are discovered, and until this it had no caller that was not a literal — a capability
shipped for a caller that did not exist.

The participant SHALL declare **every** bound it has, not only the one that demonstrates the mechanism. The
example is the one artefact teaching a third party how to join a run honestly, so a participant there that reacts
where its own stated reason does not require it — and says nothing — teaches the mechanism while withholding an
instance of it. Measured: its header rule read only a file's first line, so a module header below a license comment
was reported missing while the rule's reason, *that a reader learns what the file is for*, was satisfied. That
distance between a rule's wording and its reason is what `Reached::OverReacts` names, and it SHALL be declared
rather than closed where closing it would trade one edge for others and make the wording diverge from the code.

The example SHALL therefore exhibit **more than one extent**, so it demonstrates the bound *model* and not only the
call that declares a bound.

The example SHALL require **no addition to any crate's public API**. If joining a run needs an export the family
does not publish, the protocol is not usable by a third party, and that is the finding rather than a reason to add
the export.

#### Scenario: A third-party participant joins a composed run

- **WHEN** the example composes its own observer alongside the family's dimensions over its workspace
- **THEN** the run reacts, and the participant's contribution is present in the verdict rather than only the
  dimensions'

#### Scenario: The participant's bounds are computed rather than literal

- **WHEN** the participant declares its bounds
- **THEN** at least one id is built from what it observed, so the owned-or-borrowed declaration form is exercised
  by a caller outside the family

#### Scenario: The participant reacts where its own reason does not require it

- **WHEN** the participant's rule reacts to a shape its stated reason is already satisfied by
- **THEN** that is declared as an over-reaction with its own bound, so the example demonstrates the extent model
  rather than only the mechanism for declaring one

#### Scenario: Joining a run would require a new export

- **WHEN** an outside crate cannot implement or compose the protocol with the published surface alone
- **THEN** that is a defect in the published surface, reported as such, and not repaired by publishing whatever
  the example happened to need
