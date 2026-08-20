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
method SHALL be located by a **unique occurrence** of its signature in the source, and the reader SHALL decline
otherwise. An earlier rule required the signature to begin a trimmed line, which bought only the exclusion of a
mid-line mention: a whole-line copy inside a block comment anchors exactly as well as the definition, and a decoy
conforming copy above a divergent method let the equality pass on text that was not the method — measured.
Counting occurrences does **not** subsume the mid-line mention, and the reaction SHALL require both
conditions rather than either. A source that mentions the signature mid-line and never defines it has exactly
one occurrence, so a count-only rule admits it, anchors in the prose, and returns the next function's body as
this method's — measured. Each rule refuses something the other admits, and requiring both only ever declines
more, which is this reader's declared error direction. What neither refuses is a **whole-line** copy inside a
block comment, which is declared as a bound below rather than described as closed. And a **trailing comment** on the delegation SHALL be
prose, not a second list: the region discipline this family already holds says a comment is never executed text,
and the reaction that judges a shell gate's own text strips one before comparing for exactly this reason.
The reaction SHALL apply Rust line-comment semantics to the inspected body: a `//` line is prose, while a Rust
attribute beginning with `#` remains executed Rust text.

Where the built-in path obtains a dimension's outcome **by invoking that dimension's observer**, equality for
that dimension holds **by construction rather than by observation**, and the spec SHALL say which dimensions
those are — otherwise a reader takes a constructed equality for a measured one. The list is now held to that:
a reaction reads the built-in path's own source and refuses if a dimension it declares construction-held is
not constructed there, or if a dimension it does not declare so is. This was true in only one direction until
the 0.5.0 window: the list named runtime alone, the shell's semantic arm changed under it, and the list was
repaired by hand — a membership claim about a set with an enumerator, which is the shape this family refuses
everywhere else. What answers it is textual rather than a perturbed build: for a construction-held dimension
the built-in path does not call some *other* function that happens to agree with the observer today, it
directly constructs that dimension's own `Observer` and calls `.observe()` on it, so there is exactly one
implementation to read rather than two runs to compare. The **runtime** and **semantic** dimensions are such
cases: the built-in path invokes `RuntimeObserver::new(...).observe(...)` and
`SemanticObserver::new(...).observe(...)` directly, so for runtime its two copies of those three statements —
the corpus derivation, the audit call and the `cannot read workspace` message — become one, and for semantic
there is no second call at which the two verdicts could differ. What that does *not* settle for either is
whether the shell honours its delegation obligation, which is a different property with a bound of its own.
The **static** dimension remains independently implemented on both sides — the built-in path calls
`check_and_cover` and never constructs `StaticObserver` — and for it the reaction's equality is observed.

Where a dimension's equality is construction-held, the reaction SHALL still observe that the fixture's boundary
for that dimension **reacts at all**. Otherwise an arm that quietly went vacuous would leave the whole
comparison resting on the dimensions that did not.

#### Scenario: A whole-line occurrence that is not the definition anchors the read — a stated bound

- **WHEN** the method's definition is absent from the inspected source — the impl having moved elsewhere — and a
  whole-line copy of its signature remains anywhere in that file: inside a block comment, inside a string
  literal, or in any other position the reader does not distinguish from executed text
- **THEN** the reaction reads that copy's body and reports it as the method's. Both anchor conditions are
  satisfied — one occurrence, at a line start — and the reader knows nothing of comments or literals, so the
  class is "the unique whole-line occurrence is not the definition" rather than any one syntactic position.
  What passes is a **second, hand-maintained path that agrees today**: a *divergent* list does not, because
  `observation-bound-model` reads every dimension's declarations through `Observer::bounds` and holds them in a
  bijection with the specs, which fails on any difference of membership or content. Measured both ways. So the
  residual is narrower than a divergent list slipping through, and wider than a comment.
  **Not a defect unique to this reader.** `kanhe::region`'s own `Executed` abstraction declares the identical
  residue for the same reason — a block comment and a string literal both need nested-span lexing to tell from
  executed text, which this tree has defeated repeatedly and left declared rather than approximated. Closing
  either needs the same instrument; closing one without the other would leave the class recorded twice under
  two names for a reader to reconcile.
  This bound SHALL be **shown rather than described**: the reaction enumerates every shape it decides together
  with the decision, the reader is run against that table, and the rows where it reads a body that is not the
  method's are this bound. A sentence here that the table contradicts fails, which is what the three repair
  rounds preceding this scenario could not do
- **UNPINNED** `BACKLOG.md` — *the bounds-method reader anchors on a whole-line occurrence that is not the definition*

#### Scenario: The stated construction-held list is held against the composition path

- **WHEN** the built-in composition path's own source is read for each dimension named construction-held above
- **THEN** the reaction fails if a construction-held dimension's own `Observer` is not constructed there, or if
  a dimension not named construction-held has one constructed there instead — read directly rather than
  inferred from a mutated build, since a construction-held dimension has exactly one implementation to find,
  not two runs to compare
- **PINNED-BY** `the_construction_held_list_matches_the_built_in_composition_path`

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

#### Scenario: That trailing comment contains a closing brace

- **WHEN** an observer's bounds method holds the delegation, a trailing comment containing `}`, and a further
  statement beneath it
- **THEN** the reaction reads the body to its real closing brace and fails on the further statement, because the
  comment tail is removed **before** the braces are counted rather than after — counted through, the body closed
  at the comment and the further statement was never presented to the comparison at all, so the one thing this
  requirement refuses passed as the delegation

#### Scenario: A brace inside a block comment or a string literal no longer moves the read body extent

- **WHEN** an inspected bounds-method body carries `{` or `}` inside a block comment or a string literal
- **THEN** the reaction reads the method's real body, to its real closing brace, whichever construct the brace
  sits inside. The extent step parses the source with `syn` rather than counting braces by eye, so a comment or
  a string literal is tokenized as what it is before any brace inside either is ever available to be counted —
  closing the bound this scenario used to declare, in both directions it named: neither a block comment nor a
  string literal moves the extent any longer. What replaces that closed bound is this implementation's own
  failure mode: if the source does not parse as a Rust file, or parses without a function-like item beginning
  exactly where the anchor step said the definition starts, the reaction refuses to verify rather than passing —
  never a silent acceptance of a body it could not attribute to that exact site
- **PINNED-BY** `a_brace_in_a_block_comment_or_a_string_literal_no_longer_moves_the_body_extent`

#### Scenario: A Rust attribute appears in an inspected body

- **WHEN** an inspected Rust body contains a line whose trimmed start is `#`
- **THEN** the reaction retains that line as Rust source rather than dropping it as a shell comment

#### Scenario: An observer's bounds method cannot be found where the reaction looks

- **WHEN** the method is absent from the source the reaction reads
- **THEN** the reaction refuses to judge rather than passing, because a reaction that finds nothing to read has
  not observed that the obligation holds

#### Scenario: A second line could anchor the bounds method

- **WHEN** the bounds-method signature occurs more than once in the observer's source — a commented-out copy
  being the measured case
- **THEN** the reader declines rather than reading the first. Here the decoy inverts this reader's declared
  error direction rather than merely moving the extent: a *conforming* copy in the comment makes the exact
  one-statement equality pass while the real method holds a second, divergent list, so the over-reaction the
  bound records becomes an acceptance
