## Why

Baseline `owner` and `tracker` annotations are governance records, but a present JSON value of the
wrong type is currently normalized to absence. A typo such as an array or object can therefore
silently erase metadata on the next `--write-baseline`, violating the existing loud-failure contract
for malformed baselines.

## What Changes

- Define absent and explicit JSON `null` metadata as unannotated entries.
- Reject every present, non-null `owner` or `tracker` value that is not a string.
- Preserve the existing write-path behavior: an invalid prior baseline emits a warning before a
  fresh snapshot is written, rather than silently claiming its metadata was preserved.
- Record the completed 0.2.x backlog item without changing baseline identity or serialization.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `violation-baseline`: make the accepted JSON types for optional governance metadata explicit and
  reject wrong-typed values.

## Impact

The parser and tests in `xuanji` change, with a runner-level regression test for the existing
warning-before-rewrite behavior. No public Rust API, dependency, manifest, package version, or
version-2 identity field changes. A hand-edited baseline containing wrong-typed metadata will now
fail to gate and will warn before `--write-baseline` replaces it.
