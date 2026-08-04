## REMOVED Requirements

### Requirement: Origin is observed at the registration site, within a process trust boundary

**Reason**: The requirement's subject changes, not only its wording. An origin is no longer taken from
the registration site at all, so the requirement's whole premise — a caller-supplied label observed at
a chosen location, bounded by a process trust boundary because anything the macro can pass hand-written
code can pass too — no longer describes the system. Its trust-boundary paragraphs, its recorded closure
paths, and its `A hand-built entry's asserted origin is taken as given (known bound)` scenario all
describe a residual that this change closes. Replaced by `Origin is derived from the registered type,
not supplied by the registering code` below.

**Migration**: `register_origin!(MyType)` keeps its exact spelling; no adopter source changes. A
registration written **inside the type's own module** — the documented idiom — yields the identical
origin string it yields today, so its `only_origins(...)` entries need no edit. A registration written
**outside** the type's module now yields the type's own module instead of the registration site's, so
any `only_origins(...)` entry naming the registration site must be changed to the type's defining
module; until it is, the boundary reacts fail-closed and the emitted finding names both the observed
origin and the concrete type. Code calling the `#[doc(hidden)]` expansion target directly must use the
macro, as the 0.4.0 migration already directs.

## ADDED Requirements

### Requirement: Origin is derived from the registered type, not supplied by the registering code

A concrete type SHALL opt into an origin via a `macro_rules!` (no proc-macro, no `syn`) whose expansion
target is **generic over that type and takes no origin argument**. Every component of the resulting
registration — the type's identity, its origin, and the type name carried in findings — SHALL be
derived from the type parameter alone. The registering code SHALL have no way to supply, override, or
influence the origin it registers, so an origin naming a module the type does not belong to is
**unrepresentable** rather than detected.

An origin SHALL be the module the type is **defined** in. The system MUST NOT derive an origin from the
registration call's own location, because that location is the caller's choice and therefore a
self-asserted label; deriving it from the type is what makes the origin an observation. For a
registration written inside the type's own module — the documented idiom — the derived origin equals
the registration site's module path, so that idiom's declarations are unaffected.

Because std has no pre-`main` hook, registration SHALL be performed by an explicit startup call (the
macro yields an entry the startup installs); a type that is never registered has no known origin.
Observing the concrete type behind a `dyn Trait` requires the governed trait to carry a `louke::Tracked`
supertrait (rust-1.85-compatible; no trait upcasting), and the concrete type to be `'static`.

The derivation SHALL happen where the type is still a type parameter — inside the macro's expansion
target — because no reverse lookup from a type's identity back to its path exists; a design that
validated a supplied origin at install time instead would be reacting to a disagreement this
requirement makes impossible to express. The prod hot path SHALL be unchanged by this derivation: the
registry still holds `&'static str` origins resolved once at startup, with no lock, no allocation per
crossing, and no dependency beyond std.

#### Scenario: A type's origin is its defining module

- **WHEN** `register_origin!(PostgresRepo)` is written for a `PostgresRepo` defined in module
  `app::infra`, and installed at startup
- **THEN** the origin registry maps that type to the origin `app::infra`, derived from the type itself
  rather than from any label the registering code supplies

#### Scenario: A registration away from the type's module still names the type's module

- **WHEN** `register_origin!(PostgresRepo)` for a `PostgresRepo` defined in `app::infra` is written
  instead inside a startup module `app::startup`
- **THEN** the registered origin is `app::infra`, not `app::startup` — the registration's location does
  not enter the origin at all

#### Scenario: A registration cannot present an origin the type does not have

- **WHEN** code bypasses `register_origin!` and calls its expansion target directly for a type of its
  own, intending to register that type under an allowlisted origin it does not belong to
- **THEN** no such call can be written: the expansion target accepts only the type, so the registered
  origin is that type's own defining module, and a seam crossing by that type reacts fail-closed

#### Scenario: Naming another type's identity registers that type honestly

- **WHEN** code calls the expansion target with a type it does not own, hoping to inject a false
  mapping for it
- **THEN** the registration produced is the correct one for that type, and a second registration of an
  already-registered type fails loud as a duplicate, exactly as two `register_origin!` sites for one
  type already do

### Requirement: The derived origin's shape bounds are stated, not implied

The system SHALL state the following bounds on the derived origin rather than imply a uniform module
path. None of them is a fail-loud class today, because the existing fail-closed allowlist match already
reacts to each one loudly and in the safe direction — a bound is stated where a reaction already covers
it, never used to forbid a future reaction by prose. The origin is derived from the type's own reported
path, whose shape is not uniform across all types:

- A type defined in **another crate** reports that crate's own defining path, which may be a private
  internal module rather than the public path it is re-exported at. Registering a foreign type
  therefore does not attribute it to the registering layer; a type that should carry a layer's origin
  is a type defined in that layer (a newtype), which is also what actually crosses the seam.
- A type defined inside a **function body** reports a path qualified by the enclosing function, which
  is not a module path.
- A **generic** type's arguments are not part of its origin: the origin is taken from the path with its
  argument list removed, so two instantiations of one generic type share one origin. Argument text may
  itself contain path separators and nested argument lists, so the removal SHALL be delimiter-aware
  rather than a search for the last separator.
- A **type alias** reports the aliased type's defining path, not the alias's location, so an alias
  cannot relabel an origin.

The reported path's exact rendering is not guaranteed stable across compiler versions. The system
SHALL keep that instability confined to loud reactions: a rendering change makes an origin stop
matching its allowlist entry, which reacts fail-closed, and SHALL NOT be able to produce a silent pass.
An observed origin SHALL NOT enter a rule key or any recorded baseline identity, so no accepted
violation re-keys on a toolchain change.

#### Scenario: A foreign type's origin is its own defining path

- **WHEN** a type defined in another crate is registered
- **THEN** its origin is that crate's own defining module path for it, so it does not match an
  allowlist entry naming the registering layer, and the crossing reacts fail-closed with a finding
  naming the observed origin

#### Scenario: Two instantiations of one generic type share one origin

- **WHEN** a generic type is registered at two different argument instantiations
- **THEN** both register the same origin — the type's defining module, with the argument list removed —
  including when an argument's own text contains path separators or a nested argument list

#### Scenario: An alias cannot relabel an origin

- **WHEN** a type alias declared in one module names a type defined in another, and the alias is
  registered
- **THEN** the registered origin is the aliased type's defining module, not the alias's

### Requirement: An observed origin matches an allowed entry by equality

The allowlist match SHALL compare an observed origin to each allowed entry by **equality**, never by
module-prefix or subtree containment. A containment match would let a type defined in a module beneath
an allowed entry newly pass a seam that reacts today, converting a live reaction into a silent pass —
the forbidden false negative, reached by loosening the matcher rather than by missing an observation.
Governing a subtree SHALL therefore be expressed by declaring each module that may cross, not by
widening the comparison.

#### Scenario: A type beneath an allowed module does not pass

- **WHEN** a seam allows origin `app::infra` and a crossing object's type is defined in the descendant
  module `app::infra::pg`
- **THEN** the system reacts — the observed origin is not equal to any allowed entry — rather than
  treating the descendant as covered by its ancestor
