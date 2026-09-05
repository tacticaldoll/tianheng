# structured-violation-identity Specification

## Purpose

Define the shared structured identity model that separates a violation's stable observed fact from
its human presentation while keeping fact meaning inside the observation dimension that owns it.

## Subject

- `crates/xuanji/src/*.rs`

## Requirements
### Requirement: A finding has stable structured identity and human presentation

The shared reaction model SHALL represent an observed finding as both human-readable presentation
and a validated `StructuredFactIdentity`. The identity SHALL contain a non-empty semantic fact type,
a non-empty semantic shape, and zero or more uniquely named scalar string fields in canonical name
order. A semantic identifier SHALL name enduring meaning rather than a revision ordinal. Construction
SHALL reject empty identifiers/field names and duplicate field names, and SHALL NOT admit arbitrary
recursive values. Storage SHALL be private behind validated construction and read-only accessors.

#### Scenario: Presentation changes without changing fact identity

- **WHEN** a dimension renders the same observed fact with improved human wording or diagnostics
- **THEN** its presentation may change while its structured fact and violation identity remain unchanged

#### Scenario: Distinct facts carry distinct identities

- **WHEN** two observations differ in any identity-bearing observed value
- **THEN** their semantic type, shape, or named field values differ, so accepting one cannot suppress the other

#### Scenario: An ambiguous identity is rejected

- **WHEN** a caller supplies an empty type/shape/field name, duplicate field name, or recursive value
- **THEN** construction reports an error rather than normalizing or overwriting the ambiguous input

#### Scenario: A semantic identifier is not a generation number

- **WHEN** another fact family or compatible diagnostic field is added
- **THEN** existing identifiers remain unchanged and no global v3/v4 identity generation is introduced

#### Scenario: The declaring crate is an identity-bearing observed value when it can vary

- **WHEN** a boundary kind can be declared against more than one crate in a workspace, and two
  crates each declare the identical rule against the identical governed target
- **THEN** the crate each was declared against is itself an identity-bearing observed value, so the
  two observations' identities differ and one being accepted does not suppress the other — unless a
  dimension already encodes the declaring crate in the target or another identity role it uses for
  that boundary kind, in which case no additional field is needed to satisfy this scenario

#### Scenario: A boundary kind that already encodes its crate in another identity role is not double-counted

- **WHEN** a boundary kind's identity already varies by crate through its target (or another
  identity role), because the boundary is inherently crate-scoped rather than module-path-scoped
- **THEN** this requirement does not obligate that boundary kind's fact to carry the same crate a
  second time as a redundant field

### Requirement: Observation dimensions own fact meaning and rendering

Each observation dimension SHALL own the typed fact schemas it can observe and the conversion from
each fact to its structured identity and human presentation. The shared reaction crate SHALL own
only the dimension-agnostic envelope and SHALL NOT contain crate-, module-, semantic-, or runtime-
specific fact vocabulary. A dimension SHALL derive identity and presentation from the same typed
fact conversion. Separately observed identity-bearing components SHALL remain fact-specific named
fields rather than being concatenated into an opaque display string. Each dimension SHALL remain
usable without `tianheng` or another observation dimension.

#### Scenario: A dimension introduces a new fact shape

- **WHEN** an observation dimension begins reacting to a new kind of fact
- **THEN** its schema and rendering are added in that dimension while `xuanji` and other dimensions remain unchanged

#### Scenario: Shared identity does not reverse the dependency graph

- **WHEN** all dimension fact schemas compile with the shared reaction model
- **THEN** every observation dimension depends inward on `xuanji`, while `xuanji` depends on no observation dimension

#### Scenario: An instrument emits an independently inspectable reaction

- **WHEN** an adopter invokes 圭表, 渾儀, or 漏刻 directly
- **THEN** its Outcome exposes the same vocabulary-neutral structured identities used by the composed facade

### Requirement: Published structured identity schemas are compatibility-reacted

Every observation dimension SHALL carry an explicit compatibility reaction for every shipped fact
family and every finite typed discriminator affecting semantic type, shape, canonical field names,
or field values. Each dimension SHALL inspect at least one violation produced through its real
boundary reaction to pin target, rule key, and fact as separate identity roles. Adding a fact or
finite discriminator SHALL require an explicit catalog decision. Reactions SHALL NOT freeze human
presentation, complete report JSON, or diagnostic metadata.

Compatibility SHALL additionally be proved behaviorally: reordering declarations or inserting an
unrelated item SHALL NOT change existing identities, and distinct observed facts SHALL remain
distinct across cfg branches and unrenderable syntax. No public identity field or fallback SHALL be
derived solely from traversal position, ordinal, or collection index. A syntax-ban catalog MAY
supplement these tests but SHALL NOT replace them.

#### Scenario: Every shipped dimension fact is cataloged

- **WHEN** compatibility tests run across 圭表, 渾儀, and 漏刻 with all features
- **THEN** every fact and finite discriminator has exact expected semantic identifiers, named fields, and representative values

#### Scenario: Reordering observations preserves identity

- **WHEN** declarations are reordered or an unrelated declaration is inserted before an observed fact
- **THEN** the fact retains the same identity and baseline match

#### Scenario: Distinct unrenderable facts stay distinct

- **WHEN** two distinct facts contain syntax that cannot use the ordinary canonical renderer
- **THEN** an observed structural discriminator keeps them distinct or observation fails loud, never assigning the same positional fallback

#### Scenario: Presentation remains free to change

- **WHEN** only human wording or non-identity diagnostics change
- **THEN** compatibility reactions and baseline identity remain unchanged

### Requirement: Violations are constructed from typed identity

The public model SHALL construct a `ViolationId` from a governed target, validated semantic
`RuleKey`, and `StructuredFactIdentity`, and SHALL construct a `Violation` by attaching presentation,
boundary kind, reason, severity, and diagnostics. External callers SHALL NOT construct an id by
struct literal or mutate any identity component. `ViolationId` equality and ordering SHALL use only
the target, rule key, and fact identity. Human rule/finding presentation, reason, severity, file,
anchor, polarity, complete signature diagnostics, baseline status, owner, and tracker SHALL NOT
enter identity.

#### Scenario: A dimension emits a violation through typed identity

- **WHEN** a dimension converts an observed fact into a violation
- **THEN** it supplies the three typed identity roles rather than adjacent presentation strings

#### Scenario: Metadata and wording do not re-identify a violation

- **WHEN** only presentation, reason, severity, location, anchor, polarity, diagnostics, or annotations change
- **THEN** the new `ViolationId` compares equal to the prior identity

#### Scenario: A materially different rule is a different identity

- **WHEN** a rule parameter changes what the boundary permits or forbids for the same target and fact
- **THEN** its semantic rule key differs, so the old baseline does not suppress the new law

#### Scenario: Identity provenance cannot be forged

- **WHEN** an external caller constructs or inspects a live identity
- **THEN** validated constructors require all three roles and public access is read-only

### Requirement: Existing adopter-facing reaction entry points remain available

The adopter-written `Constitution` and boundary builders SHALL retain their existing names and
roles, as SHALL `tianheng::run`, the standalone instrument checks, and the composed pure check. The
public reaction types (`Baseline`, `BaselineEntry`, `ViolationId`, `Violation`, `Report`, and
`Outcome`) SHALL remain available with the intentional 0.3.0 identity shape changes. This
capability SHALL NOT introduce a public dimension/plugin trait or testing assertion DSL.

#### Scenario: Standalone and composed consumers inspect the same model

- **WHEN** an adopter calls a standalone instrument or `tianheng::check_constitution`
- **THEN** both return inspectable Outcomes using the same structured reaction identities

### Requirement: Every live violation identity component is read-only

The governed target stored by `Violation` SHALL be private behind a read-only accessor, matching its
private rule key and structured fact. No external caller with mutable access to a `Violation` SHALL
be able to rewrite any component returned by `Violation::id()`.

#### Scenario: External inspection cannot mutate target identity

- **WHEN** an external consumer inspects a live violation
- **THEN** it can read the target through an accessor but cannot assign a replacement target

### Requirement: Identity migration and testing harness remain separate capabilities

The structured-identity capability SHALL NOT define or require a plugin protocol or testing DSL.
This statement SHALL NOT deny the separately specified `tianheng::testing::GovernanceTest`
capability shipped by the facade.

#### Scenario: Release documentation describes both capabilities

- **WHEN** adopter-facing release notes summarize the identity migration and reusable testing harness
- **THEN** they state that identity introduces no plugin protocol while accurately listing the testing harness

### Requirement: The compilation unit is an identity-bearing observed value when a package has more than one

The compilation unit an observation came from SHALL be an identity-bearing observed value wherever a
dimension observes more than one of a package's units, so the same fact observed in two units yields two identities
and accepting one SHALL NOT suppress the other. A package may build more than one crate root — a library
beside a `bin`, several `[[bin]]` targets, or both — and each is its own compilation unit with its own
module graph.

Without it the two collapse, because every root of a package denotes the module path `crate` and shares
the package name: a violation accepted in one root would silently mask the same violation appearing later
in another — the baseline-masking false negative, arriving through the corpus rather than through a
renderer.

The role SHALL be **declaration-derived and stable**, never positional: not the order targets appear in
metadata, not an index. A target's **name** SHALL NOT be used alone, because it is not unique within a
package — a package may build a library and a `bin` of the same name. The role SHALL be the unit's root
source path relative to the package's own directory, which is unique per unit, moves with neither the
checkout nor the member set, and is the thing whose contents produced the observation.

A root whose path does not lie under that directory SHALL be a **constitution error** naming it, never
labeled by the path as given. That path is the checkout's own location, so using it would make the
identity checkout-dependent — the same commit in two clones yielding two identities, and a baseline
recorded in one matching nothing in the other, which is the defect this role exists to prevent. Refusing
to judge is the Core Contract's own ordering over a silently degraded label, and it matches the runtime
dimension's refusal of a relative or empty anchor.

This is deliberately NOT the rule the runtime dimension applies to a file reached through an absolute
path literal, and the difference SHALL be stated wherever either is: that literal is **committed text**,
identical in every checkout, so keeping it verbatim is exactly what makes it stable, whereas a root path
outside the package directory is the checkout's location, so keeping it verbatim is what makes it
unstable. Same shape, opposite consequence.

A dimension that observes exactly one compilation unit per package is unaffected and SHALL NOT add the
role, exactly as the declaring-crate requirement above does not obligate a boundary kind that already
varies by crate.

#### Scenario: The same violation in two roots of one package stays two identities

- **WHEN** a package builds both a library root and a `bin` root, the identical forbidden construct is
  written in each, and one boundary governs them
- **THEN** the two observations carry different identities, so a baseline accepting the one in the `bin`
  root does not suppress the one that later appears in the library root

#### Scenario: A target name alone does not distinguish a unit

- **WHEN** a package builds a library target and a `bin` target that share the package's own name
- **THEN** the identity role still distinguishes them, because it is derived from each unit's root source
  path rather than from the target name the two have in common

### Requirement: A fact carries every varying coordinate of its observation's location

A fact's identity SHALL carry every coordinate of **where** the observation was made that can vary
within the governed space, and SHALL NOT carry one that cannot. The coordinates are, from outermost in:
the declaration that governs it, the compilation unit, the module, the owner or item, and the
position-free discriminator of the thing itself within that item. A coordinate SHALL be omitted only
when it cannot vary for that fact family or is already encoded in the violation's target, and the
omission SHALL be recorded with the reason rather than left as silence.

No coordinate SHALL be positional — not scan order, item ordinal, traversal index, or renderer fallback
position — and none SHALL be checkout-dependent, since either makes an identity that shifts without the
observation changing.

This derivation exists because the alternative does not work: an identity collision is a missing
coordinate, and the coordinates that go missing are not foreseeable — a second crate
declaring the same boundary, a second module implementing the same owner, a second impl block bounding
the same parameter, a second spelling of one trait, a second path differing only in undecodable bytes, a
second crate root, and a second module importing the same forbidden path. Widening a fact's schema
pre-emptively SHALL NOT be used to anticipate them: an identity is its **values**, so declaring a field
before its value is known re-keys every recorded baseline once for the field and again when the value
arrives, and a coordinate that cannot vary adds nothing to distinguish.

Each dimension's own published-identity-schema reaction SHALL be the enforcement point: a fact family
whose schema omits a coordinate that can vary SHALL fail that reaction rather than await review.

#### Scenario: A new fact family omitting a varying coordinate fails its schema reaction

- **WHEN** a fact family is added or changed so that its identity omits a coordinate of the observation's
  location that can vary for it
- **THEN** the dimension's published-identity-schema reaction fails, rather than the omission surviving
  until two observations are found to collide

#### Scenario: A coordinate that cannot vary is not added

- **WHEN** a coordinate is already encoded in the violation's target, or cannot vary for a fact family at
  all
- **THEN** the fact does not carry it, and the reason is recorded — so the identity stays as narrow as the
  observation and no baseline re-keys for a field that distinguishes nothing

### Requirement: An identity-bearing path is labeled canonically, not as the platform renders it

A path that becomes part of a violation's identity SHALL be recorded through one canonical labeling
rule, shared by every dimension that records one, rather than by each site rendering the path as its
own platform and string type happen to.

The rule has two parts, and each closes a defect the other does not:

- **The separator is the label's, not the platform's.** A label SHALL use `/` as its only component
  separator, whatever separator the observing platform uses. A component cannot contain `/` on any
  supported platform, so `/` in a label unambiguously means a component boundary. Separator
  interpretation SHALL be delegated to the platform's own path-component semantics rather than
  performed by substituting characters: on some platforms a backslash is a legal byte *within* a
  component, so substituting it would map two distinct paths onto one label, and on others more than
  one character separates, so substituting one is incomplete.
- **Every byte survives.** A label SHALL preserve information the observed path carried, so two paths
  differing only in bytes that are not valid UTF-8 keep two labels. A lossy rendering that replaces
  undecodable bytes with a replacement character SHALL NOT be used, since a baseline accepting the
  first such path would silently suppress the second's never-accepted violation.

A path whose bytes are not valid UTF-8 SHALL therefore be **judged**, not refused: it has a canonical
label like any other. This applies wherever such a path is reachable — an observation dimension that
walks the filesystem — and the rule SHALL be stated once and shared rather than restated per dimension,
so a dimension for which the input happens to be unreachable cannot drift from one for which it is not.

#### Scenario: The same commit labels a compilation unit identically on either platform

- **GIVEN** a package whose crate root is at `src/lib.rs` relative to its manifest directory
- **WHEN** the compilation unit's label is recorded on a platform whose path separator is `/`, and
  again on one whose separator is `\`
- **THEN** both record `src/lib.rs`, so a baseline written by either is matched by the other, and the
  entry does not re-fire as new for a contributor on the other platform

#### Scenario: A separator byte that is legal inside a component is not treated as a separator

- **GIVEN** a platform on which a backslash is an ordinary byte within a file name
- **WHEN** a single file literally named `a\b` is labeled, and separately a file `b` inside a directory
  `a` is labeled
- **THEN** the two labels differ, because the labeling asks the platform which characters separate
  rather than substituting a fixed one

#### Scenario: Two paths differing only in undecodable bytes keep two identities

- **WHEN** two observed paths differ only in bytes that are not valid UTF-8
- **THEN** their labels differ, and neither is refused; a baseline accepting one does not suppress the
  other

#### Scenario: A path outside the anchor it is labeled against is refused for that reason alone

- **GIVEN** a labeling that is defined relative to an anchor, and an observed path not under it
- **WHEN** the label cannot be formed
- **THEN** the refusal reports that the path lies outside the anchor, and is unreachable for any other
  cause — the labeling itself SHALL be total, so no second failure mode can share the one diagnostic —
  and the message an adopter reads names the condition that actually holds
