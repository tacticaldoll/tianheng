## Why

The Markdown constitution projection currently leaks a boundary's durable anchor into the
inline rule parameters, contradicting the existing contract that the anchor is a distinct,
Some-only element. The pre-release diff also carries three trailing blank EOF lines that fail
the repository's basic diff-hygiene gate.

## What Changes

- Classify `anchor` as Markdown structural metadata rather than a rule parameter.
- Render a declared anchor as its own Markdown block element, while leaving anchor-less output
  byte-identical.
- Strengthen projection tests around the separation between structural metadata and rule
  parameters.
- Remove the three trailing blank EOF lines reported by `git diff --check v0.2.3...HEAD`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `constitution-projection`: Clarify that a Markdown anchor is a distinct structural element and
  never an inline rule parameter.

## Impact

The change is limited to Tianheng's Markdown projection and its tests, the existing projection
specification, and mechanical EOF cleanup in `PROJECT.md` and two existing specs. It changes no
Rust public API, dependency, manifest, package version, or baseline identity.
