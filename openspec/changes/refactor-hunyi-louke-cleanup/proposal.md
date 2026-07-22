## Why

`hunyi` (渾儀) provides AST semantic observation and `louke` (漏刻) provides runtime observation for Tianheng. Following 0.2.2 refinements, this change performs doc comment noise reduction across `hunyi` and `louke`, cleans up internal helper logic, and ensures alignment with semantic/runtime boundary specifications without breaking public APIs or adding dependencies.

## What Changes

- **Refactor `hunyi/src/` & `louke/src/`**: Clean up internal AST visitor logic and runtime audit scanner helpers.
- **Code doc noise reduction**: Update Rustdoc comments to high-signal forward-looking descriptions of boundary invariants.
- **Spec alignment**: Verify alignment against `semantic-boundary` and `runtime-boundary` specifications.
- **Zero API impact**: Maintain 100% backward API compatibility and existing dependency boundaries.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. (Non-breaking internal refactoring and documentation noise reduction; spec requirements remain intact.)

## Impact

- `crates/hunyi/src/` & `crates/louke/src/`: Internal code cleanup and docstring noise reduction.
- Public API and wire format: Completely unchanged.
