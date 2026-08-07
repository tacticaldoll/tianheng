## Why

The observation-bound register permits either one or more `PINNED-BY` citations or an `UNPINNED` tracker, but `BoundDecl` can represent only one mandatory pinning-test string. The typed model therefore cannot satisfy the accepted register contract and already discards one citation from a live multiply-pinned scenario.

## What Changes

- **BREAKING** Replace the undifferentiated `BoundDecl::new(..., pinned_by)` constructor with constructors for pinned, multiply-pinned, and tracked-unpinned declarations.
- Add a `Defence` tagged union whose pinned state always contains at least one test slot and whose unpinned state always contains a tracker.
- Compare the complete ordered defence state between specs and typed declarations and project every cited pin.
- Migrate the family declarations and adopter documentation to the explicit constructors.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-model`: Require typed declarations to express the register's mutually exclusive pinned and tracked-unpinned states and retain every pinning citation.

## Impact

The public `xuanji::BoundDecl` construction API changes, and every family declaration is migrated. The composed observation-bound model, its generated extent projection, the cookbook example, and adopter-facing Unreleased notes change with it. No manifest or package version changes in this development change.
