## MODIFIED Requirements

### Requirement: Every enumerated gate SHALL hold the family's exit contract in a checkable form

Each gate SHALL install the shared backstop from `scripts/lib/exit_contract.sh`, SHALL pass it a label that is
the gate's own name, SHALL declare the three-way contract in its header, and SHALL accept a target directory
argument so a fixture can be pointed at it.

The header declaration SHALL be recognized by **shape, not by wording**: a three-way statement whose third
term is cannot-judge, with the verdict words for 0 and 1 left to the gate. The gates word them differently —
"0 clean, 1 violation", "0 coherent, 1 incoherent", "0 publishable, 1 wrong source" — and each names its own
subject better than a shared phrase would. A reaction demanding one literal sentence would report gates as
violating this requirement while every one of them declares its contract: the invented-violation direction,
and the one a capability about gates can least afford.

The label SHALL be **derived from the gate's basename** rather than compared against a kept table: `check_` and
`.sh` removed, underscores read as spaces, so `scripts/check_bound_register.sh` names itself `bound register`. A
table would be a second declaration of the gate's name and would rot exactly as the thing it checks.

That label is the gate's self-identification in the one diagnostic a reader gets when the shell aborts a gate
instead of the gate refusing. The same diagnostic prints `${BASH_SOURCE[0]}` and `$LINENO`, which expand in the
failing gate's own frame, so a wrong label does not lose the location — it **contradicts** it, and a
contradiction is read in whichever direction the reader trusts first. The hazard is copy-paste, which is how
this surface came to exist: six gates carrying one shape, each written by reading a sibling.

The label SHALL be written as a **literal**, and a gate whose label is built by expansion SHALL be refused with
that as the stated reason rather than as a mismatch. The reaction reads a gate's text and does not evaluate it,
so it cannot confirm a computed label; reporting an unconfirmed label as correct is the direction this family
refuses, and reporting it as "you wrote X, the basename asks for Y" would be a false statement about a gate that
wrote neither.

Requiring a literal is a requirement on **authored form**, legitimate here by the same ownership argument that
lets this capability require the twins' helper names: these gates are authored in this repository for this
purpose. It is worth stating plainly that the form being required is not the better one — a gate deriving its own
label from `$0` could not disagree with its filename at all, where a literal can. The literal is required because
the property has to be checkable by reading, and a rule admitting derivations would be a rule about which
spellings of a derivation the reaction recognizes.

#### Scenario: A gate omits the shared backstop

- **WHEN** an enumerated gate does not source and invoke `exit_contract_backstop`
- **THEN** the reaction fails, naming the gate, because an unhandled command's status then escapes as a
  foreign exit code the contract does not define

#### Scenario: A gate's backstop label is not its own name

- **WHEN** an enumerated gate passes `exit_contract_backstop` a label that is not its basename with `check_`
  and `.sh` removed and underscores read as spaces
- **THEN** the reaction fails, naming the gate, the label it wrote and the label its basename asks for, because
  the reader who trips this has by construction just copied a sibling and is looking at neither name

#### Scenario: A gate's backstop label is not a literal

- **WHEN** an enumerated gate builds its label by expansion rather than writing it
- **THEN** the reaction fails, saying the label could not be read as a literal, because a reaction that reads
  text cannot confirm a computed label and must not report an unconfirmed one as correct

#### Scenario: A gate that installs no backstop has no label to check

- **WHEN** an enumerated gate does not invoke `exit_contract_backstop` at all
- **THEN** the reaction reports both the missing installation and the missing label, each naming a real absence,
  exactly as an absent twin reports the matrix properties it cannot hold

#### Scenario: A gate's header declares the contract in its own verdict words

- **WHEN** an enumerated gate's header states a three-way contract ending in cannot-judge, using verdict
  words of its own for 0 and 1
- **THEN** the reaction accepts it, because the property is that the contract is declared, not that it is
  declared in one sentence

#### Scenario: A gate cannot be pointed at a fixture

- **WHEN** an enumerated gate takes no target directory argument
- **THEN** the reaction fails, naming the gate, because a gate that only ever judges its own checkout cannot
  be observed refusing, and a guard is not a guard until it has been seen to fail
