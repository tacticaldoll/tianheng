# observer-protocol (delta)

## MODIFIED Requirements

### Requirement: Composition SHALL introduce no trait object

No signature of the composed shell SHALL name a trait object. The eager fold removes the exposure rather than
governing it: a collection-based entry taking `&[&dyn Observer]` was designed first and rejected on measurement —
no module of this crate is governed by a semantic boundary, and the `dyn`-trait DSL offers only forbid-all and
forbid-named-operands, so a declared exposure would have been a name with no reaction.

A reaction SHALL therefore assert it lexically, since 渾儀 is not watching this crate. **Being lexical, the
reaction SHALL state where it stops**, and SHALL check every premise it relies on rather than resting on one:

- It reads this crate's top-level source files only. That is sound *because* every subdirectory of `src/` is
  reached through a non-`pub` `mod` declaration, so nothing beneath one can be reachable from outside the crate.
  The reaction SHALL **assert that premise**, so making such a module public fails it and demands recognition
  rather than silently removing those files from its reach: measured, eight files under `src/runner/` are never
  opened, and an injected `pub fn … -> Option<Box<dyn Debug>>` among them leaves the reaction passing.
- Its recognizer requires the exposure marker on a line that itself begins with `pub `. A wrapped signature's
  continuation line therefore carries no marker the recognizer can see. This one is not a premise to check but a
  limit of a line-oriented matcher, and it is declared as an observation bound below.

The recognizer SHALL be a named function over one line of text, so its limit can be demonstrated by giving it
text rather than by rewriting this crate.

The reaction SHALL over-approximate in the safe direction: it cannot tell a `pub` item in a private module from a
publicly reachable one, and flags both. A false positive here is a sentence to write; a false negative is an
exposure nobody governs.

#### Scenario: A trait object appears in a top-level signature

- **WHEN** a line beginning `pub ` in this crate's top-level source names a trait object
- **THEN** the reaction fails, naming the file and line

#### Scenario: A source subdirectory becomes publicly reachable

- **WHEN** a subdirectory of `src/` is reached through a `pub mod` declaration
- **THEN** the reaction fails, because the premise that justifies reading only the top level no longer holds and
  the files beneath it would otherwise leave the reaction's reach with nothing said

#### Scenario: A trait object on a wrapped signature's continuation line is not seen — a stated bound

- **WHEN** a public signature spans several lines and names a trait object on a line that does not itself begin
  with `pub `
- **THEN** the reaction does not see it, a stated bound: the recognizer reads one line at a time, and the
  alternative — governing this crate semantically — is the one measured to be unavailable, since no module here
  carries a semantic boundary and the `dyn` DSL has no allow-except form. Multi-line public signatures exist in
  this crate, so the shape is live even where no instance names a trait object
