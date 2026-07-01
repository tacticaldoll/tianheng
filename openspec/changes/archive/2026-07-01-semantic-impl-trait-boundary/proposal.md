## Why

渾儀 governs two exposure vectors today: a forbidden *named type* (signature-coupling) and a
forbidden *type shape* — dynamic dispatch, `dyn` (dyn-trait-boundary). It has no reaction for the
**existential** shape: a public function that returns `impl Trait` (return-position `impl Trait`,
RPIT). An RPIT at a library seam leaks an **unnameable** type — a consumer cannot name it, cannot
store it in a field without boxing, and the seam silently commits to the hidden type's auto-traits
(`Send`/`Sync`) as part of its semver surface. "This seam returns named, nameable types, never an
existential" is a real architectural intent that is inexpressible today.

This change adds the **existential complement of dyn-trait**: `must_not_expose_impl_trait()`. Where
dyn-trait forbids the *dynamic-dispatch* shape (`dyn`), this forbids the *existential* shape
(`impl Trait` in return position) — the same shape-only reaction on the same `syn` public-surface
observation, a new leaf node (`syn::Type::ImplTrait`) rather than a new dimension. Together the two
say "no erased and no existential types cross this seam". It is the type-shape sibling of dyn-trait
and shares its admission-gate profile.

## What Changes

- **New 渾儀 capability — existential (`impl Trait`) exposure.** A new `ImplTraitBoundary`
  (`in_crate("…").module("…").must_not_expose_impl_trait().because("…")`) reacts when a governed
  module's public API **returns** a written `impl Trait` at any depth in the return type. It is
  **shape-only** (any exposed RPIT reacts; a named-operand `must_not_expose_impl_trait_of([...])`
  is a future depth, mirroring dyn-trait's stair). The finding is the rendered `impl …` shape.
- **Governed surface = existential positions only.** Public free functions, public inherent
  methods, and public trait method declarations — their **return types**. Argument-position
  `impl Trait` (APIT) is deliberately **not** governed: APIT is *universal* (sugar for a generic
  parameter, the caller chooses the type), not existential — it leaks nothing. Trait-*impl* method
  returns are excluded (their shape is dictated by the trait declaration, as dyn-trait excludes
  them). On stable Rust a written `impl Trait` cannot appear in a const/static/field/type-alias
  position, so those are not walked — a stated scope, not a gap.
- **Additive only.** A new boundary type + `SemanticBoundaries` slot + builder + projection,
  parallel to dyn-trait; every existing rule and projection is unchanged; `syn` stays quarantined
  in `hunyi`.

## Capabilities

### New Capabilities
- `semantic-impl-trait-boundary`: a module's public API must not **return** a written `impl Trait`
  (return-position `impl Trait` / RPIT) — the existential complement of dyn-trait's dynamic-dispatch
  shape. Shape-only; reuses the public-surface walk and the `dyn` shape renderer's bound-rendering,
  adding a return-position existential leaf.

### Modified Capabilities
<!-- None. semantic-dyn-trait-boundary and signature-coupling are unchanged; this is a new sibling
     shape-exposure rule on the same syn surface. -->

## Impact

- **Crate:** `hunyi` (渾儀). New `ImplTraitBoundary` + drafts + `must_not_expose_impl_trait()`; a
  `SemanticBoundaries.impl_trait` slot and a `check_all` entry; a return-position existential walk
  (`impl_trait_module_findings`) with a collector for `syn::Type::ImplTrait`, rendered via the same
  `bound_to_string` the `dyn` renderer uses. `syn`-only; no new dependency.
- **Shell (`tianheng`):** `Constitution::impl_trait_boundary(...)` + slot, re-exports, an
  `impl_trait_boundary_json` projection and a `list` markdown section, parallel to dyn-trait.
- **Stated bounds:**
  - **`async fn` is out of scope.** An `async fn` leaks an *implicit* `impl Future` inserted by the
    compiler, not a *written* `impl Trait`; it carries its own syntactic signal (`sig.asyncness`)
    and is a **distinct, named future sibling** (async-exposure), never a silent miss of this
    capability's domain (written `impl Trait`).
  - **Unstable TAIT/ATPIT** (`pub type A = impl Trait;`, associated-type `impl Trait` value) is
    nightly-only and out of scope (Tianheng targets stable).
  - **Macro-generated `impl Trait`** is the inherited 渾儀 macro bound; a written RPIT in the local
    public surface always reacts.
- **SemVer:** additive, non-breaking → folded into the ongoing **0.1.2** (no version bump).
