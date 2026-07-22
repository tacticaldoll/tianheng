# semantic-trait-impl-exposure Specification

## Purpose
An opt-in depth (`.including_trait_impls()`) that extends signature-coupling from a module's own
items to a trait `impl` block's **impl-site-authored** positions: the trait reference's generic
arguments, the `Self` type, associated type/const bindings, the impl's own generics / `where`-clause
(including const-generic parameter types), and each method's **return** type. Trait-*dictated*
positions — method parameters and the receiver — stay out of scope, since they belong to the trait
definition (which signature-coupling already governs). Opt-in, not default-on: a bare boundary
declared trait impls out of scope, and the impl-authored/trait-dictated split is a real narrowing
choice, so this is additive depth on the patch line, not a false-negative closure.

## Requirements
### Requirement: Opt-in modifier deepens signature-coupling to trait impls

Trait-impl exposure SHALL be declared as an **opt-in modifier on the existing signature-coupling
`SemanticBoundary`**, not as a new boundary type. A boundary written
`SemanticBoundary::in_crate(c).module(m).must_not_expose(p).including_trait_impls().because(r)`
SHALL forbid the same type set `p` at the same module anchor `m`, deepening the observed surface to
include the anchored module's trait `impl` blocks. A boundary WITHOUT `.including_trait_impls()`
SHALL keep the v1 signature-coupling semantics (trait impls out of scope). The system MUST NOT
require TOML, YAML, Markdown, or any generated policy file to declare or run this boundary.

#### Scenario: The opt-in modifier deepens the same boundary

- **WHEN** a developer writes `SemanticBoundary::in_crate("app").module("crate::domain").must_not_expose("crate::infra").including_trait_impls().because("domain must not leak infra even through impl-site contracts")`
- **THEN** a semantic boundary is held, anchored to `crate::domain` in crate `app`, forbidding exposure of `crate::infra` across both the signature-coupling surface AND the module's trait `impl` blocks, with a non-empty reason and a default `enforce` severity

#### Scenario: Without the opt-in, trait impls stay out of scope

- **WHEN** a boundary uses a bare `must_not_expose("crate::infra")` (no `.including_trait_impls()`) and the module declares `impl From<crate::infra::DbPool> for Service`
- **THEN** the system does not react to the trait impl (the v1 signature-coupling bound is preserved), consistent with `semantic-signature-coupling`

### Requirement: Impl-site-authored positions govern trait-impl exposure

With `.including_trait_impls()` enabled, the system SHALL observe, for each trait `impl` block whose
text appears in the governed module's source, the **impl-site-authored** positions and react to a
forbidden type that appears in any of them. The observed positions SHALL comprise exactly:

1. the trait path's generic arguments (position `trait-arg`);
2. the `Self` type, both when a forbidden type **is** the Self type and when it is **nested** within it, including the Self type's generic arguments (position `self`);
3. associated type/value bindings authored in the impl, `type Assoc = …` (position `assoc {name}`);
4. the impl block's own generic bounds and `where`-clause, keyed by the bounded type (position `where {bounded-type}`);
5. the impl method **return type as written at the impl site** (position `method {name} return`).

A forbidden type reached only through an impl-site position SHALL react even when it appears in no
signature-coupling position.

#### Scenario: A forbidden type in a trait's generic argument is a violation

- **WHEN** the governed module declares `impl From<crate::infra::DbPool> for Service` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::DbPool`, exposed at the `trait-arg` position

#### Scenario: A forbidden type that is the Self type is a violation

- **WHEN** the governed module declares `impl SomeTrait for crate::infra::Forbidden {}` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Forbidden`, exposed at the `self` position — the Self type is the impl seam's subject, the same coupling signature-coupling already treats as exposure for a `pub fn` parameter

#### Scenario: A forbidden type nested in the Self type is a violation

- **WHEN** the governed module declares `impl SomeTrait for Vec<crate::infra::DbPool>` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::DbPool`, exposed at the `self` position

#### Scenario: A forbidden type in an associated-type binding is a violation

- **WHEN** the governed module declares `impl Iterator for Service { type Item = crate::infra::Secret; … }` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Secret`, exposed at the `assoc Item` position

#### Scenario: A forbidden trait in the impl where-clause is a violation, keyed by the bounded type

- **WHEN** the governed module declares `impl<T> SomeTrait for Service<T> where T: crate::infra::Secret {}` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Secret`, exposed at the `where T` position (impl generic bounds and `where`-clause bounds share the `where` position, keyed by the bounded type so two distinct bounds never collapse)

#### Scenario: An impl-refined method return type (RPITIT) is a violation

- **WHEN** a trait declares `fn items(&self) -> impl Iterator<Item = u8>;` and the governed module declares `impl Port for Service { fn items(&self) -> crate::infra::Iter { … } }`, refining the opaque return to a concrete type, under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Iter`, exposed at the `method items return` position — the concrete return is authored at the impl site and is public API, so leaving it unobserved would be a false negative

### Requirement: Method parameters and receiver are trait-dictated non-goals; the written return is observed

The system SHALL observe an impl method's **return type as written at the impl site**, but SHALL NOT
observe its **parameter types** or **receiver**. A trait impl method's parameter and receiver types
are invariant with the trait declaration — they cannot be refined at the impl site — so they are
trait-dictated and governed at the trait's own definition (`semantic-signature-coupling`). The return
type, by contrast, MAY be refined at the impl site (return-position `impl Trait` in traits / async fn
in traits, stable), so a concretely-written return can expose a type the impl author chose. The
system SHALL observe the written return type **without classifying it as refined vs. trait-dictated**:
distinguishing the two would require resolving the (possibly foreign) trait definition — an essential
gap — so a concretely-written return SHALL be observed whether it refines an opaque trait return or
matches a concrete one. This is a deliberate, documented scope, never a silent claim.

#### Scenario: A method parameter type is not an impl-site violation

- **WHEN** a trait declares `fn put(&self, x: crate::infra::DbPool);` and the governed module declares `impl Sink for Service { fn put(&self, x: crate::infra::DbPool) { … } }` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system does not emit an impl-site violation for the parameter (it is invariant with the trait and governed at the trait's own definition), rather than claiming the impl clean by silence

#### Scenario: A concrete associated type reacts at its authored position, not the trait-dictated method that uses it

- **WHEN** the governed module declares `impl Iterator for Service { type Item = crate::infra::Secret; fn next(&mut self) -> Option<Self::Item> { … } }` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation at the `assoc Item` position (where the concrete type is authored); the `next` return `Option<Self::Item>` names no concrete forbidden path (the `Self::Item` reference does not resolve to a path) and does not double-fire

### Requirement: Implementing a forbidden trait is an out-of-scope non-goal

The system SHALL NOT emit a trait-impl-exposure violation when the forbidden path is the **trait being
implemented** rather than a type it exposes. `impl crate::infra::Sealed for crate::domain::Service {}`
acquires a forbidden trait for a local type; that is the concern of forbidden-marker
(`must_not_acquire`) or trait-impl-locality (`only_implemented_in`), not forbidden-**type** exposure.
Folding it into `must_not_expose(...).including_trait_impls()` would duplicate those reactions and
violate minimalism. This exclusion SHALL be a documented non-goal, stated so a reader never mistakes it
for a silent pass.

#### Scenario: Implementing a forbidden trait is a stated non-goal, not a silent pass

- **WHEN** the governed module declares `impl crate::infra::Sealed for crate::domain::Service {}` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system does not emit a trait-impl-exposure violation for the trait path itself; that concern is documented as belonging to `must_not_acquire` / `only_implemented_in`, not claimed clean by silence

### Requirement: Position-qualified seam identity prevents baseline masking

A trait-impl exposure finding SHALL be **seam-qualified with the impl-site position**, rendered as
`{canonical type} exposed by impl {Trait} for {SelfTy} ({position})`, where `{position}` is one of
`trait-arg`, `self`, `assoc {name}`, `where {bounded-type}`, or `method {name} return`. Two positions
that expose the **same** forbidden type SHALL therefore produce **distinct** findings, so baselining
one exposure MUST NOT mask a new exposure of the same type at another position under the
`(target, rule, finding_key)` baseline identity (the one forbidden false negative). Findings SHALL share
the `(target, rule)` of the signature-coupling boundary that carries them.

#### Scenario: Two positions in one impl exposing the same type stay distinct findings

- **WHEN** one impl exposes `crate::infra::DbPool` at both the `trait-arg` and `self` positions, and the `trait-arg` finding is recorded in the baseline as accepted
- **THEN** the `self` finding still reacts: its seam names its own position, so the baseline identity `(target, rule, finding_key)` does not mask it

#### Scenario: Two where-bounds on distinct type parameters exposing the same type stay distinct

- **WHEN** one impl declares `impl<T, U> Trait for Service<T, U> where T: crate::infra::Secret, U: crate::infra::Secret {}`
- **THEN** the system emits two distinct findings, keyed `where T` and `where U`, so baselining one does not mask the other

#### Scenario: The report names the position

- **WHEN** the impl `impl crate::api::Port for crate::domain::Service` in crate `app` refines its return to `crate::infra::Iter` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the report names the finding `crate::infra::Iter exposed by impl crate::api::Port for crate::domain::Service (method items return)`, the boundary's reason, and indicates CI failed

### Requirement: Resolution, matching, and reaction reuse signature-coupling

Trait-impl exposure SHALL reuse signature-coupling's forbidden-type matching (exact resolved path
OR `::`-delimited module prefix) and the shared `hunyi::resolve` resolver with the **same bare-name
fallback policy as signature-coupling** — a bare, unqualified local name SHALL NOT be resolved
against the current module (`BareFallback::Ignore`), so an impl position naming a bare local name is
not turned into a same-module false positive. Resolution SHALL follow in-scope `use`s (incl.
renames), `crate`/`self`/`super`-relative paths, and local `pub use` re-export chains. A type whose
resolution requires a glob import, a macro-generated type, a `cfg_attr`-wrapped `#[path]` module (an
**unconditional** `#[path = "…"]` module is followed and observed), or full
inference SHALL be an inherited OUT-OF-SCOPE bound, never a silent pass, and no new hole SHALL be
introduced. Within the resolved scope there SHALL be no false negative. Trait-impl exposure findings
SHALL fold into the same exit-code contract (**0** clean, **1** enforced violation, **2** constitution
/scan error), the same `Baseline` gating, and the same severity semantics (`enforce` default, `warn`)
as signature-coupling.

#### Scenario: A bare local name in an impl position is not a false positive

- **WHEN** the governed module declares `impl From<DbPool> for Service` where `DbPool` is a bare, unqualified name resolvable only against the current module, under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system does not resolve the bare name against the current module (parity with signature-coupling's `BareFallback::Ignore`) and does not emit a same-module false positive

#### Scenario: A re-exported forbidden type in an impl position resolves and reacts

- **WHEN** the governed module declares `use crate::facade::DbPool;` (where `crate::facade` declares `pub use crate::infra::DbPool;`) and declares `impl From<DbPool> for Service` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system follows the `pub use` chain, resolves `DbPool` to `crate::infra::DbPool`, and emits a `trait-arg` violation rather than silently passing it

#### Scenario: A glob-imported type in an impl position is a documented coverage bound

- **WHEN** the governed module declares `use crate::infra::*;` and `impl From<DbPool> for Service` with `.including_trait_impls()`
- **THEN** the system does not claim to observe it (inherited glob bound), rather than silently asserting the boundary is clean

#### Scenario: A baselined trait-impl exposure does not fail; a new one does

- **WHEN** an `enforce`-severity boundary's only trait-impl exposures are all present in the baseline
- **THEN** the system reports them accepted and does not fail; and WHEN a new exposure not in the baseline appears at any position, the system fails the reaction (exit 1)

### Requirement: The opt-in is projected in the declared law

Because `.including_trait_impls()` changes what the boundary reacts to, the `list` projection SHALL
show it, so the projected law matches the enforced law (constitution-projection's "rule together with
its parameters", applied to this parameter). A boundary WITHOUT the opt-in SHALL project exactly as
before (byte-unchanged), so the flag appears only when it is set. Specifically: the **text**
projection SHALL append ` (including trait impls)` to the rule line; the **JSON** projection SHALL
carry `"including_trait_impls": true` (present only when set); and the **Markdown** projection SHALL
carry it among the boundary's rule parameters (it is a non-structural field, surfaced generically —
no projection-specific code). The projection remains a pure projection and SHALL NOT react.

#### Scenario: The text projection shows the opt-in

- **WHEN** `list` renders a semantic boundary declared with `must_not_expose("crate::infra").including_trait_impls()`
- **THEN** the rule line reads `must not expose: crate::infra (including trait impls)`

#### Scenario: The JSON projection carries the flag only when set

- **WHEN** `list --format json` renders a boundary with `.including_trait_impls()`
- **THEN** its object carries `"including_trait_impls": true`; and a boundary WITHOUT the opt-in carries no `including_trait_impls` key (byte-unchanged from prior output)

#### Scenario: The Markdown projection surfaces the opt-in as a rule parameter

- **WHEN** `list --format markdown` renders a boundary with `.including_trait_impls()`
- **THEN** the boundary's rule parameters include `including_trait_impls: true`, carried generically from the JSON with no projection-specific code

### Requirement: Trait-impl exposure uses observed structural seams

Trait-impl exposure facts SHALL encode trait, canonical self type, associated item role/name, and
forbidden subject where observed. A traversal position or impl/item ordinal SHALL NOT substitute for
an unrenderable structural role.

#### Scenario: Inherent and trait-impl seams stay distinct
- **WHEN** the same subject appears in an inherent item and a trait-impl item on one self type
- **THEN** their owner/trait/item roles keep the identities distinct

#### Scenario: An unrenderable seam fails safely
- **WHEN** ordinary rendering cannot distinguish two structural seams
- **THEN** an observed discriminator separates them or scanning fails loud, never a positional fallback
