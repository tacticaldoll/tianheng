# rule-model-surface Specification

## Purpose

Define a builder-owned rule construction surface that remains forward-compatibly inspectable while
allowing rule representations to grow without parallel public variants.

## Subject

- `crates/xuanji/src/*.rs`
- `crates/guibiao/src/model/*.rs`

## Requirements

### Requirement: Boundary builders own public rule construction

The public `Rule` and `ModuleRule` enums SHALL remain readable model types, but every data-carrying
variant SHALL remain non-exhaustive. External consumers SHALL construct rules through the existing
boundary DSL. Each builder-produced rule SHALL expose a validated semantic `RuleKey` for reaction
identity and a separate human-readable presentation for projection. External consumers SHALL NOT
directly construct or mutate either a rule variant or its key.

#### Scenario: Existing adopter DSL still constructs a rule

- **WHEN** an external consumer declares an existing boundary through its builder
- **THEN** the declaration compiles and produces the same law semantics with a stable rule key

#### Scenario: Direct external construction stays closed

- **WHEN** an external consumer attempts to construct a data-carrying rule variant or mutate its key
- **THEN** compilation fails and the consumer must use the boundary DSL

### Requirement: Rule variants remain forward-compatibly inspectable

`Rule` and `ModuleRule` and their existing variant names SHALL remain public, and boundary `rule()`
accessors SHALL remain available. External consumers SHALL inspect known fields using open-ended
patterns. A reaction identity SHALL separately expose the rule's semantic key, so inspecting or
changing projection wording does not define identity. Every finite rule family and identity-bearing
parameter SHALL be cataloged; a parameter SHALL enter the key exactly when changing it changes what
the boundary permits or forbids.

#### Scenario: An external consumer matches a known field

- **WHEN** a consumer obtains a builder-produced rule and matches a known field with `..`
- **THEN** the match compiles without assuming the complete representation

#### Scenario: Presentation-only rule changes preserve identity

- **WHEN** only a rule's displayed wording or parameter formatting changes
- **THEN** the semantic rule key and existing baseline match remain unchanged

#### Scenario: A changed law has a changed key

- **WHEN** an identity-bearing parameter changes the allowed or forbidden set
- **THEN** the rule key differs and an old baseline cannot suppress the materially changed law

#### Scenario: A new rule family requires classification

- **WHEN** a new finite rule variant or identity-bearing parameter is added without a catalog entry
- **THEN** the rule compatibility reaction fails to compile or fails its test

### Requirement: Strict-external is one inline-rule modifier

The public builder SHALL represent `.strict_external()` as a modifier of the existing inline symbol
confinement rule rather than as a second public rule variant. The modifier SHALL preserve the
existing default-off behavior, local-precedence classification, constitution errors, human and JSON
projections, polarity, and violation identity. Adding or removing `.strict_external()` on an
otherwise identical boundary SHALL continue to leave the violation's target, rule, and finding key
unchanged.

#### Scenario: Default inline confinement stays default-off

- **WHEN** a boundary uses `must_not_call_inline(prefix)` without `.strict_external()`
- **THEN** it retains the existing default resolver behavior and omits `strict_external` from its projection

#### Scenario: The modifier preserves strict-external behavior

- **WHEN** the same builder adds `.strict_external()`
- **THEN** fully-qualified declared external paths are classified through the existing strict-external rules and the projection carries `strict_external: true`

#### Scenario: The representation fold does not re-key a violation

- **WHEN** default and strict-external forms observe a path that both already classify as the same violation
- **THEN** their target, rule, finding key, polarity, and count are identical

### Requirement: Reference consumer surfaces remain available

The guibiao check, coverage, projection, baseline, and shared-model re-export functions and types SHALL
retain their existing names. pacta's Tianheng builder/runner use and modou's guibiao
projection/baseline integration SHALL compile against the narrowed local model without source
changes.

#### Scenario: pacta and modou compile against the local crates

- **WHEN** both reference consumers are checked with their Tianheng-family dependencies patched to the local change
- **THEN** their existing builder, runner, check, coverage, projection, baseline, and type-name usage compiles unchanged

### Requirement: Specific boundary builders SHALL expose explicit ScanDepth toggles

The public reaction model SHALL provide a strongly-typed `ScanDepth` enum (`Shallow`, `Subtree`) with `#[default]` set to `Shallow`. Supporting boundary builders (`guibiao`: `ModuleBoundary`, `InlineConfinementDraft`; `hunyi`: `AsyncExposureBoundary`, `ImplTraitBoundary`) SHALL expose `.depth(ScanDepth)` to allow explicit configuration of observation depth. Legacy module boundaries SHALL preserve their default `Subtree` evaluation and baseline identity, while `.depth(ScanDepth::Shallow)` restricts observation to the anchored seam. Existing ergonomic builders (such as `.including_submodules()`) SHALL map to `.depth(ScanDepth::Subtree)` and SHALL remain fully compatible.

#### Scenario: Legacy module boundary construction preserves existing Subtree evaluation and identity

- **WHEN** a module boundary is declared without an explicit depth modifier
- **THEN** its scan depth defaults to `ScanDepth::Subtree` (governing its whole module subtree) and its rule key and evaluation behavior are 100% preserved

#### Scenario: Explicit depth configuration via ScanDepth enum

- **WHEN** a supporting boundary builder is configured with `.depth(ScanDepth::Subtree)` or `.depth(ScanDepth::Shallow)`
- **THEN** the boundary retains the specified depth and evaluates matching targets accordingly

#### Scenario: Existing builder ergonomics delegate to ScanDepth

- **WHEN** an adopter calls an existing modifier like `.including_submodules()`
- **THEN** the boundary configures its depth to `ScanDepth::Subtree` without breaking caller code

### Requirement: Every generic module rule honors its declared ScanDepth

Every Guibiao generic `ModuleBoundaryDraft` rule that exposes `.depth(ScanDepth)` SHALL use that
depth in its observation and matching. `Shallow` SHALL restrict the governed or permitted module
scope to the exact anchored seam; legacy `Subtree` SHALL retain `::`-delimited descendant matching.
No rule family MAY retain the selected depth only in projection, identity, or misconfiguration
checking while evaluating with a hard-coded subtree. An inbound rule's target match SHALL be
resolved to the module an import path actually denotes (itself when the path names a module
directly, otherwise its longest reachable-module prefix) before the depth comparison, so an
item-form import (`use m::Item;`) reaches `m` exactly as a bare import of `m` itself does; depth
then distinguishes that from an import of only a descendant module's item, never by comparing the
raw import path string (which would conflate an item in `m` with an item in a descendant of `m`).
That path resolution alone is **namespace-blind**, and the target match SHALL compensate for it rather
than inherit it. Rust resolves a module and a value of the same name in different namespaces, so `mod
foo` and `fn foo` may both be declared in one module and a single `use m::foo;` binds **both**.
Observing only the path yields the module reading (the longest reachable module), which under `Shallow`
anchored at that module's own parent resolves to a mere descendant and would not react — while the value
reading reaches the anchored module and MUST. Under `Subtree` both readings lie within the anchored
module, so nothing turns on it there.

The system SHALL close that gap by **observing the value namespace**, and SHALL NOT close it by reacting
on both readings: reacting on both would make an ordinary bare import of a child module react under
`Shallow`, contradicting the exact-seam scenario above, and a narrow false negative SHALL NOT be traded
for a broad false positive. Concretely, an import whose whole path resolves to a module that is a
single-segment child of the anchored module SHALL additionally react when the anchored module itself
declares a value-namespace item (`fn`, `const`, `static`) of that same final segment, and SHALL NOT react
when it declares only the module. This was previously recorded as a live decision on the premise that the
required observation did not exist in this dimension; the premise was false — the definition observation
backing the strict-external local-precedence ladder already reads exactly those names, per module, at
module top level.

An import whose **form cannot bind a value** SHALL NOT react through the value reading, whatever the
anchored module declares. Two forms cannot, and the exclusions rest on what the language admits rather
than on likelihood:

- a **glob** (`use m::foo::*;`) imports the *contents* of the named module and never binds the name
  itself, so with `mod foo` and `fn foo` both declared, calling `foo()` is `error[E0425]`;
- a **`{self}` leaf** (`use m::foo::{self};`, or `{self as f}`) imports the named module, so the same
  declarations give `error[E0423]: expected function, found module` while `foo::INSIDE` compiles.

Both SHALL be stated, and stated together, because neither is distinguishable from a bare import by its
**recorded path**: a glob is stored at its base module with `::*` removed, and a `{self}` leaf is stored
as its prefix module, so all three arrive at the reaction as the same string. The import form SHALL
therefore be carried alongside the normalized path rather than inferred from it. Naming only the glob is
what let the `{self}` form through: the requirement read as complete while admitting a false positive in
the same cell, so this bound SHALL be expressed as the single question "can this form bind a value" rather
than as a list of excluded spellings.

The **observation** carries the remaining bounds, not the resolution, and each SHALL be stated:

- A value name SHALL be read from **declaration-cleaned** source — comments, string and character
  literals, and macro bodies removed — so a name appearing only as text declares nothing. Reading raw
  source instead makes a name written in a comment or a string react, which is a false positive in the
  same cell this rule exists to make correct.
- A value name SHALL be read past an interposed **modifier token** where the walk cannot otherwise recover
  it. `static [mut] NAME` is the one item of that shape: `const fn` / `async fn` / `unsafe fn` recover
  because `fn` is itself an item keyword and the scan reaches the real name on its next step, while `mut`
  is not, so reading the identifier straight after `static` recorded a value named `mut` and left the real
  one unobserved. The skip SHALL apply to the unraw'd token only — `static r#mut` genuinely names the item
  `mut`, and skipping that spelling would attribute a following token to this item, turning the false
  negative into a false positive.
- A value declared inside an **`extern` block** SHALL be observed as a value of the module that contains
  the block. Such a block opens no naming scope, and its item can coexist with a `mod` of the same name
  for precisely the namespace reason this rule exists to observe — so treating its brace as a scope makes
  a real import of the governed module pass silently. The transparency SHALL NOT extend to a brace that
  *does* re-scope, an inline `mod` body above all: a value declared there belongs to the submodule, and
  attributing it upward would trade this false negative for a false positive.
- A value declared inside a macro body, or reaching the module through a re-export, is therefore not
  observed, consistently with every other declaration reader in this dimension, and SHALL direct the
  reaction toward the module reading alone.

#### Scenario: An import binding both a module and a value of the anchored module reacts under Shallow

- **GIVEN** a module `m` declaring both `mod foo` and `fn foo`, and an inbound boundary anchored at `m`
  with `Shallow` depth
- **WHEN** an unauthorized module writes `use m::foo;`, which binds both
- **THEN** it reacts, because the import reaches `m` itself through the value binding

#### Scenario: A `{self}` leaf naming the child module does not react under Shallow

- **GIVEN** a module `m` declaring both `mod foo` and `fn foo`, and an inbound boundary anchored at `m`
  with `Shallow` depth
- **WHEN** an unauthorized module writes `use m::foo::{self};`, in any spelling — aliased, nested inside
  an outer brace group, or beside a sibling leaf
- **THEN** it does not react, the leaf binding the module `foo` and no value of `m`

#### Scenario: A glob import of the child module does not react under Shallow

- **GIVEN** a module `m` declaring both `mod foo` and `fn foo`, and an inbound boundary anchored at `m`
  with `Shallow` depth
- **WHEN** an unauthorized module writes `use m::foo::*;`
- **THEN** it does not react, a glob binding the contents of `m::foo` and never the name `foo`, so it
  reaches only the descendant

#### Scenario: A value name appearing only as text does not react under Shallow

- **GIVEN** a module `m` declaring `mod foo` and no value `foo`, but containing the text `fn foo` inside a
  comment, a string literal, or a macro body
- **WHEN** an unauthorized module writes `use m::foo;`
- **THEN** it does not react, because a name that is only text declares nothing

#### Scenario: An import naming only a child module still does not react under Shallow

- **GIVEN** a module `m` declaring `mod child` and no value named `child`, and the same boundary
- **WHEN** an unauthorized module writes `use m::child;`
- **THEN** it does not react, the import reaching only a descendant — so closing the case above does not
  widen an exact-seam boundary into a subtree one

An inbound rule's importer-side self-import exemption — a file within the protected module's own
subtree is never an inbound importer — is orthogonal to this target match and SHALL NOT be
depth-gated: it holds identically at `Shallow` and `Subtree`, because depth narrows what counts as
*reaching* the protected module, never who counts as *inside* it. The exemption SHALL be applied by
**one** predicate at both the file-level pre-filter and the per-import check, so an excused file is
never read at either depth. A depth-gated pre-filter over a depth-free exemption leaves the reported
violations identical while still reading the file, and any fail-loud condition in that read (an
unreadable file, a `use` tree nested past the scanner's brace-nesting cap) then makes `Shallow` exit 2
where `Subtree` exits 0 — what the exemption excuses MUST NOT be able to decide the exit code. The
external-crate confinement family's own file-level pre-filter is a **different** contract and SHALL
remain depth-sensitive: it may skip a file only when every importer that file can host is permitted,
which under `Shallow` is never, because an inline `mod` inside the permitted file lies outside the
anchored module.

#### Scenario: Outbound rules honor Shallow

- **WHEN** `must_not_import` or `restrict_imports_to` is configured as `Shallow`
- **THEN** an import found only in a descendant module is outside the observation

#### Scenario: Inbound rules honor Shallow

- **WHEN** `must_not_be_imported_by` or `must_only_be_imported_by` protects a module with `Shallow`
- **THEN** an external importer of only a descendant module does not violate the exact-seam
  boundary, while importing the anchored module still reacts — including an item-form import of an
  item declared directly in the anchored module, not only a bare import of the module itself

#### Scenario: Inbound self-import exemption ignores depth

- **WHEN** a module within the protected module's own subtree imports an item declared directly in
  the protected module, under `Shallow`
- **THEN** the importer is exempt as a self-import, exactly as it would be under `Subtree`

#### Scenario: An excused file's content cannot decide the inbound exit code

- **WHEN** a file inside the protected module's own subtree contains source the import scanner fails
  loud on (a `use` tree nested past its brace-nesting cap), and an inbound rule protects that module
  under `Shallow`
- **THEN** the rule reports clean at exit 0, exactly as under `Subtree` — the exemption excuses the
  file from being read at all, rather than excusing its importers after a read that can itself fail

#### Scenario: External confinement honors Shallow

- **WHEN** `confine_external_crate` permits an external crate at an anchored module with `Shallow`
- **THEN** that external crate remains forbidden in a descendant importer, while legacy `Subtree`
  permits the descendant
