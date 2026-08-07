## Context

`BoundDecl` currently stores one mandatory `pinned_by: Cow<'static, str>`. The spec register has two mutually exclusive defence forms (`PINNED-BY` and `UNPINNED`) and permits multiple `PINNED-BY` lines on a scenario. The composed model parser currently overwrites repeated pins, so both absence and multiplicity are lost before comparison.

## Goals / Non-Goals

**Goals:**

- Make pinned and tracked-unpinned declarations distinct typed states.
- Make a pinned state contain at least one test slot by construction and retain further pins in declaration order.
- Keep every string owned-or-borrowed so computed adopter declarations remain supported while family declarations remain measurable as literals.
- Preserve the existing rule that `Extent` alone derives what defence evidence must demonstrate.

**Non-Goals:**

- Validate Rust test registration or tracker paths in `xuanji`; the bound-register reactions already own those observations.
- Change extent classification, ownership, or false-negative semantics.
- Add a live unpinned family declaration merely to exercise the type.

## Decisions

1. Add a non-exhaustive `Defence` enum with `PinnedBy { first, additional }` and `Unpinned { tracker }`. Keeping `first` separate from `additional` makes a zero-citation pinned collection structurally unavailable through `BoundDecl` construction; a plain vector would reintroduce an invalid empty state.
2. Replace `BoundDecl::new` with `pinned`, `pinned_by_many`, and `unpinned`. Named constructors make migration explicit and prevent callers from fabricating an irrelevant placeholder pin for unpinned debt.
3. Put `pinned_by`, `pinning_tests`, and `tracker` accessors on `Defence`, while `BoundDecl::defence` exposes the tagged state. This keeps evidence-specific questions on the value that owns them.
4. Parse every `PINNED-BY` line into an ordered vector and reject a scenario that mixes pinned and unpinned forms. Compare the complete defence value between spec and code; id-only equality is insufficient.
5. Render every pin into the generated extent projection. A projection retaining only one citation would let its generator bless the same loss the model is intended to prevent.

## Risks / Trade-offs

- [Public constructor break] Existing adopters must rename construction calls and select an explicit state. → Document the mechanical single-pin migration in `[Unreleased]` and the cookbook.
- [Additional vector allocation] Multiply-pinned declarations carry a vector container. → Single-pin declarations keep an empty vector and all string payloads remain borrowed for family declarations; the existing allocation audit measures every string position.
- [Future enum variants] External exhaustive matching would freeze the model. → Keep `Defence` non-exhaustive and make in-family projections fail loud on an unknown variant.
- [Order sensitivity] Reordering citations becomes an observable mismatch. → Preserve spec declaration order deliberately because generated documents present the same order to readers.
