## 1. `ImplTraitBoundary` subtree scope (crates/hunyi)

- [x] 1.1 Add `including_submodules: bool` field to `ImplTraitBoundary`
      (`crates/hunyi/src/dsl/impl_trait.rs`), defaulting `false`; add the builder method
      `including_submodules()` on the rule draft, mirroring `AsyncExposureBoundary`'s exactly.
- [x] 1.2 Add `including_submodules` to `ImplTraitBoundary::rule_key()`'s fields, mirroring
      `AsyncExposureBoundary::rule_key()`. **This changes `crates/hunyi/src/tests.rs:3384`'s
      `every_hunyi_rule_family_has_exact_semantic_identity`** — its existing `ImplTraitBoundary`
      block (`tests.rs:3415-3423`, currently asserting only `[("forbidden_operands", ...)]`) must be
      updated to include the new field, in the same commit that adds it (not a follow-up).
- [x] 1.3 Add the subtree-walk branch to the impl-trait reaction side (`crates/hunyi/src/impl_trait.rs`),
      modeled on `crates/hunyi/src/async_exposure.rs`'s existing
      `if boundary.including_submodules() { … } else { … }` branch. Reuse `walk_subtree_modules` and
      `push_multi_module_violations` as-is (both are genuinely collector-agnostic) — but this is
      **not** a byte-for-byte copy of `async_exposure_subtree_findings`: the per-item collector
      (`collect_item_return_impl_traits`) returns `()` into `&mut Vec<ShapeExposure>` and needs an
      additional `shape_finding(exposure, ExposureKind::ImplTrait)` render step that async's
      collector-and-push shape doesn't have. The new function **must** explicitly call
      `sort_attributed_facts(&mut findings)?` on its collected facts before returning, mirroring
      `async_exposure.rs:122` exactly — this is not automatic inside `push_multi_module_violations`,
      and it is the only thing standing between an unrenderable self type and a silently-published
      positional identity (see 1.3a).
- [x] 1.3a Thread the `ordinal` parameter passed into `collect_item_return_impl_traits` continuously
      across every module the subtree walk visits — never hardcode `0` for every call (async's own
      subtree function can do that only because its own collector ignores the parameter entirely;
      impl-trait's does not). Add the "unrenderable self type under subtree scope fails loud" test
      (task 3.5) to prove the failure mode is exercised, not just theoretically covered by 1.3's
      `sort_attributed_facts` call.
- [x] 1.4 Add the subtree-scope marker to the `list` text/JSON/markdown projection for
      `ImplTraitBoundary`, mirroring the async-exposure projection.
- [x] 1.5 Confirm the default (opt-in unset) path is byte-identical to today's behavior — no existing
      `ImplTraitBoundary` test's expected output changes other than the one named in 1.2.

## 2. Composed profile (crates/tianheng)

- [x] 2.1 Create `crates/tianheng/src/existential.rs`: `NoExistentialLeak` struct + builder chain
      (`in_crate(...).module(...).because(...)`, optional `.warn()`), mirroring `sans_io.rs`'s
      `SansIoPure` structure and doc-comment style exactly. No operand-scoping exposed.
- [x] 2.2 Add `impl Constitution { pub fn no_existential_leak(self, profile: NoExistentialLeak) -> Self }`
      expanding to `.impl_trait_boundary(ImplTraitBoundary::in_crate(...).module(...).must_not_expose_impl_trait().including_submodules()...)`
      and `.async_exposure_boundary(AsyncExposureBoundary::in_crate(...).module(...).must_not_expose_async_fn().including_submodules()...)`,
      both carrying the same `reason`/`severity`. Subtree scope is unconditional in the expansion —
      never an adopter-facing toggle on `NoExistentialLeak` itself.
- [x] 2.3 Register the new module in `crates/tianheng/src/lib.rs` and re-export `NoExistentialLeak`
      through the prelude, mirroring how `SansIoPure` is registered/exported.

## 3. Tests — encode the two protected facts as reactions, not prose

- [x] 3.1 Add a `mod tests` to `existential.rs` mirroring `sans_io.rs`'s exactly:
      `hand_composed(...)` (explicitly constructs both boundaries with `.including_submodules()` on
      each) and `via_profile(...)` (uses `Constitution::no_existential_leak`).
- [x] 3.2 `no_existential_leak_composes_faithfully`: asserts
      `constitution_markdown(&via_profile(...)) == constitution_markdown(&hand_composed(...))`.
- [x] 3.3 `no_existential_leak_threads_severity_to_both`: mirrors `sans_io_pure_threads_severity_to_both`
      (warn threads to both composed boundaries; warn projection differs from enforce).
- [x] 3.4 **No new test needed for async** — `crates/hunyi/src/tests.rs:3384`'s
      `every_hunyi_rule_family_has_exact_semantic_identity` already asserts
      `AsyncExposureBoundary::rule_key()`'s fields are exactly `[("including_submodules", ...)]`
      (`tests.rs:3460-3469`). Confirm this still passes unchanged (it should — async's shape doesn't
      change in this task). See task 1.2 for the test this change actually needs to touch.
- [x] 3.5 New `ImplTraitBoundary` subtree-scope tests, mirroring the async-exposure suite's shape:
      submodule leak reacts under the opt-in; anchor's own seam finding stays byte-identical;
      subtree bounded by the anchor, not the whole crate; cfg-gated fileless submodule tolerated vs.
      non-cfg missing file is a scan error; body-nested module is a stated bound (not observed);
      subtree opt-in projects in `list` output, default path byte-identical without it; **and the
      unrenderable-self-type-under-subtree-scope scenario** (two mutually-exclusive `#[cfg]` branches
      each with a same-named, unrenderable const-generic self type) fails loud with a constitution
      error, exit 2 — proving task 1.3's `sort_attributed_facts` wiring and task 1.3a's ordinal
      threading actually work, not just that they were written.
- [x] 3.6 `cargo test -p hunyi -p tianheng --all-features`.

## 4. Spec, dogfood, and documentation sync

- [x] 4.1 Sync `openspec/specs/semantic-impl-trait-boundary/spec.md`,
      `openspec/specs/governance-dogfood/spec.md`, and `openspec/specs/adopter-surface/spec.md`
      with this change's deltas (via the archive flow, not a hand copy).
- [x] 4.2 Add a `no_existential_leak` dogfood/example owner satisfying `governance-dogfood`'s
      updated family inventory (a self-governance test or an isolated example, matching how
      `sans_io_pure` is already owned).
- [x] 4.3 Resolve `BACKLOG.md`'s `must_not_expose_existential (unifier)` "Forward depths" entry:
      record it shipped as a composed profile (not a fused Rule), and name `DynTraitBoundary`
      subtree-scope and `AsyncExposureBoundary` operand-scoping as continuing, deliberate non-goals
      (not silently dropped).
- [x] 4.4 Add a `CHANGELOG.md` `[Unreleased]` entry: additive/patch — new `NoExistentialLeak` profile,
      new `ImplTraitBoundary::including_submodules()` opt-in (defaults off, byte-identical existing
      behavior). No breaking change.

## 5. Verification (Definition of Done, per AGENTS.md)

- [x] 5.1 `cargo build --workspace`
- [x] 5.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 5.3 `cargo clippy --workspace -- -D warnings`
- [x] 5.4 `cargo clippy -p louke -- -D warnings` (audit-OFF, prod-light build; unaffected by this
      change but required by the standing Definition of Done)
- [x] 5.5 `cargo fmt --all --check`
- [x] 5.6 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [x] 5.7 `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` (the new
      `NoExistentialLeak` rustdoc example must compile)
- [x] 5.8 `cargo deny check`
- [x] 5.9 `bash scripts/test_release_coherence.sh` / `bash scripts/check_release_coherence.sh`
- [x] 5.10 `bash scripts/test_examples.sh`
