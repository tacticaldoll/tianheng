# observer-protocol (delta)

## MODIFIED Requirements

### Requirement: Composition SHALL introduce no trait object

`Observer` SHALL name no `dyn` in its own signature, and composition SHALL introduce none either. Assembly
SHALL fold each observer **as it arrives**, so the heterogeneous collection never exists: each `observe` call is
monomorphized and the accumulator carries only the outcome so far.

A collection-based entry taking `&[&dyn Observer]` was designed first and rejected on measurement. The exposure
would have needed governing, and it cannot be: no module of the composed shell is governed by a semantic
boundary today, and the `dyn`-trait DSL offers only *forbid all* and *forbid named operands* — there is no
allow-except form, so *forbid all* would refuse the protocol's own signature while *forbid named* would never
see it. A declared exposure that no reaction could refuse is a name without a reaction, which this family
forbids. Removing the trait object is therefore not a preference over governing it; governing it was not
available.

The eager fold also carries the short-circuit for free: composing onto an accumulator that already cannot judge
SHALL NOT evaluate the observer at all.

Because that same measurement leaves 渾儀 unable to watch this crate, the reaction holding this requirement is
**lexical**, and a lexical reaction SHALL state where it stops and SHALL check every premise it rests on:

- Its recognizer SHALL be a **named function over one line** of text, so its limit can be demonstrated by giving
  it text rather than by rewriting the crate.
- It SHALL over-approximate in the safe direction: it cannot distinguish a `pub` item in a private module from a
  publicly reachable one, and flags both. A false positive here is a sentence to write; a false negative is an
  exposure nobody governs.
- It reads this crate's **top-level** source files only. That is sound exactly while every subdirectory of `src/`
  is reached through a non-`pub` `mod` declaration, so nothing beneath one is reachable from outside the crate —
  and the reaction SHALL **assert that premise** rather than rest on it. Measured, eight files under
  `src/runner/` are never opened, and an injected `pub fn … -> Option<Box<dyn Debug>>` among them leaves the
  reaction passing: harmless while those modules are private, and invisible the moment one is not.
- Reading one line at a time leaves a residual the premise check cannot remove, declared as an observation bound
  below.

#### Scenario: An adopter composes observers of different concrete types

- **WHEN** two observers of unrelated types are composed into one run
- **THEN** each is folded as it is added, with no trait object in any signature and no collection holding both

#### Scenario: Composition onto a cannot-judge accumulator

- **WHEN** an observer is composed onto an accumulator that already holds a constitution error
- **THEN** that observer is not evaluated, because a verdict resting on a boundary that could not be evaluated
  is not a verdict, and evaluating further would spend work on an answer that cannot be reported

#### Scenario: A source subdirectory becomes publicly reachable

- **WHEN** a subdirectory of `src/` is reached through a `pub mod` declaration
- **THEN** the reaction fails, because the premise justifying its top-level-only reading no longer holds and the
  files beneath that module would otherwise leave its reach with nothing said

#### Scenario: A trait object on a wrapped signature's continuation line is not seen — a stated bound

- **WHEN** a public signature spans several lines and names a trait object on a line that does not itself begin
  with `pub `
- **THEN** the reaction does not see it, a stated bound: the recognizer is handed one line at a time, so the
  continuation is never a candidate it declined — it is text the observation never presents. Closing it needs 渾儀
  watching this crate, which the same measurement above found unavailable. Multi-line public signatures exist
  here, so the shape is live even where no instance names a trait object
