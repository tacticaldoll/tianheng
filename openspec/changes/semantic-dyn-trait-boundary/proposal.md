## Why

渾儀 already observes one kind of public-surface exposure — a module's public API must
not expose a forbidden **named type** (`semantic-signature-coupling`). But a module can
leak its most architecturally significant boundary decision without naming any forbidden
type at all: by exposing a **type shape** — `dyn Trait` — across its public seam. A core
crate that declares "my public interface is statically dispatched" has, today, no reaction
that holds that line; the drift is invisible to every existing instrument.

This change deepens the *same* reaction from named-type exposure to **type-shape
exposure**, on the same observation source (the `syn` AST). It is the purest available
demonstration of the project's depth axis: 渾儀 grows by **deepening a proven reaction**,
not by bolting on a new opinion — it reuses signature-coupling's exposed-surface walk
(which public positions are governed), its resolver, and its stated bounds, and adds a
leaf observation that retains type-node structure (a `dyn` node) where the existing
collector kept only resolved paths.

## What Changes

- **New 渾儀 capability — dyn-trait exposure (`must_not_expose_dyn`).** A module may declare
  that its public API must not expose trait-object syntax. The reaction fires when a `dyn`
  type node appears at **any depth** in a governed public type position (return, parameter,
  `pub` field, `pub` const/static, `pub` trait method signature, or a `pub` type alias's own
  target).
- **It is the type-shape analog of signature-coupling, not a lint.** signature-coupling
  forbids an exposed *named type* (`pub fn f() -> infra::Pool`); this forbids an exposed
  *type shape* (`pub fn f() -> Box<dyn Port>`). It does **not** say "don't use `dyn`" — a
  module may use `dyn` internally without reaction; the violation is leaking dynamic
  dispatch across the *declared* public seam. Static-dispatch-at-the-seam has no universal
  right answer (two sane projects declare the opposite), so it is the developer's intent,
  not the tool's opinion.
- **Additive API only.** A new builder on the semantic dimension (`must_not_expose_dyn`),
  gathered into the existing `SemanticBoundaries` container. No existing signature changes;
  no existing reaction changes.
- **Shape-only scope.** `must_not_expose_dyn` takes no trait operand — *any* exposed `dyn`
  reacts. A future `must_not_expose_dyn_of([...])` (only a named trait's `dyn`) is an
  explicit non-goal of this change (born when built, a later depth — not smuggled in now).

## Capabilities

### New Capabilities
- `semantic-dyn-trait-boundary`: a module's public API must not **expose** trait-object
  (`dyn`) syntax. The type-shape complement of `semantic-signature-coupling`: reacts on the
  presence of a `dyn` node at any depth in the governed public surface; `impl Trait` does
  not react (it carries no `dyn` node). Reuses signature-coupling's exposed-surface walk
  (the public positions governed) and the `hunyi::resolve` resolver; adds a new
  trait-object-recording visitor, because the existing `PathCollector` yields resolved
  paths and erases the `dyn` wrapper node.

### Modified Capabilities
<!-- None. signature-coupling's requirements do not change; this is a new sibling capability
     that reuses its implementation machinery. The shared walk/resolver is an implementation
     detail recorded in design.md, not a spec-level change to signature-coupling. -->

## Impact

- **Crate:** `hunyi` (渾儀) only. New `pub` builder + boundary type; extends the
  `SemanticBoundaries` container with one field. The `syn` dependency allowlist
  (`{serde_json, syn, xuanji}`) is untouched — findings render via the existing hand-rolled
  path/type stringification, never `quote`/`syn`'s `printing` feature.
- **Shell:** `tianheng` adds a `Constitution::dyn_trait_boundary(...)` adder mirroring the
  existing `signature_boundary`/`trait_impl_boundary`/… adders, re-exports `DynTraitBoundary`
  from the prelude, and composes the new field through `hunyi::check_all` (reached via
  `constitution.semantic_boundaries()`). The shell's `run(constitution: &Constitution, args)`
  signature is unchanged — the capability is absorbed by the `SemanticBoundaries` container
  the `Constitution` already holds, which is the deliberate reason that container exists.
- **Observation source:** none added. Same `syn` AST, same anchor (a `syn`-resolvable
  module), same bound set (macro-generated `dyn` and non-expanded named `type` aliases are
  the *incidental, already-stated* 渾儀 bounds, never a new essential gap).
- **SemVer:** additive, non-breaking → **0.1.2 patch** under the project's SemVer-honesty.
  It *is* an adopter-facing OpenSpec capability change (a new reaction), unlike the internal
  refactors recorded only in `PROJECT.md`.
