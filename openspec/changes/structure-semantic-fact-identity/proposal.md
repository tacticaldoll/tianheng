## Why

The shared structured-identity model is now live, but 渾儀 still converts each observation to its
complete display string and stores that string as one `descriptor` field. Its real pipeline
therefore re-identifies violations when presentation wording changes, contrary to the structured
identity contract and unlike 圭表 and 漏刻, which derive named key fields and text from typed facts.

## What Changes

- Replace 渾儀's kind-plus-descriptor envelope with a typed semantic fact catalog whose variants own
  both human rendering and named structured-key fields.
- Represent public exposure seams as structured data, including their item kind and owner/module/name
  components, rather than stamping a pre-rendered seam string into observations.
- Keep human finding text byte-identical while making wording independent from baseline identity.
- Cover every semantic finding family and seam shape with identity-injectivity and presentation-
  independence tests.
- **BREAKING**: change 渾儀's version-2 `finding_key` schemas from a single `descriptor` field to
  fact-specific named fields. Version-1 baseline migration remains supported; no adopter-written
  Constitution or boundary-builder API changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `structured-violation-identity`: require composite semantic observations to encode their
  identity-bearing values as fact-specific named fields, so a display-only wording change does not
  alter a live 渾儀 violation's key.
- `semantic-dyn-trait-boundary`: replace the obsolete rendered-descriptor identity wording with
  the named `subject` and structured-seam key contract while preserving its stated rendering bound.

## Impact

The change is confined to 渾儀's private finding/collector/resolver model, its tests and fixtures,
the structured-identity specification, and the decisions/backlog text that still describes seam and
subject typing as deferred. It changes emitted version-2 semantic keys but not report wording,
public builder/check entry points, dependencies, or package versions.
