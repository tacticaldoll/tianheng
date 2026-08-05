## MODIFIED Requirements

### Requirement: A cited pinning test SHALL resolve to exactly one definition in the tree

A citation's syntax SHALL be validated before it is resolved. The cited name SHALL be a Rust identifier, an
optional crate qualifier SHALL be a crate-directory name, and at most one `::` separator SHALL appear;
anything else SHALL fail, naming the bound id and the rejected citation. This closes two directions **by
construction** rather than by escaping. The name is interpolated into the search pattern, so a regular-expression
metacharacter would let a citation for a test that does not exist resolve to a differently-named function —
defeating the renamed-or-deleted direction this requirement exists for. The qualifier is joined to a
filesystem path, so `../` would resolve a citation against a function outside the `crates/` boundary this
requirement declares.

The reaction SHALL verify that each `PINNED-BY` name resolves to exactly one Rust function
**definition** under `crates/`, and that the resolved definition is a **test**. Resolving to none SHALL
fail: a test that was renamed or deleted leaves a citation that reads as coverage while defending
nothing, which is the silent pass the register opposes. Resolving to more than one SHALL also fail: a
name defined twice makes the citation name a set rather than a reaction, so the bound's defender is not
identified. Resolving to a function that is **not a test** SHALL fail for the same reason as an absent
one: a citation names what defends the bound, and a helper or production function of the right name
defends nothing while reading as coverage.

A definition SHALL be recognized as a test by an attribute run immediately above it containing `#[test]`,
read upward past interleaved attributes rather than only on the line before — `#[should_panic]` sits
between the attribute and the `fn` in three places in this tree, so a single-line read would refuse a
real test. The walk SHALL run to the enclosing item's boundary and SHALL NOT be capped at a fixed number of
lines: no attribute-run length is declared anywhere, so a cap refuses a legitimate test whose run is longer
than the cap happened to be.

The walk SHALL stop at a block-comment delimiter rather than interpret one, so a `#[test]` written inside a
block comment does not satisfy the run. It SHALL NOT strip or track comments: comment state is a forward
property of a file that an upward walk cannot know, and stripping requires lexing string literals — this
tree's own lexer fixtures carry 49 `/*` occurrences **inside string literals**, several of them nested, so a
delimiter-counting stripper would manufacture phantom comments and swallow real definitions. Stopping at the
delimiter refuses the shape without either, and its error direction is loud: a test whose attribute run
genuinely contains a block comment is refused rather than quietly accepted.

Requiring the cited function to be a test is not a naming convention imposed on a suite the register does
not own; it is what the citation already means. The register SHALL require nothing of the test's **name**
beyond its being an identifier, which is what lets the bound-pinning tests keep at least three naming
variants while some carry no "bound" in the name at all.

Matching SHALL be on the definition form, never on a bare mention: a citation SHALL NOT be satisfied by a
name appearing in a line comment, a doc link, or a string. **That claim is a floor over the mention forms
the definition pattern excludes, and SHALL be stated as one.** The pattern reads a line's shape, not its
comment state, so a function definition that is itself inside a block comment satisfies a citation. Closing
that needs the same string-literal lexing measured above as out of reach for a text-scanning gate, so the
residual SHALL be stated here and in the projection's header, and SHALL be pinned by a fixture recording the
accepted behaviour, so a later repair is not silently absorbed.

That residual SHALL NOT be declared as a bound of this capability, and the reason is a limit of the citation
form rather than a judgment: `PINNED-BY` names a Rust test under `crates/`, while this reaction's own
defences are shell fixtures, so the declaration would have to be `UNPINNED` against a tracker owning
something already measured as out of reach — permanent debt wearing an owner's name, which the unpinned
requirement forbids. That the register cannot pin a bound of its own capability SHALL be recorded as an
observation in `BACKLOG.md` rather than worked around here.

#### Scenario: A citation whose name is not an identifier

- **WHEN** a declared bound's `PINNED-BY` contains a character no Rust identifier may hold
- **THEN** the reaction fails before resolving it, naming the bound id and the rejected citation, so a
  metacharacter cannot resolve a citation to a differently-named function

#### Scenario: A citation whose crate qualifier leaves the crates directory

- **WHEN** a declared bound's `PINNED-BY` qualifier is not a plain crate-directory name — a traversal, a
  nested path, or a second `::` component
- **THEN** the reaction fails before resolving it, so a citation cannot be satisfied by a function outside
  the boundary this requirement declares

#### Scenario: A citation naming a test that no longer exists

- **WHEN** a declared bound's `PINNED-BY` names a function defined nowhere under `crates/`
- **THEN** the reaction fails, naming the bound id and the unresolved test name

#### Scenario: A citation naming a test defined twice

- **WHEN** a declared bound's `PINNED-BY` name is defined by two functions under `crates/`
- **THEN** the reaction fails, naming the bound id and both definition sites, because the citation is
  ambiguous rather than merely imprecise

#### Scenario: A citation satisfied only by a mention

- **WHEN** a declared bound's `PINNED-BY` name appears in the tree only inside a line comment or a string,
  with no function definition of that name
- **THEN** the reaction fails exactly as for an absent test, because a mention defends nothing

#### Scenario: A citation resolving to a function that is not a test

- **WHEN** a declared bound's `PINNED-BY` resolves to exactly one function definition under `crates/` and
  that definition carries no `#[test]` in the attribute run above it
- **THEN** the reaction fails, naming the bound id and the definition site, because a function that never
  runs as a test defends nothing while occupying the place of the defence

#### Scenario: A pinning test whose attribute run carries another attribute

- **WHEN** a cited test's definition is preceded by `#[test]` and then a further attribute such as
  `#[should_panic]`
- **THEN** the reaction resolves it as a test, so the check reads the attribute run rather than one line

#### Scenario: A pinning test whose attribute run is longer than any cap

- **WHEN** a cited test carries `#[test]` above more interleaved attributes than a fixed-window walk would
  read
- **THEN** the reaction still resolves it as a test, because the walk ends at the item boundary rather than
  at a line count

#### Scenario: An attribute written inside a block comment

- **WHEN** a cited function's `#[test]` sits inside a block comment above the definition
- **THEN** the reaction fails as for any non-test definition, because the walk stops at the delimiter rather
  than reading commented text as an attribute

#### Scenario: A definition inside a block comment is not distinguished from a real one

- **WHEN** a cited function's whole definition sits inside a block comment
- **THEN** the reaction resolves it, which is the stated residual of matching on a line's form: a fixture
  records this so a later repair is not absorbed silently, and the projection's header states it where a
  register reader sees it

#### Scenario: One test cited by bounds in two capabilities

- **WHEN** declared bounds in two different capabilities cite the same `PINNED-BY` test
- **THEN** the reaction fails, naming every declaring capability and the shared test, because one behaviour
  has one defence and therefore one declaration; the others reference it

#### Scenario: A bound citing two tests is not a restatement

- **WHEN** one declared bound whose heading covers two shapes cites two tests
- **THEN** the reaction passes, since a bound covering two shapes is defended by two tests

#### Scenario: One capability citing one test from two bounds is not a restatement

- **WHEN** two declared bounds within a single capability cite the same test
- **THEN** the reaction passes: the restatement this direction exists for is one defence claimed by two
  capabilities, never repetition inside one
