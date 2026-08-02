## Why

`crates/guibiao/src/cargo_metadata.rs::is_self_dependency` exempts a `cargo metadata`
dependency edge from every crate rule (forbid/restrict/source) whenever its `name` equals the
target package's own name — with no check on the edge's `source`. The exemption's own doc
comment states its intent is narrow: excuse only the genuine self-referential **path**
dependency idiom (`main = { path = "." }`, a doctest/dogfooding pattern), which `cargo metadata`
always emits with a **null** `source`. Because the function never reads `source`, it also
excuses a *different, external* package that merely happens to share the target's own name and
is declared via `git`/registry — a real wrapper/fork/self-comparison shape — so
`restrict_dependency_sources_to`, `restrict_dependencies_to([])`, and `forbid_dependency_on`
all silently return `exit=0 Clean` against it. This is the exact false negative
`docs/audit/0.3.1-adversarial-sweep.md`'s "圭表 manifest/deps" finding names, and it is a Core
Contract violation (PROJECT.md: "the one forbidden bug is a false negative").

The bug is reproduced directly against the real `guibiao::check` entry point (see `design.md`):
a probe crate `foo` declaring `foo = { git = "https://example.invalid/foo.git" }` reads
`exit=0 Clean` under all three named rule constructors, when every one of them should react.

## What Changes

- `is_self_dependency` additionally requires the edge's declared `source` to be `null` before
  treating it as the crate's own self-referential edge — narrowing the exemption to exactly the
  genuine path-dependency idiom its doc comment already claims, closing the false negative for
  a same-named externally-sourced (git/registry) dependency.
- No public API, rule shape, or CLI surface changes; this is a same-crate observation-source
  correctness fix consumed identically by every existing rule (`dependencies`,
  `dependencies_with_disallowed_source`, `declared_features`).
- A regression test reproduces the exact audit trigger (a same-named `git` dependency) through
  each of the three cited rule constructors and asserts they now react instead of reading Clean.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `crate-dependency-boundary`: the shared self-referential-dependency exemption scenario
  ("A crate's own self-referential dependency is never a violation under any crate rule")
  currently states the exemption holds "regardless of ... declared source kind" — that is the
  bug's spec-level manifestation. Narrow it to a **path**-sourced self-reference only, and add a
  new scenario stating a same-named but externally-sourced (git/registry) dependency is NOT
  exempted and IS governed like any other cross-crate dependency.

## Impact

- Code: `crates/guibiao/src/cargo_metadata.rs` (`is_self_dependency`), plus a new regression
  test (unit-level fixtures alongside the existing self-dependency tests in
  `crates/guibiao/src/tests.rs`, and a real-entry-point integration test under
  `crates/guibiao/tests/` mirroring the audit's exact trigger).
- Spec: `openspec/specs/crate-dependency-boundary/spec.md` (one scenario narrowed, one scenario
  added).
- No dependency, manifest, or CLI-surface change. Not a breaking change: the exemption still
  holds for every existing legitimate self-path-dependency case; it only stops over-exempting a
  case it was never meant to cover.
