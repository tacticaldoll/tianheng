## Why

`Rule` and `ModuleRule` are `#[non_exhaustive]` enums, but their existing struct variants remain
externally constructible and exhaustively matchable. Adding a field is therefore breaking and has
already forced `.strict_external()` to ship as a duplicate hidden variant; the 0.2 line is the
honest window to stop that model drift while preserving the adopter-written builder.

## What Changes

- **BREAKING**: make every data-carrying `Rule` and `ModuleRule` variant non-exhaustive, so external
  consumers can inspect known fields with `..` but construct rules only through the boundary DSL.
- Fold the hidden `ConfineInlineSymbolPathExternal` twin into `ConfineInlineSymbolPath` as a private-
  by-construction `strict_external` modifier field.
- Preserve the names and behavior of `Constitution`, boundary builders, `.strict_external()`,
  `run`, projections, reactions, and violation identity; retain `CrateBoundary::rule()` and add the
  symmetric read-only `ModuleBoundary::rule()` so both model enums remain genuinely inspectable.
- Add public-surface compile evidence and re-check pacta and modou against the local crates.
- Do not change any Cargo package version or add a dependency.

## Capabilities

### New Capabilities

- `rule-model-surface`: Defines the builder-owned construction and forward-compatible inspection
  contract for guibiao's public rule enums.

### Modified Capabilities

None. Existing boundary reactions and adopter-written DSL behavior remain unchanged.

## Impact

- `crates/guibiao/src/model.rs`: public enum variant annotations and inline-rule representation.
- Internal matches across guibiao and Tianheng projections: one inline variant instead of two.
- Downstream code that directly constructs a `Rule`/`ModuleRule` variant or matches one without
  `..` must migrate to the builder or an open-ended match; the stable builder path is unchanged and
  module rules become readable through a new symmetric accessor.
- No dependency, wire-format, baseline-identity, runner, or package-version change.
