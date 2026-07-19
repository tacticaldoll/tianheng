## Context

`xuanji::Finding` now separates display text from a namespaced `FindingKey`. 圭表 and 漏刻 feed it
typed fact variants and fact-specific named fields. 渾儀 instead returns rendered `String`s from its
collectors, wraps each in `SemanticFact { kind, descriptor }`, and uses the whole display string as
the sole key field. The abstraction can demonstrate presentation independence in an artificial unit
test, but the live observation path cannot change its wording without changing baseline identity.

The earlier `PublicSeam` / `ExposureSubject` work was deliberately deferred until a structured
baseline made it necessary. That forcing event now exists. The implementation must preserve all
current human finding strings and must not widen 渾儀's public API.

## Goals / Non-Goals

**Goals:**

- Derive every semantic finding's display text and structured key from one typed fact value.
- Encode composite observations with named identity fields rather than one rendered descriptor.
- Preserve injectivity across every currently distinguished seam and semantic fact shape.
- Keep report text, public builders, checks, and the version-1 baseline migration unchanged.

**Non-Goals:**

- Publishing `PublicSeam`, `ExposureSubject`, or the internal fact catalog as adopter API.
- Changing what source syntax 渾儀 observes or adding a semantic rule.
- Generalizing the dimension-owned schemas into `xuanji`.
- Retaining compatibility with unreleased descriptor-based version-2 semantic keys.

## Decisions

### One semantic fact catalog owns key and text

Replace `SemanticFactKind` plus `SemanticFinding` with one private `SemanticFact` enum. Each variant
contains the observed values needed to render the existing text and constructs a fact-specific code
and named key fields from those same values. Collectors return these facts rather than strings; the
emit layer accepts facts and calls `into_finding()` exactly once.

Keeping a kind enum plus an opaque string was rejected because it preserves the current failure:
the compiler cannot ensure that identity-bearing values remain available after rendering.

### Public seams are typed at collection time

Introduce a private `PublicSeam` enum whose variants represent the current seam vocabulary: free,
inherent, and trait functions; named items; inherent and trait associated items; fields/variants;
re-exports; and trait-impl positions. A seam supplies both its existing display spelling and stable
named key fields. `PathExposure` and `ShapeExposure` carry `PublicSeam` (temporarily optional while a
syntax visitor has not yet been stamped) instead of `String`.

A string newtype was rejected: it prevents accidental argument swaps but still binds identity to
presentation. A generic bag of seam fields was rejected because it would duplicate
`FindingKey`'s weakly typed envelope inside the dimension instead of making invalid seam shapes
unrepresentable.

### Subjects stay canonical observed values, but gain role names

Exposure facts distinguish path, dyn-trait shape, and impl-trait shape in their fact code and carry
the canonical observed value under the named `subject` field. A separate public or recursive subject
AST is unnecessary: the canonical path/shape string is itself the scanner's observation, while its
role and containing seam—not its typography—were previously lost in `descriptor`.

### Module attribution travels beside, not inside, identity

Whole-crate scans continue pairing a fact with its enclosing module for source-file resolution.
That module enters the key only for fact variants where it already distinguishes the observed fact;
the attribution tuple alone remains metadata. Single-module file resolution tests emptiness over
facts rather than rendered strings.

## Risks / Trade-offs

- [A seam variant omits an identity-bearing component] → enumerate all current seam constructors,
  retain byte-for-byte text tests, and add pairwise key-distinction tests for same-subject seams.
- [A collector emits an unstamped exposure] → represent the pre-stamp state as `Option<PublicSeam>`
  and fail loudly at fact conversion; do not silently substitute an empty seam.
- [Sorting or deduplication changes] → derive ordering on typed facts and assert existing fixture
  reports remain byte-identical.
- [Version-2 baselines made from an intermediate 0.2 branch stop matching] → accept this within the
  unreleased breaking window; preserve published version-1 text migration and document the key-shape
  transition in the synced spec.
