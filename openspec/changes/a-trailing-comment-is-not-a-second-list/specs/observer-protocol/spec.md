# observer-protocol (delta)

## MODIFIED Requirements

### Requirement: The built-in path SHALL keep its behaviour, and the two paths SHALL be held equal

`check_constitution` and the CLI SHALL keep their present composition path and observable behaviour, coverage
included. The protocol SHALL be an additional entry rather than a replacement.

A reaction SHALL assert that folding the three dimensions **through the trait** yields the same outcome as the
existing path on this workspace, and that each dimension's observer declares exactly the bound set that
dimension exports. Two composition paths that could disagree silently is the drift a seam is supposed to end.

Both obligations name a *property*, not a comparison, and the paragraphs below say how each is reacted to —
because for parts of both the two sides are one thing, and a comparison of one thing against itself is an
assertion that cannot fail. Where that is so, the reaction is over the construction that makes the property
true.

**The comparison SHALL NOT be able to hold vacuously in any one dimension.** The fixture it compares over SHALL
declare a deliberately violated boundary in **every** dimension, and the reaction SHALL assert that every
dimension reacted. A dimension whose declared set is empty contributes nothing to either side, so the two paths
agree for that dimension however wrongly one of them behaves: measured, an empty constitution is `Clean` on this
workspace, and with a static-only fixture, replacing an observer's body with `Clean` left the reaction passing.
Asserting per-dimension reaction is what keeps the fixture from silently going vacuous when the workspace
changes under it.

**An observer declares its dimension's bounds by delegating, and the reaction SHALL be over that delegation's
shape** rather than over a comparison of the two sides. Comparing an observer's `bounds()` against its
dimension's exported declarations cannot fail while `bounds()` *is* that export — measured, it is `f() == f()`,
and drifting a declaration left the reaction passing. What the requirement refuses is a **second, divergent
list**, and a second list is something written in a body; so each observer's `bounds()` SHALL hold exactly the
delegation and nothing else, recognized by position within that method rather than by the call appearing
anywhere in the file. The declarations' *content* is held by `observation-bound-model`'s extent projection and
SHALL NOT be re-asserted here.

Two things follow from *recognized by position*, and both were measured as gaps rather than reasoned about. The
method SHALL be located by **line position** — a line whose trimmed start is the signature — so a mention of it
inside a comment or a string cannot be brace-matched from. And a **trailing comment** on the delegation SHALL be
prose, not a second list: the region discipline this family already holds says a comment is never executed text,
and the reaction that judges a shell gate's own text strips one before comparing for exactly this reason.

Where the built-in path obtains a dimension's outcome **by invoking that dimension's observer**, equality for
that dimension holds **by construction rather than by observation**, and the spec SHALL say which dimensions
those are — otherwise a reader takes a constructed equality for a measured one. The runtime dimension is
currently such a case: the built-in path delegates to `RuntimeObserver`, so its two copies of the corpus
derivation, the audit call and the `cannot read workspace` message become one. The static and semantic
dimensions remain independently implemented on both sides, and for them the reaction's equality is observed.

#### Scenario: The trait-driven fold disagrees with the existing path

- **WHEN** the two paths produce different outcomes for this workspace
- **THEN** the reaction fails, because an additional entry that quietly judges differently is worse than no
  entry at all

#### Scenario: A dimension of the equality fixture stops reacting

- **WHEN** the fixture's declared boundary for some dimension no longer produces a violation of that
  dimension's kind
- **THEN** the reaction fails, because from that moment the comparison proves nothing about that dimension —
  and it fails naming the dimension, since the repair is to the fixture rather than to either path

#### Scenario: An observer's bounds method holds a list of its own

- **WHEN** an observer's bounds method holds anything other than the delegation to its dimension's exported
  declarations
- **THEN** the reaction fails, so the protocol's obligation cannot be satisfied by a second, divergent list

#### Scenario: The delegation carries a trailing comment

- **WHEN** an observer's bounds method holds the delegation followed by a comment explaining it
- **THEN** the reaction accepts it, because a comment is prose and not a list — the same region rule every other
  reaction in this repository reads its subject through

#### Scenario: An observer's bounds method cannot be found where the reaction looks

- **WHEN** the method is absent from the source the reaction reads
- **THEN** the reaction refuses to judge rather than passing, because a reaction that finds nothing to read has
  not observed that the obligation holds

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
  and the reaction SHALL **assert that premise** rather than rest on it. Measured, the files under `src/runner/`
  are never opened, and an injected `pub fn … -> Option<Box<dyn Debug>>` among them leaves the reaction passing:
  harmless while those modules are private, and invisible the moment one is not. How many files that is is
  deliberately not written here — a count of an enumerable set, kept by hand, goes stale in silence.
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
- **PINNED-BY** `a_trait_object_on_a_continuation_line_is_not_recognized`
