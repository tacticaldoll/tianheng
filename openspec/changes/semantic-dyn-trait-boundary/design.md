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

### Decision 1 — Reuse the surface walk and resolver; add a trait-object-recording leaf

What is genuinely reused vs. what must be new — stated precisely so the effort is not
mis-scoped as a "predicate swap":

```
            signature-coupling (today)          dyn-trait-boundary (0.1.2)
  surface   collect_item_exposures decides       REUSED (same governed positions)
            which positions are public/exposed
  resolver  pub use chains; no type-alias expand  REUSED (hunyi::resolve, unchanged)
  bounds    macro-gen, glob, #[path], alias       INHERITED as-is (no new bound)
  leaf      PathCollector → Vec<syn::Path>;       NEW: a visitor that records the
            erases the dyn wrapper node           syn::Type::TraitObject node
  predicate resolved path ∈ forbidden set     →   a recorded TraitObject node is present
```

The honest cost is the **leaf**: detection needs a visitor that overrides `visit_type` /
`visit_type_trait_object` to record the `TraitObject` node (or its rendered finding via the
existing `resolve::type_to_string`, which already renders types without `quote`/`syn`
`printing`). The surface walk that decides *which positions are exposed* is reused; the
collector that observes *what is there* is extended.

**Why X over Y:** an alternative was to *generalize signature-coupling itself* — add a
"shape" variant to `must_not_expose`. Rejected: it would entangle two declarations (a named
forbidden set vs. a shape that takes no operand) in one builder and one spec, and 渾儀's
established pattern is one capability = one builder = one spec, each born when built
(signature-coupling, trait-impl-locality, visibility, forbidden-marker are all separate). So
this ships as a **sibling** `DynTraitBoundary` + `must_not_expose_dyn`, sharing the surface
walk but owning its own declaration, leaf observation, and spec. Refactoring the surface
walk to feed two leaf observations touches signature-coupling's engine, so it is
**behavior-preserving only if** signature-coupling's existing fixtures stay green — that is
the regression gate, not a spec change to signature-coupling (see Risks).

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
