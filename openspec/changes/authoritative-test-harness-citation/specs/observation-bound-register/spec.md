## MODIFIED Requirements

### Requirement: A cited pinning test SHALL resolve to exactly one definition in the tree

A citation's syntax SHALL be validated before it is resolved. The cited name SHALL be an ASCII Rust
identifier, optionally raw (`r#name`); an optional crate qualifier SHALL be a crate-directory name; and at
most one `::` separator SHALL appear. Anything else SHALL fail, naming the bound id and the rejected citation.
This closes two directions **by construction** rather than by escaping. The name is interpolated into the
search pattern, so a regular-expression metacharacter would let a citation for a test that does not exist
resolve to a differently-named function — defeating the renamed-or-deleted direction this requirement exists
for. The qualifier is joined to a filesystem path, so `../` would resolve a citation against a function
outside the `crates/` boundary this requirement declares.

The restriction to ASCII is narrower than Rust's own identifier grammar and SHALL be stated as such rather
than implied: the search pattern is byte-oriented, no cited name needs otherwise, and the refusal of a
non-ASCII identifier is loud — an author sees it — where accepting one and matching it unreliably would not
be.

**Whether a cited name is a test that runs SHALL be decided by the test harness, not by the source text.**
The reaction SHALL enumerate each workspace member's registered tests and SHALL fail when the cited name is
absent from the cited crate's set. Enumeration SHALL be per package rather than per workspace, because the
enumeration carries no crate label while a citation may be crate-qualified — this repository already has one
test name registered in two crates, so a workspace-wide match would let a citation qualified to one crate be
satisfied by the other's test.

The harness is the authority because it is the only exact observation source for the claim. A text scan reads
shape, so it accepted a `#[test]` neutralised by `#[cfg(any())]`, a `#[test] fn` inside an uninvoked
`macro_rules!` body, and a definition inside a raw string or a block comment — all measured, none of which
registers a test. Enumerating those sub-cases in the scan is unbounded (`cfg`, `cfg_attr`, feature gates, a
cfg-gated `mod`, comments, strings, macros), and the previous version of this requirement declared one of them
as a residual before three more were found.

**The text scan SHALL remain as a declared fallback, and the degradation SHALL be reported.** A repository
with no root manifest cannot be enumerated — the failure matrix builds such repositories deliberately — so
there the attribute-run walk decides test-ness, and the reaction SHALL say on its own output that it did. A
gate that silently drops its strongest direction reports a weaker clean than the one it claims.

Where the enumeration itself cannot be produced — no `cargo`, or a workspace that does not build — the
reaction SHALL exit **cannot judge** rather than fall back silently, because a citation's test-ness is then
undecided rather than decided weakly.

The reaction SHALL verify that each `PINNED-BY` name resolves to exactly one Rust function **definition**
under `crates/`. Resolving to none SHALL fail: a test that was renamed or deleted leaves a citation that reads
as coverage while defending nothing. Resolving to more than one SHALL also fail: a name defined twice makes
the citation name a set rather than a reaction, so the bound's defender is not identified. This direction
supplies the **site**, which the enumeration does not carry, and the crate precision that makes a qualified
citation exact.

When the harness registers the cited test but the definition scan locates no site, the reaction SHALL report
the **line-shape limitation** — the scan requires `fn` and the name on one line — rather than reporting the
test absent, since the two directions disagree about a form rather than about existence.

The fallback walk SHALL recognize a definition as a test by an attribute run immediately above it containing
`#[test]`, read upward past interleaved attributes, to the enclosing item's boundary, with no line cap, and
stopping at a block-comment delimiter rather than interpreting one.

Requiring the cited function to be a test is not a naming convention imposed on a suite the register does not
own; it is what the citation already means. The register SHALL require nothing of the test's **name** beyond
its being an identifier.

#### Scenario: A citation whose name is not an identifier

- **WHEN** a declared bound's `PINNED-BY` contains a character no ASCII Rust identifier may hold
- **THEN** the reaction fails before resolving it, naming the bound id and the rejected citation, so a
  metacharacter cannot resolve a citation to a differently-named function

#### Scenario: A citation naming a raw identifier

- **WHEN** a declared bound's `PINNED-BY` names `r#name`
- **THEN** the reaction accepts the citation's form, because a raw identifier is a Rust identifier and the
  register imposes no naming convention of its own

#### Scenario: A citation whose crate qualifier leaves the crates directory

- **WHEN** a declared bound's `PINNED-BY` qualifier is not a plain crate-directory name — a traversal, a
  nested path, or a second `::` component
- **THEN** the reaction fails before resolving it, so a citation cannot be satisfied by a function outside
  the boundary this requirement declares

#### Scenario: A cited name the harness does not register

- **WHEN** a cited name is absent from the enumerated tests of the crate it is qualified to, or of the
  workspace when unqualified
- **THEN** the reaction fails, naming the bound id and the name, because a citation names what defends the
  bound and an unregistered function defends nothing

#### Scenario: A test neutralised by a cfg attribute

- **WHEN** a cited function carries `#[test]` and a `cfg` attribute that removes it from the build
- **THEN** the reaction fails, because the attribute run says test while the harness registers nothing

#### Scenario: A test inside an uninvoked macro body

- **WHEN** a cited function's `#[test] fn` tokens sit inside a `macro_rules!` body that nothing invokes
- **THEN** the reaction fails, because tokens that expand nowhere register no test

#### Scenario: A definition inside a string or a block comment

- **WHEN** a cited function's definition sits inside a multi-line string literal or a block comment
- **THEN** the reaction fails, because the harness registers no test for it — retiring the residual the
  previous version of this requirement declared for the block-comment case

#### Scenario: The harness cannot be enumerated

- **WHEN** the judged repository has no root manifest
- **THEN** the reaction decides test-ness by the fallback walk and reports on its own output that it did, so a
  reader of a clean result knows which direction produced it

#### Scenario: The enumeration cannot be produced at all

- **WHEN** a root manifest exists but the enumeration fails — no `cargo`, or a workspace that does not build
- **THEN** the reaction exits cannot-judge, because test-ness is undecided rather than weakly decided

#### Scenario: A citation naming a test defined twice

- **WHEN** a declared bound's `PINNED-BY` name is defined by two functions under `crates/`
- **THEN** the reaction fails, naming the bound id and both definition sites, because the citation is
  ambiguous rather than merely imprecise

#### Scenario: A registered test the definition scan cannot locate

- **WHEN** the harness registers a cited test whose `fn` keyword and name sit on different source lines
- **THEN** the reaction reports the line-shape the scan requires, rather than reporting the test absent

#### Scenario: A citation satisfied only by a mention

- **WHEN** a declared bound's `PINNED-BY` name appears in the tree only inside a comment or a string, with no
  registered test of that name
- **THEN** the reaction fails, because a mention defends nothing

#### Scenario: A pinning test whose attribute run carries another attribute

- **WHEN** the fallback walk reads a definition preceded by `#[test]` and then a further attribute such as
  `#[should_panic]`
- **THEN** it resolves as a test, so the fallback reads the attribute run rather than one line

#### Scenario: A pinning test whose attribute run is longer than any cap

- **WHEN** the fallback walk reads a definition whose `#[test]` sits above more interleaved attributes than a
  fixed-window walk would read
- **THEN** it still resolves as a test, because the walk ends at the item boundary rather than at a line count

#### Scenario: An attribute written inside a block comment

- **WHEN** the fallback walk reads a definition whose `#[test]` sits inside a block comment
- **THEN** it does not resolve as a test, because the walk stops at the delimiter rather than reading
  commented text as an attribute

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
