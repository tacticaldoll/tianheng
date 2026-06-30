## Context

渾儀 (`hunyi`) already observes public-surface exposure through
`semantic-signature-coupling`: it walks a module anchor's public API (`collect_item_exposures`
decides which positions are public/exposed) and reacts when a **named forbidden type**
appears, resolving names through the shared `hunyi::resolve` resolver. The exposed-surface
walk, the resolver (follows local `pub use` chains; does **not** expand `type` aliases), and
the CI/baseline/report contract are all in place and proven.

One piece is **not** reusable as-is, and the design must be honest about it: the leaf
collector `PathCollector` (`resolve.rs`) overrides `visit_type_path`/`visit_trait_bound` and
accumulates `Vec<syn::Path>` — for `Box<dyn crate::Port>` it yields `[Box<…>, crate::Port]`
and **erases the `dyn` wrapper**. Signature-coupling only needs the paths; dyn detection
needs the `syn::Type::TraitObject` node, which the path collector discards. So this is **not**
a predicate swap over an identical visitor.

What is missing is a reaction for a module that leaks its most significant boundary decision
**without naming any forbidden type**: exposing a `dyn Trait` **type shape** across its
public seam. This change adds that reaction by *deepening the existing one*: it reuses
signature-coupling's surface walk, resolver, and bound set, and adds a **new
trait-object-recording observation** at the leaf (retaining type-node structure where the
path collector kept only paths). This is the project's "depth, not width" axis made concrete
(PROJECT.md: signature-coupling is "the case that provably earns the AST"; this deepens its
reaction from a named type to a type shape).

## Goals / Non-Goals

**Goals:**
- A new 渾儀 capability `must_not_expose_dyn`: a module may declare that its public API must
  not expose trait-object (`dyn`) syntax, reacting on a `dyn` node at **any depth** in the
  governed public surface.
- Reuse signature-coupling's exposed-surface walk and `hunyi::resolve` **unchanged** — add
  **zero new coverage bounds**; inherit the macro-expansion and named-alias-non-expansion
  bounds as-is.
- Additive, behavior-preserving for every existing reaction; a 0.1.2 patch under the
  project's SemVer-honesty.

**Non-Goals:**
- **Operand-scoped dyn** (`must_not_expose_dyn_of(["crate::Port"])` — only a named trait's
  `dyn`). A later depth, born when built; not smuggled into this change.
- **"Don't use `dyn`" anywhere** — this is not a lint. Internal dynamic dispatch is never a
  violation; only exposure across the *declared* public seam is.
- **Expanding named `type` aliases** to chase a `dyn` hidden behind a private alias — the
  resolver's existing bound is kept, not extended.
- Touching `guibiao`, `xuanji`, or the `hunyi` `syn`-allowlist; touching the
  `run(constitution: &Constitution, args)` shell signature (the capability is reached via the
  `SemanticBoundaries` container the `Constitution` already holds, plus a new typed adder).

## Decisions

### Decision 1 — Mirror the position-walk in a parallel dyn walk; reuse resolver + plumbing; add a trait-object leaf

What is genuinely reused vs. what must be new — stated precisely so the effort is not
mis-scoped as a "predicate swap":

```
            signature-coupling (today)            dyn-trait-boundary (0.1.2)
  positions collect_item_exposures decides        MIRRORED in a parallel walk
            which positions are public/exposed     (collect_item_dyn_exposures) — NOT shared
  resolver  pub use chains; no type-alias expand   REUSED (hunyi::resolve, unchanged)
  plumbing  find_package / resolve_module_items /  REUSED (check_dyn_trait_boundary →
            Outcome / Violation / Baseline          check_all, same as every capability)
  bounds    macro-gen, glob, #[path], alias        INHERITED as-is (no new bound)
  leaf      PathCollector → Vec<syn::Path>;        NEW: DynCollector records the
            erases the dyn wrapper node            syn::Type::TraitObject node, any depth
  predicate resolved path ∈ forbidden set      →   a recorded TraitObject node is present
```

**The position-walk is mirrored, not shared — discovered during implementation, and the
honest design.** A first instinct was to refactor `collect_item_exposures` to feed two leaf
visitors. Two facts in the code make that *change* signature-coupling's behavior rather than
preserve it: (1) for supertraits and associated-type *bounds*, `collect_item_exposures`
pushes the **bare `trait_bound.path`** and does not descend into its generic arguments;
routing those through a shared `Visit` would make signature-coupling newly collect nested
arg paths — a behavior change in the proven capability. (2) The dyn walk must observe a
position signature-coupling **does not cover** — an associated-type *default* (`type T =
Box<dyn …>;`, the `= Type` arm of `TraitItem::Type`, which exposure-governance ignores). So
the two walks are genuinely different and live side by side (`collect_item_dyn_exposures`
beside `collect_item_exposures`, cross-referenced by comment). What *is* shared: the
resolver, the module-scanning, the `cargo metadata` IO, and the whole `Outcome` / `Violation`
/ `Baseline` reaction contract. signature-coupling's engine is **untouched**; its full
existing suite (59 tests) is the regression gate and stayed green.

The leaf is a new `DynCollector` (in `resolve.rs`) overriding `visit_type_trait_object`,
which fires for a `dyn` at any depth; findings render via a new `resolve::trait_object_to_string`
(the existing `type_to_string` returns `None` for a `TraitObject`), never `quote`/`syn`
`printing`.

**Why a sibling, not a generalized `must_not_expose`:** entangling a named-forbidden-set
declaration and an operand-less shape in one builder/spec fights 渾儀's established pattern
(one capability = one builder = one spec, each born when built). So `DynTraitBoundary` +
`must_not_expose_dyn` ships as its own declaration, leaf, and spec.

### Decision 2 — Detection is any-depth; `impl Trait` is clean only for lack of a dyn node

The reaction is the **presence of a `syn::Type::TraitObject` node** anywhere in an exposed
type position, not a match on a top-level shape. Consequently:
- `Box<dyn P>`, `&dyn P`, `Vec<Box<dyn P>>`, `Option<&dyn P>` all react (nested `dyn`).
- `-> impl P` does **not** react — *not* because `impl Trait` is whitelisted, but because it
  contains no `dyn` node. This framing is load-bearing: `-> impl Iterator<Item = Box<dyn
  P>>` therefore **does** react (the `dyn` is exposed to the caller through the item type),
  which a "whitelist `impl Trait`" rule would wrongly pass.

**Why:** "type-shape exposure" means the caller can observe dynamic dispatch in the type
they receive — which is true at any nesting depth. Anchoring the rule to the node's presence
(the visitor already descends into trait-object bounds to collect their paths) is both more
honest and strictly simpler than enumerating allowed/forbidden outer shapes.

### Decision 3 — Public type-alias target reacts; named alias is not expanded

A `pub type Handler = Box<dyn P>;` reacts **at the alias item**: a public type-alias target
is one of the governed positions the surface walk already visits (`collect_item_exposures`
`Item::Type` arm), so the new trait-object leaf observes its target's `dyn` node — the
reaction is newly implemented there, not inherited for free. A `pub fn make() -> Handler`
gets **no extra reaction** by expanding the alias — the resolver does not expand `type`
aliases (its existing bound). So the `dyn` is still caught (at the public alias site), and
only a *private* alias used in a public position escapes — the same stated bound
signature-coupling already carries. (Note: the resolver's `pub use`-chain following, central
to signature-coupling, is **inert** for dyn detection — a re-export carries a *name*, never a
`dyn` node — so it is not cited as load-bearing here.)

**Why X over Y:** the alternative considered was to *teach the resolver to expand `type`
aliases* so `make`'s return is chased to `Box<dyn P>`. Rejected for this change: it widens
the resolver for every 渾儀 capability at once (a cross-capability change with its own
risk/spec), and it is unnecessary here because the public alias declaration *already*
reacts — expansion would only catch the private-alias case, which is the honestly-stated
bound. Keeping the resolver fixed makes this change a pure predicate swap with zero new
bounds; alias expansion can be its own future change if ever wanted.

### Decision 4 — Shape-only scope; operand-scoping is a named future depth

`must_not_expose_dyn` takes no trait operand. The spec records `must_not_expose_dyn_of` as
an explicit non-goal so a reviewer does not read the missing operand as an oversight. This
keeps the first depth-benchmark a *pure* shape predicate (the cleanest possible
demonstration) and leaves operand-scoping as a later born-when-built capability.

**Honesty about declarativeness:** because the rule takes no operand, it is *less*
parametric than signature-coupling (which always names a forbidden set). Its only
project-specific intent knob is the **anchor** — the declared module seam. It is therefore
declarative *by virtue of seam scoping* ("this seam is statically dispatched"), not by virtue
of an operand; the not-a-lint gate is passed by the anchor, not by per-type intent. A future
reviewer should not mistake the shape-only form for a global lint: a global "no `dyn`
anywhere" toggle would be a lint; a `dyn`-free *declared seam* is architecture.

## Risks / Trade-offs

- **[Mistaken for a lint]** "no `dyn` in the public API" can read like clippy. → Mitigation:
  the boundary governs *exposure across a declared module seam*, never internal use; and
  static-dispatch-at-the-seam has no universal right answer (two projects declare the
  opposite). The not-a-lint gate is passed in the proposal and the spec's "used only
  internally is clean" scenario pins it.
- **[Private-alias false negative]** a `dyn` hidden behind a *private* alias used in a public
  position is not observed. → Mitigation: it is the *same* incidental bound signature-coupling
  already states, surfaced explicitly in the spec ("A private alias hiding a dyn … is a stated
  bound"); the public-alias case — the common one — does react. No *new* essential gap.
- **[Macro-generated dyn]** a macro expanding to a public `dyn` is invisible. → Mitigation:
  the universal 渾儀 macro-expansion bound, stated, not silently passed.
- **[impl-Trait nesting surprise]** users may expect `-> impl Trait` to always be clean. →
  Mitigation: Decision 2's node-presence framing is documented; the spec carries both the
  clean (`impl P`) and the reacting (`impl Iterator<Item = Box<dyn P>>`) scenarios.
- **[Finding-render collision → baseline-masked false negative]** *(found by adversarial review
  of the implementation.)* The finding identity is `(target, rule, finding)`, so if the renderer
  maps two structurally-distinct trait objects to the same string, a baselined one masks a new
  one — a silent pass, the one forbidden bug. The first draft rendered any unrenderable bound as
  `dyn _`, which the boxed-closure family (`dyn Fn(…)`, `dyn FnMut(…)`, the *most common* exposed
  trait object) and `dyn Foo<…dyn…>` all hit. A **second review round** found the same collision
  class still open for **associated-type bindings** (`dyn Iterator<Item = u8>` vs `<Item = u16>`
  — both → `dyn Iterator<_>`; the most common assoc-bound dyn) and **macro/bare-fn generic args**
  (→ `dyn _`). → Mitigation: the renderer now renders every *observable* distinguishing payload —
  the `Fn(…) -> …` shape, associated-type/const bindings (`Item = T`), lifetimes, simple const
  generics, macro *names* (`bar!`), fn-pointers, and nested trait-objects (and `*ptr`/`!`). Unit
  tests pin closures, assoc-type, and macro/fn-pointer distinctness. The **irreducible residual**
  — a complex const-generic *expression*, a same-named macro with different args, a `verbatim`
  type — cannot be rendered without `quote`/token-printing (architecturally forbidden) or
  edit-unstable spans, so it is a **stated rendering bound** (declared in the spec and the
  `trait_object_to_string` doc): the dyn still *reacts*, only baseline-dedup granularity is
  bounded for those exotic shapes. This is the same `(target, rule, finding)` render-granularity
  bound `semantic-trait-impl-locality`'s `(impl for <self_ty>)` finding already carries — the
  honest limit, not a silent pass.

## Migration Plan

No migration. Purely additive: adopters opt in by declaring a `DynTraitBoundary`. No
existing reaction, exit code, baseline identity, or shell signature changes. Self-governance
and the existing 渾儀 tests must stay green; new fixtures cover the scenarios above. Rollback
is removing the boundary declaration (and, if needed, the builder) — nothing else depends on
it.

## Open Questions

- **Finding wording for the alias-item site.** The use-site findings read "`pub fn connect`
  exposes `dyn crate::Port`"; the alias-item finding needs a parallel phrasing (e.g. "`pub
  type Handler` exposes `dyn crate::Port`"). To be settled in implementation against the
  existing report renderer; the spec fixes the *contract* (anchor + rule + finding + reason),
  not the exact string.
