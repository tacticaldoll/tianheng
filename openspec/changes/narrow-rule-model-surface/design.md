## Context

`Rule` is publicly readable through `CrateBoundary::rule()`, while `ModuleRule` is re-exported but
`ModuleBoundary` currently exposes no symmetric accessor. Enum-level `#[non_exhaustive]` permits new variants, but it does not protect fields of an
existing struct variant: downstream code can construct it and can match every current field. The
0.1.9 `.strict_external()` modifier therefore required a second payload-identical hidden variant to
avoid breaking the 0.1 line. pacta and modou use the builder/projection surface and do not directly
construct or match these variants.

The local rustc micro-spike establishes the intended language contract: a
`#[non_exhaustive]` struct variant remains externally inspectable through a pattern containing `..`,
while an external struct expression constructing it fails with E0639.

## Goals / Non-Goals

**Goals:**

- Make future fields on existing rule variants additive to open-ended downstream matches.
- Establish boundary builders as the sole public construction path without hiding rule inspection.
- Add the missing read-only module-rule accessor so narrowing construction does not leave a dead
  public model name.
- Remove the strict-external twin variant while preserving its reaction, projection, and identity.
- Keep the adopter-written `Constitution` / boundary DSL / `run` surface unchanged.

**Non-Goals:**

- Do not make `Rule` or `ModuleRule` opaque or remove their accessors.
- Do not change any boundary's observable scope, verdict, text, JSON, polarity, or finding identity.
- Do not redesign builder typestate, move projection machinery, or narrow guibiao's real downstream
  projection/baseline surface.
- Do not bump a package version in this change.

## Decisions

### Apply non-exhaustiveness at every data-carrying rule variant

Every struct variant of both public rule enums receives variant-level `#[non_exhaustive]`. This is a
uniform surface contract: an external consumer may inspect fields with `..` and must retain the
enum wildcard already required by enum-level non-exhaustiveness, but may not construct a rule or
assume that a known variant's field set is closed.

Annotating only `ConfineInlineSymbolPath` would solve the immediate twin but leave the same failure
mode on every other modifier-bearing rule. Making the enums opaque would be stronger than the
observed pain requires and would break read-side consumers unnecessarily.

### Builders own construction; no replacement public constructors

The existing boundary DSL already constructs every valid rule and enforces typestate such as the
required `.because(...)`. No public `Rule::new_*` constructors are added. Direct enum construction is
an accidental second DSL that can bypass builder sequencing; closing it reduces rather than moves
the public contract.

`ModuleBoundary::rule() -> &ModuleRule` is added symmetrically with `CrateBoundary::rule()`. It is
the smallest read-side path that makes the retained public enum observable; module package, target,
reason, and severity storage remain outside this change.

### Fold strict-external into the existing inline variant

`ConfineInlineSymbolPath` gains `strict_external: bool`; the hidden
`ConfineInlineSymbolPathExternal` variant is removed. `.strict_external()` flips the field on the
same variant. Internal label, polarity, text, JSON parameters, scan dispatch, and validation read the
field from one payload. The field is projection metadata and scan configuration, not violation
identity, preserving the existing no-rekey guarantee.

### Prove both sides of the public contract

Rustdoc examples provide an external-crate compile-pass example for open-ended matching and a
`compile_fail` example for direct construction. Existing reaction/projection/baseline tests pin
strict-external parity, and pacta/modou compile probes verify the real downstream surfaces.

## Risks / Trade-offs

- **Breaking direct constructors and closed-field matches** → This is the intentional 0.2 model-
  surface break. Document migration to the existing builder and `Variant { known, .. }` patterns.
- **A compile-fail example could fail for the wrong reason** → Import the public type explicitly and
  keep the example minimal so E0639 is the only unavailable operation.
- **Folding the twin could shift reaction identity or projections** → Reuse the existing shared
  inline helpers and retain the current identity-parity and strict-external projection tests.
- **Uniform annotations look broader than the immediate fix** → Every annotated variant has the
  same demonstrated field-growth hazard; no behavior or internal dependency changes accompany it.

## Migration Plan

1. Mark every struct variant in `Rule` and `ModuleRule` non-exhaustive, add
   `ModuleBoundary::rule()`, and add external-view docs.
2. Fold the strict-external twin into the existing inline variant and update internal matches.
3. Run rule, projection, identity, compile-doc, downstream, and full repository gates.
4. Roll back the change branch before release if unanticipated direct-variant consumers appear; no
   artifact or wire migration is required because behavior and serialization remain unchanged.

## Open Questions

None. The compiler spike and both reference consumers resolve the construction/matching choice.
