## Why

`BACKLOG.md` has a long-deferred "forward depth" entry: `must_not_expose_existential` (unifier) —
folding impl-trait's written `impl Trait` (RPIT) and async-exposure's implicit `impl Future` under
one "no existential at this seam" declaration, deferred because "it must not blur the two findings'
identities." Today an adopter wanting full existential-leak coverage at a seam must hand-write two
separate boundaries. The blocker no longer holds: `crates/tianheng/src/sans_io.rs`'s `SansIoPure`
proves a shell-level **composed profile** can give adopters one declaration while keeping two fully
independent reactions — zero identity risk, because nothing about identity changes. This also
surfaces a real gap (impl-trait has no subtree-scope option, unlike async) that must close for the
composed profile to make an honest whole-subtree claim.

## What Changes

- New composed profile `NoExistentialLeak` / `Constitution::no_existential_leak(...)` in
  `crates/tianheng`, mirroring `SansIoPure`/`sans_io_pure` exactly: one declaration expands into an
  `ImplTraitBoundary::must_not_expose_impl_trait()` and an `AsyncExposureBoundary::must_not_expose_async_fn()`
  on the same module, each keeping its own separate `rule_key`/fact identity. Adds no new reaction.
- Both composed boundaries have subtree scope (`including_submodules()`) **unconditionally enabled
  by the profile itself** — never an adopter-facing toggle, matching how `sans_io_pure` already
  hardcodes it for its own async half.
- **New capability:** `ImplTraitBoundary` gains an opt-in subtree scope, `including_submodules()`,
  mirroring `AsyncExposureBoundary`'s existing one field-for-field (DSL option, `rule_key` field,
  reaction-side subtree walk, `list` projection). Required for the composed profile's whole-subtree
  guarantee to be honest — without it, the profile would silently under-cover the impl-trait half.
  Ships independently useful for hand-written `ImplTraitBoundary` declarations too.
- **Deliberately out of scope (stated, not silent):**
  - `DynTraitBoundary` does **not** gain subtree scope in this change — no demonstrated consumer
    needs it yet, even though it shares the same `ShapeExposure` collector as impl-trait.
  - `AsyncExposureBoundary` does **not** gain operand-scoping — its compiler-inserted `impl Future`
    is invariant, with no written principal-trait text for the existing operand-resolution
    mechanism to target. A deeper feature (scoping by the `Future::Output` type) is a different,
    unattempted capability.
- Two new tests, not new prose, encode the two facts above as reactions:
  - A `_composes_faithfully`-style test (mirroring `sans_io_pure`'s own `mod tests`) proving the
    profile expands to exactly a hand-composed pair with both halves' subtree scope on — fails
    loud if a future edit silently drops either half's `including_submodules()`.
  - An exhaustive `rule_key` schema test proving `AsyncExposureBoundary`'s fields are exactly
    `["including_submodules"]` — fails loud (forcing a conscious edit) if anyone later tries to
    add an operand field.

## Capabilities

### New Capabilities

(none — the composed profile follows `sans_io_pure`'s own precedent of not getting a dedicated
capability spec; see Impact)

### Modified Capabilities

- `semantic-impl-trait-boundary`: gains the subtree-scope opt-in requirement (new `### Requirement`
  and scenarios, mirroring `semantic-async-exposure-boundary`'s existing "Subtree scope opt-in"
  requirement).
- `governance-dogfood`: the public boundary-family inventory gains the `no_existential_leak`
  composed profile (alongside the existing `sans_io_pure` entry).
- `adopter-surface`: the exhaustive prelude export list gains `NoExistentialLeak`.

## Impact

- `crates/hunyi/src/dsl/impl_trait.rs`: new `including_submodules` field, builder method, and
  `rule_key()` field.
- `crates/hunyi/src/impl_trait.rs`: new subtree-walk branch, mirroring
  `crates/hunyi/src/async_exposure.rs`'s existing one (reusing `walk_subtree_modules` and the
  multi-module violation push path).
- `crates/tianheng/src/existential.rs` (new file): `NoExistentialLeak` DSL + `Constitution::no_existential_leak`,
  mirroring `sans_io.rs`'s structure, doc-comment style, and test suite shape exactly.
- `crates/tianheng/src/lib.rs`: register the new module, re-export via the prelude.
- `openspec/specs/semantic-impl-trait-boundary/spec.md`, `governance-dogfood/spec.md`,
  `adopter-surface/spec.md`: modified requirements per above.
- `BACKLOG.md`: resolve the `must_not_expose_existential (unifier)` entry — record it shipped as a
  composed profile (not a fused Rule), with dyn-trait's subtree-scope and async's operand-scoping
  named as continuing, deliberate non-goals.
- `CHANGELOG.md`: additive/patch. `including_submodules` on `ImplTraitBoundary` defaults off, so an
  existing boundary projects and reacts byte-identically; the new profile is a wholly new,
  opt-in declaration. No existing behavior changes.
