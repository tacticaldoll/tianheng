## MODIFIED Requirements

### Requirement: An empty semantic observer SHALL not read workspace metadata

The semantic dimension's public composed entry point SHALL return `Clean` for an empty boundary bundle without
reading the manifest. The shell and `SemanticObserver` SHALL delegate both empty and non-empty semantic bundles
to that entry point rather than maintaining independent empty-boundary guards, so every semantic composition
path has one behavior owner.

The repository reaction SHALL inspect the executed body of the shell's `evaluate_constitution` composition
function. That body SHALL access `constitution.semantic_boundaries()` exactly once, as the direct boundary
argument to `hunyi::check_all`; a missing function, an additional semantic-boundary inspection, or an indirect
shell-local decision SHALL fail rather than be treated as delegation.

The reaction SHALL distinguish a body that does not delegate from a body it could not read. The extent it
inspects is found by counting braces, and a string literal, a character literal, or a block comment inside the
body moves that extent; where the extent carries `"`, `'`, `/*`, or `*/` on an executed line, the reaction SHALL
refuse to judge rather than assert over text it cannot be sure is the function's body. Stating the refusal here
is what closes a false negative rather than describing one: this requirement's comparison is a count and a
containment, and both survive a truncated extent unharmed — a second semantic-boundary access sitting past the
cut is simply absent from what is compared, so the one shape the requirement refuses reads as the delegation it
demands. Measured on the tracked body with a delimiter introduced, not supposed.

Over-refusal is the declared direction, and the character literal is why it must be said out loud: a lifetime is
spelled with the same delimiter, so a composition body that names one is refused too. That cost is accepted
because a refusal is loud and repairable in the commit that causes it, while the alternative is the silent pass
above. The reaction SHALL therefore be held to still *judging* the tracked body, so a refusal that swallowed
every input could not pass for the closure.

**This does not contradict an empty observer set being a cannot-judge**, and the difference is stated here
because the two sentences read as a contradiction otherwise. An empty *bundle* means a participant was
composed and declares nothing for its dimension — a static-only adoption is exactly that, and there is nothing
to observe, so `Clean` is the honest answer. An empty *observer set* means nothing was composed at all: the
misconfiguration is in the assembly, not in a dimension's declarations, and there is no participant whose
silence could be read as cleanliness. Unifying them fails in both directions: making an empty bundle a
cannot-judge would make every static-only adopter's composed run report exit `2`, a false refusal on the
primary use case, and reporting an empty observer set as clean is the vacuous pass this repository has
re-opened most often. The asymmetry is therefore a property of the two constructions rather than a claim a
reaction could observe, and it carries no scenario for that reason.

#### Scenario: Empty semantic boundaries through the public semantic entry point

- **WHEN** `check_all` receives an empty semantic boundary bundle and a path that cannot be read
- **THEN** it returns `Clean`, because there is no semantic observation to perform

#### Scenario: Empty semantic boundaries through an observer

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns `Clean` by delegating to the public semantic entry point

#### Scenario: Empty semantic boundaries through the shell

- **WHEN** the shell composes a constitution whose semantic boundary bundle is empty
- **THEN** the source-shape reaction finds exactly one semantic boundary access, passed directly to the public semantic entry point, and fails if the shell decides emptiness itself

#### Scenario: A second semantic-boundary access sits past a moved extent

- **WHEN** the composition body holds the delegation, a construct whose delimiter moves the read extent, and a
  further `constitution.semantic_boundaries()` access beyond the resulting cut
- **THEN** the reaction refuses to judge rather than reporting the delegation as satisfied, because the further
  access is the one shape this requirement refuses and a moved extent never presents it to the comparison at all

#### Scenario: The composition body carries a delimiter that can move the read extent — a stated bound

- **WHEN** the extent read for `evaluate_constitution` carries `"`, `'`, `/*`, or `*/` on an executed line
- **THEN** the reaction refuses to judge, naming the delimiter — a stated bound. It does not decide whether the
  body delegates, because the extent it would decide over may not be the body; separating a brace in code from
  one inside a string, a character literal, or a block comment needs the lexing this repository measured and
  rejected, so the reaction declines the verdict instead of guessing at it
- **PINNED-BY** `an_ambiguous_delegation_extent_is_refused_rather_than_judged`

#### Scenario: The tracked composition body is still judged

- **WHEN** the reaction reads the tracked `evaluate_constitution` body, which carries none of those delimiters
  on an executed line
- **THEN** it returns a verdict rather than a refusal, because a refusal that swallowed every input would
  satisfy the bound above while observing nothing

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
The reaction SHALL apply Rust line-comment semantics to the inspected body: a `//` line is prose, while a Rust
attribute beginning with `#` remains executed Rust text.

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
  comparison rather than of the extent, and SHALL NOT be read as a property of every reader of that extent: the
  same moved extent meeting a count-and-containment comparison accepts a divergent body instead, which is why
  the shell-delegation reaction refuses an ambiguous extent rather than inheriting this bound
- **PINNED-BY** `a_brace_in_a_block_comment_moves_the_body_extent`

#### Scenario: A Rust attribute appears in an inspected body

- **WHEN** an inspected Rust body contains a line whose trimmed start is `#`
- **THEN** the reaction retains that line as Rust source rather than dropping it as a shell comment

#### Scenario: An observer's bounds method cannot be found where the reaction looks

- **WHEN** the method is absent from the source the reaction reads
- **THEN** the reaction refuses to judge rather than passing, because a reaction that finds nothing to read has
  not observed that the obligation holds
