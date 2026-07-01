## ADDED Requirements

### Requirement: Operand-scoped impl-trait boundary declared in Rust

An operand-scoped impl-trait boundary SHALL be expressed as Rust code on an `ImplTraitBoundary`,
part of the single source of truth, declared on the 渾儀 dimension and composed with the other
dimensions at the gate. It SHALL name a target crate and a module anchor and a closed set of
**forbidden trait operands** via `must_not_expose_impl_trait_of([...])`, a human-readable reason,
and a severity. A returned `impl Trait` in the governed module's public surface whose **principal
trait** canonicalizes to a member of the set is a violation. The shape-only
`must_not_expose_impl_trait()` is unchanged and reacts to any returned `impl Trait`; the operand
variant is a distinct, narrower rule on the same boundary type. The system MUST NOT require TOML,
YAML, Markdown, or any generated policy file to declare or run the boundary.

#### Scenario: Operand-scoped boundary declared in Rust

- **WHEN** a developer writes `ImplTraitBoundary::in_crate("core").module("crate::core").must_not_expose_impl_trait_of(["crate::ports::Port"]).because("the core seam may return impl Iterator but never a dyn-free existential Port")`
- **THEN** an impl-trait boundary is held, targeting `crate::core`, forbidding a returned `impl Trait` whose principal trait is `crate::ports::Port`, with a non-empty reason and a default `enforce` severity, ready to be composed with the semantic dimension at the gate

### Requirement: A returned impl Trait of a forbidden operand is a violation

The system SHALL emit a violation for each returned `impl Trait` in the governed module's public
surface whose principal trait — the first trait bound of the `impl Trait` — canonicalizes to a
member of the forbidden operand set, and SHALL report no violation for a returned `impl Trait`
whose principal trait is outside the set. The principal trait path SHALL be canonicalized and
matched exactly as signature-coupling matches a forbidden type (resolved against the module's
`use` map and re-export closure via `BareFallback::Ignore`, then compared exact-or-module-prefix),
so a re-exported or aliased trait facade matches its defining path; a principal trait that does not
resolve (a bare name with no `use`, a macro-generated or glob/cross-crate re-exported trait) is
dropped — the stated resolver-coverage bound, never a silent pass of a *resolvable* operand.
Auto-trait and lifetime bounds are never operands. The finding is the rendered `impl …` shape, and
the return-position scoping is inherited unchanged (argument-position `impl Trait` and `async fn`
are not governed).

#### Scenario: A returned impl Trait of a named forbidden trait is flagged

- **WHEN** the governed module declares `pub fn make() -> impl crate::ports::Port` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation whose finding is the rendered shape `impl crate::ports::Port`

#### Scenario: A returned impl Trait of an unlisted trait passes

- **WHEN** the governed module declares `pub fn it() -> impl Iterator<Item = u8>` and the boundary forbids only `["crate::ports::Port"]`
- **THEN** the system reports no violation, because the principal trait is outside the forbidden operand set (and a bare `Iterator` does not resolve to the forbidden path)

#### Scenario: A module-prefix operand forbids a subtree of returned traits

- **WHEN** the boundary forbids `["crate::ports"]` (a module prefix) and the module declares `pub fn make() -> impl crate::ports::Port`
- **THEN** the system emits a violation, because the principal trait canonicalizes under the forbidden prefix

#### Scenario: A re-exported trait operand matches its defining path

- **WHEN** the module returns `impl crate::Port`, a `pub use crate::ports::Port` facade of the trait defined at `crate::ports::Port`, and the boundary forbids the defining path `["crate::ports::Port"]`
- **THEN** the system emits a violation, because the returned principal canonicalizes through the re-export closure to the same defining path

### Requirement: Empty operand set degenerates to shape-only, never a silent no-op

The system SHALL treat an **empty** forbidden operand set as "no operand filter — any returned
`impl Trait` is a violation" (the shape-only behavior). `must_not_expose_impl_trait()` constructs
the empty set; `must_not_expose_impl_trait_of([])` therefore reacts to any returned `impl Trait` as
well — a loud over-reaction, never a boundary that reacts to nothing. The system MUST NOT model an
operand-scoped boundary that silently passes every returned `impl Trait`.

#### Scenario: An empty operand list forbids any returned impl Trait

- **WHEN** a boundary is declared with `must_not_expose_impl_trait_of([])` and the module returns any `impl Trait`
- **THEN** the system emits a violation for that returned `impl Trait`, identical to the shape-only `must_not_expose_impl_trait()` reaction — the empty set is unfiltered, not an inert no-op

### Requirement: Reaction, severity, baseline, and projection parity with the shape-only rule

The operand-scoped impl-trait boundary SHALL share the 渾儀 impl-trait reaction contract: findings
fold into the same aggregated report and exit-code outcome (**0** clean, **1** enforce violation,
**2** constitution/scan error such as an unresolvable crate or module); the boundary carries a
severity (`enforce` default, or `warn`) and is gated against the same `Baseline` under the shared
violation identity `(target, rule, finding)`, the finding being the rendered `impl …` shape; and
the rule projects through the existing impl-trait `list` text/JSON/markdown projection, adding a
`forbidden` parameter listing the operand set when non-empty (a shape-only, empty-set boundary
projects unchanged). The implementation SHALL keep the `syn` dependency quarantined in `hunyi`
(no new dependency) and SHALL NOT change the return-position walk.

#### Scenario: An operand violation fails CI

- **WHEN** an enforce-severity operand-scoped impl-trait boundary is violated
- **THEN** the system prints a report naming the target module, the rule, the offending `impl …` shape, and the reason, and exits 1

#### Scenario: An unresolvable target module is a constitution error

- **WHEN** an operand-scoped impl-trait boundary anchors to a crate or module not present in the workspace
- **THEN** the system emits a constitution/scan error and exits 2, never exit 0 and never exit 1

#### Scenario: Severity and baseline behave as for the shape-only impl-trait rule

- **WHEN** a `warn`-severity operand boundary is violated and no enforce boundary is, or an enforce operand boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and an operand violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The operand set projects in list output

- **WHEN** the constitution is projected via `list` (text/json/markdown)
- **THEN** an operand-scoped boundary appears with its target, module, rule, the forbidden operand set, severity, and reason; a shape-only boundary appears exactly as before, with no operand parameter
