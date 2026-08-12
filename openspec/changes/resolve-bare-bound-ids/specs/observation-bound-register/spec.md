## MODIFIED Requirements

### Requirement: Prose MAY reference a declared bound, and a reference SHALL resolve

Prose that mentions a bound SHALL be cleared by the undeclared-prose reaction when it carries an explicit
reference of the form `(bound: <capability>/<slug>)`, where `<slug>` is the declaring scenario's heading
lowercased with each run of non-alphanumeric characters replaced by a single hyphen.

**Every reference SHALL be resolved wherever it appears, independent of whether its line also states a bound.**
Resolution belongs to the id, not to the wording around it: a reference reachable only through the
bound-prose scan is un-checked the moment a sentence is reworded out of that scan's pattern, which happened
here — a repair reworded a capability's overview out of the scan's pattern while improving it, and the two
references that repair added were never resolved again. A reference in a Purpose
paragraph, a requirement's prose, or inside a declared bound scenario SHALL be resolved the same way.

Each reference SHALL resolve to exactly one declared bound across all specs, and **every** reference the prose
carries SHALL be checked rather than one of them: resolving to none SHALL fail, because a reference that points
nowhere is indistinguishable from an undeclared bound, and resolving to more than one SHALL fail, which is also
what keeps derived ids unique rather than merely assumed unique.

What a reference does **not** establish SHALL be stated wherever the reference form is described: it clears
the prose it sits with, and it does not certify that the bounds the prose states are the bound it names. A
sentence listing four inherited bounds is cleared by one reference to a fifth. Authors SHALL therefore carry
one reference for each bound the prose names, and the reaction cannot enforce that.

A reference exists because the floor's alternative is worse. Without it, a sentence that legitimately
**points at** a bound declared elsewhere — in the same file, or in another dimension's spec — must either
be rewritten to avoid the words or be restated as a second declaration of the same bound. The first
degrades prose that is doing its job; the second is exactly the restatement this register exists to end,
and the drift it produces is already recorded as a live `BACKLOG.md` item.

A reference SHALL NOT be treated as a declaration: it carries no citation of its own, contributes nothing
to the register's bound count, and cannot be the only mention of a bound anywhere.

**A bound id written bare SHALL resolve too, wherever tracked Rust or Markdown carries it.** The `(bound: …)`
form is what *clears prose*; resolution belongs to the **id**, and an id is no less a reference for being
written without the wrapper. Measured before this was proposed: three occurrences across the tree did not
resolve, every one of them a stale citation of a bound that does exist, and two of the three sat in a doc
comment above the very test defending it. One was in a published crate. The bijection cannot see them — it
compares the two *declaration* sides, and a doc comment is neither.

**Recognition SHALL be by shape against the enumerated capability set, never by a list written beside it.** A
reference is a **maximal run of path characters** that is exactly `<capability>/<slug>`, where `<capability>`
is a directory under `openspec/specs/` and `<slug>` is kebab-case. Reading maximal runs is what keeps a path
from being mistaken for a reference: `openspec/specs/repository-checks/spec.md` is one run and not a
`<capability>/<slug>` pair, so it is not a reference — the same word-reading rule the adopter-narrative
reaction already applies for the same reason. Enumerating the capabilities rather than listing them is the
register's own prohibition: a capability added later must be recognized without this reaction being touched.

**This is reference resolution, not a judgement over prose.** The distinction is the one this repository has
drawn three times when rejecting a detector over sentences: a bound id has a recognizable shape and the set it
must land in is *produced* by the declarations, exactly as a path, an `--exact` identifier and a `(bound: …)`
reference already are. Nothing here decides what a sentence means.

What it does **not** establish is the same as for the wrapped form and SHALL be stated with it: resolving says
the id names a declared bound, never that the prose around it describes that bound.

#### Scenario: Prose referencing a declared bound is cleared

- **WHEN** a sentence mentions a bound and carries `(bound: <capability>/<slug>)` naming a declared bound
- **THEN** the reaction passes for that occurrence, and the register's bound count is unchanged

#### Scenario: A reference that resolves to nothing

- **WHEN** a reference names a `<capability>/<slug>` that no declared bound produces
- **THEN** the reaction fails, naming the file, the line, and the unresolved id, because a dangling
  reference is indistinguishable from an undeclared bound

#### Scenario: A reference on a line that states no bound

- **WHEN** a reference sits in prose that does not itself match the bound-prose pattern — a Purpose
  paragraph, or a sentence reworded away from those words
- **THEN** the reaction resolves it exactly as it would on any other line, so rewording a sentence cannot
  silently un-check the references it carries

#### Scenario: An earlier reference on the same line that resolves to nothing

- **WHEN** prose carries two references and only the later one resolves
- **THEN** the reaction fails, naming the unresolved one, because a line examined at one reference leaves the
  rest unchecked whichever one that is

#### Scenario: A reference that resolves to two declared bounds

- **WHEN** two declared bounds in one capability produce the same slug and a reference names it
- **THEN** the reaction fails, naming both declarations, so a derived id's uniqueness is checked rather
  than assumed

#### Scenario: A bare id in a doc comment names no declared bound

- **WHEN** tracked Rust or Markdown carries `<capability>/<slug>` with no `(bound: …)` wrapper, and no declared
  bound produces that id
- **THEN** the reaction fails, naming the file, the line and the unresolved id — the same refusal the wrapped
  form gets, because resolution belongs to the id rather than to the syntax around it

#### Scenario: A path that merely contains a capability name

- **WHEN** tracked content carries a path such as `openspec/specs/repository-checks/spec.md`, whose characters
  include a capability name followed by a slash
- **THEN** it is not read as a reference, because a reference is a maximal run of path characters that is
  exactly `<capability>/<slug>` and this run is neither

#### Scenario: A capability added later

- **WHEN** a new capability directory appears under `openspec/specs/` and prose cites one of its bounds bare
- **THEN** the citation is recognized and resolved without this reaction being edited, because the capability
  set is enumerated rather than listed beside the recognizer

#### Scenario: A reference is not a declaration

- **WHEN** a bound is mentioned only by references and declared nowhere
- **THEN** every reference fails to resolve, so the bound cannot exist in the register as a reference alone

