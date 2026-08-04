## Context

漏刻's prod face resolves a crossing `dyn` object's concrete `TypeId` through a write-once origin
registry and matches the seam's allowlist fail-closed. The registry is populated at startup from
`OriginEntry` values, and each one is built by `register_origin!`:

```rust
register_origin!(Repo)
  └─▶ OriginEntry::__from_register_origin(
          TypeId::of::<Repo>(),   //  a value
          module_path!(),         //  a value
          type_name::<Repo>(),    //  a value
      )
```

All three arguments are ordinary values, so a hand-written call fabricates all three at once:

```rust
__from_register_origin(TypeId::of::<Rogue>(), "app::blessed", "BlessedAdapter")
```

`install` accepts it, `Rogue` crosses a seam declared `only_origins(["app::blessed"])`, and nothing
reacts. Reproduced against the real public API during the 0.3.1 sweep and re-derived by an
independent review of the 0.4.0 window.

The 0.4.0 window made this bound honest everywhere it is claimed (spec Purpose and requirement name,
`crates/louke/README.md`, the `register_origin!` doc, `PROJECT.md`'s Core Contract), and pinned the
residual with `a_hand_built_origin_entry_is_accepted_a_known_trust_bound`. What it did not do is close
it. Two previously recorded closure paths were tested and **both fail**:

| Recorded path | Why it fails | Evidence |
|---|---|---|
| `#[track_caller]` / `std::panic::Location` | yields a **file path**; an origin's whole vocabulary is a module path, so adopting it redefines origin *and* breaks every declaration | prior window |
| a **proc-macro**, "so no constructor need be public" | a proc-macro is expanded into its **caller's** crate and resolved there, exactly as a `macro_rules!` is — it has no privilege its caller lacks | three-crate probe: `error[E0603]: function` `hidden_constructor` `is private`, reported at the *consumer's* call site |

Both share one shape: hunting for a **macro** that can pass something hand-written code cannot. No
such macro exists, in either macro system.

**The residual was narrower than documented, for a Tianheng-governed workspace.** 圭表's inline
symbol-path confinement already reacts to the hand-written bypass at CI — measured against a real
adopter workspace with the family patched to local source:

```rust
ModuleBoundary::in_crate("my_app").module("crate")
    .must_not_call_inline("louke::OriginEntry")
    .strict_external()                       // required — see below
    .depth(ScanDepth::Subtree)
```

| spelling | reaction |
|---|---|
| `louke::OriginEntry::__from_register_origin(…)` | exit 1 |
| `::louke::OriginEntry::__from_register_origin(…)` | exit 1 |
| `use louke::OriginEntry as OE; OE::__from_register_origin(…)` | exit 1 (the resolver follows the rename and reports the canonical path) |
| any of the above **without** `.strict_external()` | clean — the default resolver does not classify an external crate's paths |

A control boundary confining a local path was verified to react first, so the clean row is the rule not
observing, not the probe failing. The bound worth stating with it: such a boundary scans the adopter's
own workspace, so a third-party **dependency** that hand-builds an entry is outside it. This does not
reduce the case for closing the gap in the crate — a crate's own guarantee must not depend on the
adopter also adopting 圭表 and remembering `.strict_external()` — but it does mean the 0.3.x residual was
CI-preventable, which the prose never said.

**Capability pressure, recorded rather than faked.** The invariant this change establishes *inside*
louke — the origin is never taken from the call site — has **no observation source** in the current
vocabulary. Measured: a `must_not_call_inline("std::module_path").strict_external()` boundary stays
clean against `std::module_path!()`, `::std::module_path!()`, and bare `module_path!()` alike, because
the inline symbol scan does not observe a **macro invocation** as a symbol path. Under the drift law
("no target type or name without a reaction") that invariant therefore must not be given a name in the
constitution; it stays a test, and "macro invocation as an observable inline call" is a capability
candidate for `shape-capability`, not something to assert as law. Prose that pulls toward a shape
Tianheng cannot react to is the open loop this project exists to close.

## Goals / Non-Goals

**Goals:**

- Make a forged origin **unrepresentable**, not detected — the same move this window already made for
  the cfg-collided owner label (delete the single-candidate resolver rather than patch three call
  sites).
- Keep `register_origin!(MyType)`'s adopter-facing spelling byte-identical, so no adopter edits source.
- Keep the probe hot path exactly as it is: std-only, lock-free, `&'static str` origins, no `syn`, no
  new work per crossing.
- Retire the residual's whole footprint — prose, scenario, and both pinning tests — in the same
  change that closes it, so nothing keeps describing a limit that no longer exists.

**Non-Goals:**

- **Allowlist prefix matching.** Not part of this change and actively refused: today an observed
  origin must *equal* an allowed entry, and loosening to a `::`-delimited prefix would let a type
  defined under a descendant module newly pass a boundary that reacts today. That is a false
  negative, the exact bug class this change exists to remove.
- Defending against an adversary who can edit the governed workspace. Deleting a `RuntimeBoundary`
  declaration remains possible and is out of scope for every in-repo governance tool; what changes
  here is that *product* code can no longer forge an observation while the law stays intact.
- Any change to the CI face. `audit_probe_coverage` audits seams and probes and never observes an
  origin.

## Decisions

### 1. Derive the whole entry from a type parameter, not from arguments

```
        today                                   after
┌──────────────────────────────┐      ┌──────────────────────────────┐
│ __from_register_origin(       │      │ __of::<T>()                   │
│   TypeId::of::<T>(),   ←forgeable   │        ▲                      │
│   module_path!(),      ←forgeable   │        │                      │
│   type_name::<T>(),    ←forgeable   │  a type parameter cannot      │
│ )                             │      │  lie about itself            │
└──────────────────────────────┘      └──────────────────────────────┘
  entry content = what the caller said   entry content = f(T)
```

The constructor becomes generic and **argument-free**: `TypeId`, the origin, and the type name are all
derived from `T` inside it. A forger's only remaining move is `__of::<Rogue>()`, which honestly
reports `Rogue`'s own module. Naming someone else's type — `__of::<Blessed>()` — produces the *correct*
mapping and additionally collides with the existing duplicate-registration panic.

**Why the derivation cannot live in `install` instead:** std has no `TypeId → path` reverse lookup, so
by the time `install` sees an entry the type is gone. The derivation must happen where `T` is still a
type parameter, which is the macro's expansion target. That is the only available position, and it is
why "keep the signature, validate inside `install`" is not an option.

**Falsifier attempted against the unforgeability claim — a dependency aliased to the victim's own crate
name.** If the reported path's leading crate segment could be chosen by the *importer*, a rogue crate
pulled in as `composed_app = { package = "evil", … }` would define
`adapters::blessed::Rogue` and report a blessed-looking origin. Tested: the alias does **not** leak
into the reported path, which names the defining crate — `evil::adapters::blessed::Rogue` — so the
importer's chosen name is irrelevant. Publishing a crate literally named `composed_app` does not work
either: two crates of one name in a single build graph collide at compile time (`E0464`, observed while
constructing the probe). Forging the crate segment therefore requires being the crate, and forging the
module segments requires writing inside the victim's own module — which is editing the law's subject,
already out of scope. The claim survives an attack rather than being asserted.

Alternative considered — **keep `module_path!()` and cross-check it against `T`'s own path inside the
generic constructor, reacting on disagreement.** This also closes forgery, and its error message is
better ("you registered `Repo` in `app::startup` but it lives in `app::infra`"). Rejected: the check's
only purpose is to catch a disagreement that argument-free derivation makes unrepresentable, which the
minimalism bound ("fail loud only on observable misconfiguration; no defensive over-foolproofing")
excludes. It also adds a failure mode whose trigger is `type_name`'s unspecified rendering — turning a
format change into an **availability** problem (every adopter's startup fails) rather than a loud,
recoverable reaction. The diagnostic value is already covered: the existing `RegisteredCrossing`
finding names both the observed origin and the type name, so an adopter sees exactly which value to
put in `only_origins`.

### 2. An origin is the module the type is **defined** in

This follows from decision 1 — with no caller-supplied label there is nothing else it could be — and it
is the notion the documentation already describes ("which **concrete-type** origins may cross",
"a forbidden-origin type slipping through a `dyn Trait`"). "Where registered" was only ever a proxy
for it, and a weaker one, since it was the caller's choice.

Measured: for the documented idiom the value does not change at all.

```
                                    module_path!()                     type_name
nested inline mod, at defn site  composed_app::adapters::blessed  …::blessed::BlessedAdapter
                                 └──────────── identical ────────┘
same type, registered elsewhere  composed_app::startup            …::blessed::BlessedAdapter
                                 └──── differs: today's value is the misleading one ────┘
```

`examples/composed`'s `only_origins(["composed_app::adapters::blessed"])` therefore needs no edit, and
`COOKBOOK.md` already teaches the idiom (`/* registered inside its own module */`).

This also resolves a standing tension with the drift law ("no drift type without an observation
source"): the origin drift type's observation source was, until now, a *declaration made at the call
site*. It becomes an observation of the type.

### 3. Stated shape bounds instead of new fail-loud classes

`type_name` is measured, not assumed. Eight shapes:

```
nested inline mod   composed_app::adapters::blessed::BlessedAdapter   → module = …::blessed        ✓
crate-root type     composed_app::AtRoot                              → module = composed_app     ✓
alias to a type     composed_app::adapters::blessed::BlessedAdapter   → reports the DEFINING path ✓
                                                                        (an alias cannot lie)
generic type        composed_app::Generic<u8>                         → strip `<…>` first, then
                                                                        take the last `::`
foreign type        std::collections::hash::map::HashMap<u8, u8>      → a private INTERNAL path,
                                                                        not `std::collections`
boxed dyn           alloc::boxed::Box<dyn core::fmt::Debug>           → `alloc`, not `std`
fn-local type       composed_app::main::local::FnLocal                → not a module path at all
```

The last three are **stated bounds**, not new errors. Rejected alternative — refuse to register a type
whose derived path is not a plausible module path (a new exit-2/panic class): it would need louke to
know which crate is "yours", which is again caller-supplied information, and the existing fail-closed
match already handles every one of these loudly and correctly — a foreign type's origin simply is not
on your allowlist, and the finding names it. Adding a gate buys nothing the reaction does not already
give, and costs a failure mode.

Registering a foreign type to attribute it to your own layer stops working, and that is the point: it
*is* the self-asserted label being removed. The Rust-idiomatic replacement — a newtype defined in your
own layer — is also the more honest architecture, since the thing crossing the seam is your adapter.

Generic-argument stripping must scan for the **first top-level** `<` and honor nesting, because
argument text can itself contain `::` (`Repo<std::string::String>`).

### 4. Retire the residual's footprint in the same change

The two tests added this window are written to fail when the gap closes, and they will:

- `a_hand_built_origin_entry_is_accepted_a_known_trust_bound` — **delete**; there is no residual left
  to pin.
- `the_origin_guarantee_is_never_summarized_as_absolute` — **invert**. It currently requires the
  process-trust-boundary prose to be *present* in `crates/louke/README.md`, `crates/louke/src/dsl.rs`,
  `openspec/specs/runtime-origin-assertion/spec.md`, and `PROJECT.md`. After this change that prose
  must be *absent* from all of them, and what must be present is the derived-origin statement.

Five prose sites carry the claim and must move together (spec, `PROJECT.md`, `crates/louke/README.md`,
`register_origin!`'s doc, `CHANGELOG.md`), plus `BACKLOG.md`'s DESIGN-BREAKING entry, which closes. A
later fix inside one unreleased window invalidating earlier prose from the same window is a known trap
here — the 0.4.0 CHANGELOG text explaining the cooperative trust boundary becomes wrong the moment
this lands.

## Risks / Trade-offs

- **A registration written away from the type's module changes its origin value, silently at compile
  time.** → The change is loud at runtime in the safe direction: the allowlist stops matching and the
  boundary reacts fail-closed, with a finding naming the real origin and type. Recorded as a
  `**BREAKING**` CHANGELOG entry with the migration step (update `only_origins` to the type's actual
  module). No baseline is affected, because an observed origin never reaches one.
- **`type_name`'s rendering is documented as unstable across compiler versions.** → The blast radius is
  bounded to loud false positives (origins stop matching allowlists), never a false negative, and
  never a startup failure — which is exactly why the cross-check variant was rejected. The origin does
  not enter any `RuleKey` or baseline, so no recorded identity re-keys on a toolchain change.
- **A foreign type's origin becomes an implementation-detail path** (`std::collections::hash::map`). →
  Stated as a bound, with the newtype idiom as the answer; fail-closed means the wrong path reacts
  rather than passes.
- **A type defined inside a blessed module still gets a blessed origin.** → Unchanged by this design
  and out of scope: writing into that module is editing the law's subject, which no in-repo tool
  guards.
- **Losing the "attribute a foreign type to my layer" affordance.** → Deliberate. It is the
  self-assertion this change removes.

## Migration Plan

1. Land the derivation and the argument-free constructor together with the spec delta, so no surface
   states a bound that no longer exists at any commit.
2. Delete the residual-pinning test and invert the claim guard in the same commit as the code, so the
   guard never sits in a state where it enforces the wrong direction.
3. Sweep the five prose sites plus `BACKLOG.md` in the same change; the claim guard's inverted form is
   what keeps the sweep honest rather than a checklist.
4. Rollback is a revert: nothing persisted (no baseline, no wire format) changes, so reverting restores
   the previous behaviour exactly.

## Open Questions

- **Closed: the react→pass transition needs no migration diagnostic, and the reason is a proof rather
  than an assurance.** Adversarial review of this document upheld that decision 1's minimalism argument
  did not answer the *one-release diagnostic* case, so it was reopened and settled on the structure of
  the transition itself.

  Let `D` be the type's defining module, `R` the registration module, and `A` the seam's allowlist.
  Today a crossing passes iff `R ∈ A`; afterwards iff `D ∈ A`. So a reaction can only go quiet when
  `R ∉ A` **and** `D ∈ A` — and `D ∈ A` is the seam explicitly declaring that types defined in `D` may
  cross, while the crossing type *is* defined in `D`. **Every reaction that goes quiet is therefore one
  a seam raised against a source it declares allowed**: a false alarm under the definition this change
  adopts, not a violation. There is no configuration in which a type crosses a seam that forbids its
  defining module and the reaction disappears.

  Two attempts to break that proof, both failing:
  - *Multi-seam.* A type allowed at seam 1 (`R ∈ A₁`) and rejected at seam 2 (`R ∉ A₂`) can go quiet at
    seam 2 with no loud counterpart, if `A₁ ⊇ {R, D}`. Contrived but constructible — and seam 2 still
    satisfies `D ∈ A₂`, so its reaction was a false alarm too. The proof covers it.
  - *Population.* `R ∉ A` means that type reacts on **every** crossing today, so an adopter in this
    state is already living with continuous false violations, not silence. They do not experience a
    reaction disappearing; they experience spurious ones stopping. Registering a type purely to have it
    rejected is redundant, since fail-closed already rejects an *unregistered* type.

  The realistic breakage is the opposite direction — `R ∈ A`, `D ∉ A`: registration away from the
  module with the allowlist naming the registration site. That **starts** reacting, loudly, with a
  finding naming the real origin and type. Nothing needs to be added for it.

  The proof is relative to the new definition of an origin; a reader who rejects decision 2 rejects
  this too, which is why decision 2 states its own grounds separately.

- The other question raised during exploration is decided above: stated bounds over a new fail-loud
  class for foreign and function-local types (decision 3), recorded with the alternative and the reason
  it lost.
