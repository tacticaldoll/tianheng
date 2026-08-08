## Why

`observer-protocol` requires every semantic composition path to have one behaviour owner: the shell and
`SemanticObserver` both delegate to 渾儀's composed entry point rather than keeping independent empty-boundary
guards. The reaction that claimed to observe it read the characters of the shell's composition body, was
defeated at every level it could be narrowed to across four review rounds, and was retired one change earlier.
The gap it left is a declared, unpinned bound owned by this repository, and it is the register's visible debt.

Closing it by writing a fifth reader would repeat the mistake. The obligation is about what the shell *does*,
and execution is not a property of text.

## What Changes

- The shell's semantic arm invokes `SemanticObserver` instead of calling 渾儀's composed entry point beside it.
  The observer's `observe` **is** that entry point, so the two call sites become one call, and there is no
  second site in which a shell-local decision could sit. This is the route the runtime arm already takes.
- `observer-protocol` states the property as a **construction** in requirement prose rather than as a scenario,
  because a scenario asserting it could not fail — the rule this repository already applies to a property the
  data model constructs.
- The semantic dimension joins runtime in the construction-held list for the two paths' equality, and the
  requirement gains the sentence that keeps that honest: where a dimension's equality is construction-held, the
  reaction SHALL still observe that the fixture's boundary for it reacts at all.
- The unpinned bound is retired, in its scenario, its typed declaration, and both generated projections.

Not **BREAKING**. `SemanticObserver::observe` is `check_all` and always was, so no declaration's verdict moves.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observer-protocol`: two requirements change.
  - *An empty semantic observer SHALL not read workspace metadata* — the delegation is held by construction and
    its bound is retired.
  - *The built-in path SHALL keep its behaviour, and the two paths SHALL be held equal* — semantic moves into
    the construction-held list, and the reacts-at-all obligation is stated for every dimension in it.

## Impact

- `crates/tianheng/src/runner.rs` — the composition arm. The only product-code change.
- `crates/tianheng/src/bounds.rs` — the retired declaration. Unreleased public surface.
- `crates/tianheng/tests/observer_protocol.rs` — the module doc naming which dimensions are compared.
- `openspec/specs/observer-protocol/spec.md` — the two requirements, at sync.
- `docs/observation-bounds.md`, `docs/observation-bound-extents.md` — regenerated, never hand-edited.
- `CHANGELOG.md`, `BACKLOG.md` — the entry, and the READY-PATCH item this closes.

The static dimension is deliberately untouched: the built-in path calls `check_and_cover`, whose coverage
advisory the protocol cannot carry and whose second call would read `cargo metadata` twice, while the observer
calls `check`. Two implementations, so that dimension's equality stays measured rather than constructed.
