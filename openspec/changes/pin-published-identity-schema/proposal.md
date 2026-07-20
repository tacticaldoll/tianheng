## Why

Version 2 baselines made each dimension's finding namespace, fact code, field names, canonical
values, and outer target/rule roles part of the published compatibility wire, but the current tests
mostly prove local distinctness or whole-document rendering. A refactor can therefore re-key an
adopter's baseline while all existing tests remain green.

## What Changes

- Add an explicit compatibility catalog that exercises every shipped 圭表, 渾儀, and 漏刻 fact
  family and finite identity-bearing nested discriminator through its dimension-owned conversion
  and pins its published version-2 identity shape.
- Pin namespace, code, field roles, representative canonical values, and the target/rule roles that
  complete `ViolationId`; do not snapshot human finding text or an entire report document.
- Mark key-producing canonicalizers, including async signature-tail rendering, as wire-sensitive so
  readability maintenance cannot be mistaken for presentation-only polish.
- Keep production keys, baseline format, public APIs, and human presentation unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `structured-violation-identity`: require a cross-dimension compatibility reaction that pins every
  shipped fact family's published version-2 identity schema independently of presentation.

## Impact

The change affects test-only identity catalog coverage in `guibiao`, `hunyi`, and `louke`, plus
documentation on key-producing canonicalizers. It adds no dependency, changes no production
behavior, and does not alter the version-2 baseline wire or package versions.
