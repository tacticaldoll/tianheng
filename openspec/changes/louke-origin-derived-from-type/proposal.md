## Why

漏刻's runtime origin is supposed to be **observed**, but it is not: `register_origin!` passes three
pre-computed values — a `TypeId`, `module_path!()`, and `type_name::<T>()` — and any code in the
process can hand-write the same call with all three fabricated, registering a rogue type under an
allowlisted origin so a real seam crossing passes silently. That is a false negative, the one bug the
Core Contract forbids, and it is reachable today (reproduced against the real `install`/
`assert_boundary!` API).

The 0.4.0 window already made the bound honest on every documented surface. This change closes the
capability instead of describing it, and this is the window for it: the correction is breaking, and
deferring it means the claim stays qualified for another release line.

## What Changes

- **BREAKING**: an `OriginEntry`'s entire content becomes a function of a **type parameter**. The
  macro's expansion target takes the type and nothing else, so there is no caller-supplied value left
  to fabricate. Forgery stops being detected and becomes **unrepresentable**.
- **BREAKING**: a registered origin is the module the type is **defined** in, not the module the
  registration call sits in. For the documented idiom — `register_origin!` written inside the type's
  own module — the origin string is **byte-identical** to today's, including the family's own
  `examples/composed` dogfood; only a registration written away from the type's module changes value.
- **BREAKING**: the public `OriginEntry::__from_register_origin(TypeId, &str, &str)` is replaced. It
  is `#[doc(hidden)]` and documented as not-a-constructor, and the 0.4.0 migration already tells a
  hand-written caller to switch to the macro.
- `register_origin!(MyType)`'s **adopter-facing spelling does not change**, so no adopter edits source
  for this; an adopter who registered a type outside its own module may need to update
  `only_origins(...)` to the origin the type actually has.
- The residual this window pinned is **retired**: the trust-boundary paragraphs, the known-bound
  scenario, and the two tests that hold the gap in place all come out, in the same commit that closes
  it, so no surface keeps describing a limit that no longer exists.
- **Not** included, deliberately: prefix/subtree matching of the allowlist. Today an origin must
  equal an allowed entry; loosening that to a `::`-delimited prefix would let a type under a
  descendant module newly pass a boundary that reacts today — a false negative, and the opposite of
  this change's purpose.

## Capabilities

### New Capabilities

None. This closes a stated bound inside an existing capability rather than adding a dimension.

### Modified Capabilities

- `runtime-origin-assertion`: the origin-observation requirement changes what an origin **is** (the
  type's defining module, derived from the type) and **deletes** the process-trust-boundary bound it
  currently states, together with the scenario that pins a hand-built entry as accepted. Gains
  requirements for the derivation's own stated shape bounds (a foreign type reports its defining
  crate's internal path; a function-local type's path is not a module path; a generic type's
  arguments are not part of its origin).

## Impact

- `crates/louke/src/dsl.rs` — `OriginEntry`'s constructor becomes generic and argument-free.
- `crates/louke/src/lib.rs` — `register_origin!`'s expansion drops `module_path!()`, `TypeId::of`,
  and `type_name`; same invocation spelling.
- `crates/louke/src/registry.rs` — unchanged in shape: it still stores `&'static str` origins and the
  probe hot path is untouched (no new work, no lock, still std-only, still no `syn`).
- `crates/louke/src/tests.rs` — `a_hand_built_origin_entry_is_accepted_a_known_trust_bound` is
  deleted (its residual is gone) and `the_origin_guarantee_is_never_summarized_as_absolute` inverts:
  the process-trust-boundary prose must now be **absent** from the surfaces that were required to
  carry it.
- `PROJECT.md` — the Core Contract's "Non-bypassable, precisely" paragraph loses its 漏刻 exception.
- `crates/louke/README.md`, `README.md`, `COOKBOOK.md`, `CHANGELOG.md`, `BACKLOG.md` — the same claim
  is stated in five places and must move together; `BACKLOG.md`'s DESIGN-BREAKING entry closes. Note
  that the `register_origin!` samples in the root `README.md` and `COOKBOOK.md` are compiled by
  nothing: `ReadmeDoctests` covers `crates/tianheng/README.md`, which does not mention the macro. They
  are prose that can rot, and this change is what would rot them.
- **No baseline impact.** An observed origin never reaches a `Report` or a baseline: `check_crossing`
  is reached only from the prod probe path and its `Violation` goes to the sink, while every CI-face
  fact (`UnprobedSeam`, `UndeclaredProbe`, `DuplicateSeam`, `UnauditableProbe`) carries no origin and
  the runtime `RuleKey` is built from the **declared** allowlist, which this change does not touch.
- **No CI-face impact.** `audit_probe_coverage` audits seams and probes, never origins.
- Public API: one `#[doc(hidden)]` function's signature. Version class: **BREAKING**, already inside
  the 0.4.0 minor.
