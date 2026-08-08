## MODIFIED Requirements

### Requirement: An empty semantic observer SHALL not read workspace metadata

The semantic dimension's public composed entry point SHALL return `Clean` for an empty boundary bundle without
reading the manifest. The shell and `SemanticObserver` SHALL delegate both empty and non-empty semantic bundles
to that entry point rather than maintaining independent empty-boundary guards, so every semantic composition
path has one behavior owner.

**Whether the shell honours that is not observed, and the source-shape reaction that claimed to observe it is
retired.** It read the characters of one function body; the obligation is about what the shell *does*. Four
review rounds narrowed it and each narrowing was defeated: by resolution (a `use` shadowing the entry point's
name, with the body byte-identical), by the binding site (the parameter renamed, or a second one added — the
parameter list is outside the read extent), by which definition is the subject (a raw identifier, leaving a
commented copy as the only signature occurrence), by the caller frame (the guard moved into `check_constitution`),
and — the group no widening reaches — by **execution**: a delegation bound to `let _`, written inside a
never-invoked `macro_rules!`, or placed in a conditionally-called closure satisfies every textual rule while the
shell decides for itself. It also cost two false-positive classes, one of them fired by `rustfmt` reformatting a
conforming body.

Retiring it was the honest disposition rather than a fifth narrowing, and the gap it left is **closed by
construction** rather than by a sixth reader. The shell's semantic arm invokes `SemanticObserver` instead of
calling the composed entry point beside it, so the two are one call rather than two agreeing ones and there is
no second site in which a shell-local decision could sit — the route the runtime arm already took. What a text
reader could still have said truthfully is narrow — that the body reaches its constitution only through the
declared accessors — and that was never the obligation.

The property therefore stands where the equality does: stated as a construction in this requirement rather
than as a scenario, because a scenario asserting it could not fail. The cost is one clone of the declared
bundle per run, the same price the runtime arm pays for its `to_vec`.

#### Scenario: Empty semantic boundaries through the public semantic entry point

- **WHEN** `check_all` receives an empty semantic boundary bundle and a path that cannot be read
- **THEN** it returns `Clean`, because there is no semantic observation to perform

#### Scenario: Empty semantic boundaries through an observer

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns `Clean` by delegating to the public semantic entry point

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
Counting occurrences excludes the mid-line mention too, as a second occurrence. And a **trailing comment** on the delegation SHALL be
prose, not a second list: the region discipline this family already holds says a comment is never executed text,
and the reaction that judges a shell gate's own text strips one before comparing for exactly this reason.
The reaction SHALL apply Rust line-comment semantics to the inspected body: a `//` line is prose, while a Rust
attribute beginning with `#` remains executed Rust text.

Where the built-in path obtains a dimension's outcome **by invoking that dimension's observer**, equality for
that dimension holds **by construction rather than by observation**, and the spec SHALL say which dimensions
those are — otherwise a reader takes a constructed equality for a measured one. The **runtime** and
**semantic** dimensions are such cases: the built-in path invokes `RuntimeObserver` and `SemanticObserver`, so
for runtime its two copies of the corpus derivation, the audit call and the `cannot read workspace` message
become one, and for semantic there is no second call site in which a shell-local decision could sit. The
**static** dimension remains independently implemented on both sides — the built-in path calls
`check_and_cover`, the observer calls `check` — and for it the reaction's equality is observed.

Where a dimension's equality is construction-held, the reaction SHALL still observe that the fixture's boundary
for that dimension **reacts at all**. Otherwise an arm that quietly went vacuous would leave the whole
comparison resting on the dimensions that did not.

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

#### Scenario: A brace inside a block comment or a string literal moves the read body extent — a stated bound

- **WHEN** an inspected bounds-method body carries `{` or `}` inside a block comment or a string literal
- **THEN** the reaction reads an extent that is not the method's body — a stated bound.
  It counts braces outside line comments only, and closing the gap needs the string-literal lexing this
  repository measured and rejected: this tree's own lexer suites put comment delimiters inside string literals,
  several of them nested, so a delimiter-counting scan opens a phantom comment at the first of them and swallows
  every definition to the next close. For **this** comparison the error direction is the safe one, and
  it is what the pin shows — no brace-carrying construct survives the exact one-statement comparison, so a moved
  extent refuses a **conforming** body rather than accepting a divergent one. The direction is a property of the
  comparison rather than of the extent, and it does not transfer to another reader of that extent: the
  same moved extent meeting a count-and-containment comparison would accept a divergent body instead. A reader of
  that second kind existed over the shell's composition body and is retired; the direction is recorded here so
  the next one is not written on the assumption that this bound's safety transfers to it
- **PINNED-BY** `a_brace_in_a_block_comment_moves_the_body_extent`

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
