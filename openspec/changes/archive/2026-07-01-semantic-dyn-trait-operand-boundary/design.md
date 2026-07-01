## Context

渾儀's shape-only dyn-trait-boundary (`hunyi`, v0.1.2) walks a module's public surface and
collects every `dyn` node, rendered to a stable finding string (`dyn crate::Port`,
`dyn Iterator<Item = u8>`, …) by `DynCollector` / `collect_item_dyn_exposures`; `dyn_module_findings`
returns the sorted, deduplicated shapes and any is a violation. It deliberately needs no `use`-map
or re-export closure — the reaction is on the *presence* of a `dyn`, so no name resolution happens.

Signature-coupling (`module_findings`) is the sibling that *does* resolve: it canonicalizes each
exposed type path (`canonical_path_str` → `resolve_path(BareFallback::Ignore)` →
`canonicalize_through_reexports`) and filters against a canonicalized **forbidden set**
(`matches_forbidden`, exact-or-module-prefix). This change composes those two proven halves: the
shape-only rule's `dyn` walk, plus signature-coupling's forbidden-set resolution — applied to the
trait *inside* the `dyn`.

## Goals / Non-Goals

**Goals:**
- A `must_not_expose_dyn_of([trait paths])` builder: a `dyn` node in the governed public surface
  whose **principal trait** canonicalizes to a member of the forbidden set is a violation; a `dyn`
  of any other trait passes. Reuse the shape-only walk, the shared resolver, and the
  reaction/projection/baseline/exit-code contract; the only new logic is principal-trait
  extraction + operand matching. Same `syn` source, no new crate, hermetic.

**Non-Goals:**
- Changing `must_not_expose_dyn()` — it stays shape-only (any `dyn` reacts).
- A new observation source, a resolved/whole-graph read, or touching the `syn` quarantine.
- Matching auto-trait / marker bounds (`Send`, `Sync`, lifetimes) as operands — a trait object
  has exactly one principal (non-auto) trait; only it is the operand.

## Decisions

### Decision 1 — One boundary type; the operand is a forbidden set, empty ⇒ shape-only

`DynTraitBoundary` gains a `forbidden_operands: Vec<String>` field. `must_not_expose_dyn()`
constructs it **empty** — meaning "no operand filter, any `dyn`", the unchanged shape-only
behavior. `must_not_expose_dyn_of([...])` constructs it **non-empty**. The reaction is:

```
   a dyn node is a finding  ⇔  forbidden_operands.is_empty()                 // shape-only: any dyn
                              ∨ forbidden_operands ∋ canon(principal trait)  // operand-scoped
```

One struct, one check path, one projection — maximal parity, minimal surface. **Empty ⇒ any**
is the one subtlety, and it is *safe by direction*: `must_not_expose_dyn_of([])` (an empty operand
list) degenerates to "forbid any `dyn`" — a **loud over-reaction**, never a silent no-op. This
respects the prime directive (a false negative is the one forbidden bug): a mis-declared empty
operand set over-reports, it does not silently pass. (A separate `Any | Of(set)` enum was
considered; it would make `Of([])` a *silent no-op* boundary — governance that reacts to nothing —
which the drift law forbids. The empty-⇒-any collapse is the safer modeling.)

### Decision 2 — The principal trait is the FIRST trait bound (guaranteed by grammar); resolve it like the forbidden type set

For each collected `dyn` node (`syn::Type::TraitObject`), the **principal trait** is the **first
`TypeParamBound::Trait`** in its `bounds`. Rust's grammar guarantees the principal (base) trait is
syntactically first; any auto-trait (`Send`, `Sync`) or lifetime bound can only *follow* it (`dyn
Send + Foo` is a compile error). So the operand is `bounds`' first trait bound, full stop — we do
**not** skip bounds by name (verified against `syn`: `dyn Port + Send + 'a` → `[Trait(Port),
Trait(Send), Lifetime]`, first = `Port`; `dyn Send` → `[Trait(Send)]`, first = `Send`, which is
correctly its own principal — a name-skipping rule would wrongly find no principal here). Trailing
auto-trait markers are therefore never the matched operand.

The principal trait's path (segments only, generic/parenthesized args dropped) is canonicalized and
matched exactly as signature-coupling matches a forbidden type: `canonical_path_str` →
`resolve_path(uses, module, BareFallback::Ignore)` → `canonicalize_through_reexports(reexports)` →
`matches_forbidden(canon, forbidden)`. So a forbidden entry may be an exact trait path
(`crate::ports::Port`) or a module prefix (`crate::ports`), and a re-exported/aliased trait facade
matches its defining path — closing the same re-export false negative the sibling rules close.

Consequence: unlike the shape-only findings, the operand findings pass **needs** the module's
`use`-map and re-export closure (to resolve the principal trait), exactly as `module_findings`
does. The shape-only path stays resolution-free; the operand path adds resolution. A principal
trait that does not resolve under `BareFallback::Ignore` — a **bare** name with no `use`
(`dyn Fn(…)`, `dyn Iterator<…>`, a bare `dyn Send`), a macro-generated or glob/cross-crate
re-exported trait — is dropped, exactly as signature-coupling drops an unresolvable exposed type.
This is the **stated resolver-coverage bound**: operand-scoping targets *resolvable* traits (the
common case, a local `crate::…` trait or one reachable through `use`/re-export); it is never a
silent pass of a *resolvable* operand. (It also makes "forbid `Send`" a no-op — a bare auto-trait
does not resolve — which is the intended outcome, reached through the resolver rather than a
name-skip.)

### Decision 3 — Finding is the rendered `dyn …` shape (parity); projection carries the operand set

The finding stays the existing rendered shape (`dyn crate::Port`) — it already names the trait,
so it "makes the operand explicit" for free and keeps baseline identity `(target, rule, finding)`
byte-identical to the shape-only rule's. The rule label is unchanged (`must not expose dyn`); the
`list` JSON/text/markdown projection gains a `forbidden` parameter listing the operand set **when
non-empty** (mirroring signature-coupling's `forbidden`), so a reader sees which traits the
boundary scopes to; an empty set (shape-only) emits no such param, leaving the shape-only
projection unchanged.

### Decision 4 — Reuse the check / severity / baseline path unchanged

The operand variant is the same `DynTraitBoundary`, so it flows through the existing
`check_dyn_trait_*` reaction, `Severity` (`enforce`/`warn`), `Baseline` gating, and exit-code
contract (0 clean / 1 enforce violation / 2 constitution or scan error — an unresolvable
crate/module stays a constitution error). No new reaction plumbing.

## Risks / Trade-offs

- **[Empty-operand surprise]** `_of([])` forbids all `dyn`. → Mitigation: safe-by-direction (loud
  over-reaction, not a silent pass); stated in the builder doc and spec.
- **[Principal-trait resolution bound]** a macro/glob-re-exported principal trait is unresolvable
  and thus unmatched. → the inherited, *stated* 渾儀 resolver bound (never a silent pass of a
  resolvable operand); identical to signature-coupling.
- **[Confusion with shape-only]** an adopter may expect `_of` to also catch other `dyn`s. →
  Mitigation: the two builders are documented as distinct; shape-only forbids all, `_of` forbids
  the named subset.

## Migration Plan

No migration. Purely additive: adopters opt in with `must_not_expose_dyn_of([...])`; the shape-only
`must_not_expose_dyn()` and every existing dyn finding are unchanged. Self-governance and the
existing 渾儀 tests must stay green; new tests cover principal-trait matching (exact + module
prefix), a re-exported/aliased operand, a non-matching `dyn` passing, auto-trait markers ignored,
the empty-⇒-any degeneracy, and projection/severity/baseline parity. Rollback removes the builder
and the operand field.

## Open Questions

- **Convenience over module-prefix.** Whether to document module-prefix operands
  (`must_not_expose_dyn_of(["crate::ports"])` forbids any `dyn` of a trait under `crate::ports`) as
  a first-class idiom or leave it implicit in `matches_forbidden` parity. Leaning: state it in the
  builder doc (it falls out of the shared matcher for free), no extra API.
