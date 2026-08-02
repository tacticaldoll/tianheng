## 1. Reproduce

- [x] 1.1 Reproduce the audited finding directly: a bare `#[cfg(unix)] mod serde;` /
      `#[cfg(not(unix))] pub use serde::Value;` pair under `must_not_expose("serde")` yields zero
      findings.
- [x] 1.2 Reproduce the `cfg_if!` form of the identical trigger.
- [x] 1.3 Reproduce the crate-wide-closure/facade sibling (not named in the audit, found while
      reading the whole file): the identical pair reached only through a facade in another module.

## 2. Adjudicate the dissent

- [x] 2.1 Grep `semantic-reexport-exposure/spec.md` in full for `cfg` — zero matches, confirming the
      dissent's cited SHALL (lines 190-206) was written for the unconditional coexistence case only.
- [x] 2.2 Read `change/hunyi-cfg-branch-use-reexport-merging` (PR #149, `b353264`) in full — its own
      commit body names this exact line as out of scope for a future change, not an accepted bound.
- [x] 2.3 Verdict: dissent rejected. Proceed to fix.

## 3. Fix — syn_util.rs primitives

- [ ] 3.1 `FlatItem` gains a private `ArmKey { invocation, arm }` (alongside the existing
      `in_transparent_arm` bool, set together so they cannot drift), and `flatten_transparent_macros`
      assigns a fresh per-invocation counter so two arms of the SAME `cfg_if!` are distinguishable
      from two arms of a DIFFERENT one.
- [ ] 3.2 `transparent_macro_arms`/`parse_transparent_arms` return arms as SEPARATE `Vec<syn::Item>`s
      (not one flattened list), so arm index survives.
- [ ] 3.3 `provably_mutually_exclusive(a, b)`: true iff two different arms of the same `cfg_if!`
      invocation, or two bare `#[cfg(...)]` attributes that are syntactic negations of one another
      (structural `Meta` equality, not source-text comparison).
- [ ] 3.4 `child_module_decls`/`reexport_externs_for`: the item-level companions to
      `child_module_names`/`externs.difference(&child_mods)`, computing the shadow per specific
      `pub use` item against its own cfg-compatible sibling `mod` declarations only.

## 4. Fix — direct head (module_findings)

- [ ] 4.1 New `resolve_module_items_with_cfg_tags` in `module_resolve.rs` (parallel to
      `resolve_module_items_with_files`, which stays untouched for its other six callers).
- [ ] 4.2 `exposure.rs`'s `module_findings`: `FileScope` carries `mod_decls` instead of a
      precomputed `externs_reexport`; each re-export exposure computes its own externs set via
      `reexport_externs_for` against its own originating item's tag.
- [ ] 4.3 Regression tests 1.1/1.2 pass.

## 5. Fix — crate-wide closure (collect_reexports)

- [ ] 5.1 `resolve/mod.rs`'s `collect_reexports` takes `items: &[FlatItem]` and
      `child_mods: &[(String, FlatItem)]`, computing the per-item shadow the same way the direct
      head does.
- [ ] 5.2 `scan.rs`'s `walk_module` feeds `collect_reexports` the `flat: Vec<FlatItem>` it already
      computes (previously only used for the child-resolution absence check) instead of the
      flattened plain-item list.
- [ ] 5.3 Regression test 1.3 passes.

## 6. Verify no regression

- [ ] 6.1 `cargo test -p hunyi` — full suite green, including every pre-existing
      cfg-branch/child-module-shadow test from rounds 6-8.
- [ ] 6.2 `cargo clippy -p hunyi --all-targets --all-features -- -D warnings` clean.
- [ ] 6.3 `cargo fmt -p hunyi --check` clean.

## 7. Sync

- [ ] 7.1 Fold the MODIFIED deltas into `openspec/specs/semantic-reexport-exposure/spec.md` and
      `openspec/specs/semantic-signature-coupling/spec.md`.
- [ ] 7.2 Prune this change's dated archive copy immediately after sync, keeping only
      `openspec/changes/archive/.gitkeep`.

## 8. Definition of Done (workspace-wide)

- [ ] 8.1 Run the full Definition of Done command list from `AGENTS.md` before reporting the change
      done.
