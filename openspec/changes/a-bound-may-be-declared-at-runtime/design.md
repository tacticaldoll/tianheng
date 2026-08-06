## Context

`Observer::bounds` has no default body, so declaring bounds is a condition of implementing the trait. Every string
on the declaration path is `&'static str`, which admits only compile-time literals — and that constraint is
documented nowhere. The pre-release review of the `0.5.0` window found it by asking what a third party can build,
not by reading the types.

## Goals / Non-Goals

**Goals.** Make a runtime-declared bound expressible. Keep a literal declaration allocation-free and unchanged at
the call site. State what the model claims, either way.

**Non-Goals.** Changing what a bound *is*, or the bijection that holds every declared bound against its spec
scenario. Changing `Extent`'s value set. Making the ids dynamic *in the family's own* declarations — all 53 stay
literals, because the family's bounds are properties of its code.

## Decisions

### D1 — `Cow<'static, str>`, not `String`

`String` would allocate for all 53 family declarations, every one a literal, and would be paid on each
`observation_bounds()` call. `Cow<'static, str>` borrows a literal and owns a computed value, which is exactly the
distinction being modelled: the family's bounds are static, a third party's may not be.

### D2 — `impl Into<Cow<'static, str>>` on the constructors, `.into()` at the struct-variant fields

The constructors take `impl Into<…>`, so `BoundId::new("…")` and `BoundDecl::new(…, "shape", …, "pin")` are
unchanged at every existing site. The struct-variant fields (`Extent::…{ because }`, `Owner::Inherited{ from }`)
cannot: struct-literal syntax performs no conversion, so each of the 61 rationale sites gains `.into()`.

Constructor functions per variant (`Extent::out_of_reach("…")`) were considered and rejected: they would leave the
public variants constructible anyway, so the `.into()` would still be reachable — two spellings of one thing, and
the enum is matched on as much as it is constructed.

### D3 — rustc is the enumeration

The change's blast radius is 55 constructor calls and 61 fields. Rather than grep for them and risk a miss, the
field types change first and the compiler lists every site. That is the honest tool: a compile error per site is an
enumeration with no false negatives, which is more than any pattern over multi-line string continuations could
promise.

### D4 — Accessors return `&str`

`id()`, `shape()` and `pinned_by()` returned `&'static str`. A `Cow` can only honestly lend `&str` bound to the
declaration. Measured across the family: no caller holds one beyond its declaration — `observation_bound_model.rs`
copies into a `String` for its map keys, and the projection formats immediately.

### D5 — Not breaking, because it never shipped

`git show v0.4.0:crates/xuanji/src/lib.rs` mentions none of these types. The whole surface is new in the unreleased
window, so this owes no `**BREAKING**` mark and no migration note. Stating that from a measurement rather than from
memory is the point: the same refinement one release later would be a migration for every implementor, and the
window is the reason it is free now.

## Risks / Trade-offs

**A `Cow` in a public struct is a wider surface than a `&'static str`.** An adopter can now construct a declaration
whose id is computed, which means ids can collide at runtime where before they could only collide in source. The
bijection already refuses duplicate ids and says so; it now refuses them for a second reason, which is the same
refusal.

**The churn is real and mechanical.** 61 rationale sites gain `.into()` in files whose prose is carefully written.
The risk is a mangled string, not a wrong type — so the guard is that every declaration still renders into the
extents projection byte-identically apart from nothing, and the projection is diffed on every run.
