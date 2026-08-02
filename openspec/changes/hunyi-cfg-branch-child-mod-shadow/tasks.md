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

- [x] 3.1 `FlatItem` gains a private `ArmKey { invocation, arm }` (alongside the existing
      `in_transparent_arm` bool, set together so they cannot drift), and `flatten_transparent_macros`
      assigns a fresh per-invocation counter so two arms of the SAME `cfg_if!` are distinguishable
      from two arms of a DIFFERENT one.
- [x] 3.2 `transparent_macro_arms`/`parse_transparent_arms` return arms as SEPARATE `Vec<syn::Item>`s
      (not one flattened list), so arm index survives.
- [x] 3.3 `provably_mutually_exclusive(a, b)`: true iff two different arms of the same `cfg_if!`
      invocation, or two bare `#[cfg(...)]` attributes that are syntactic negations of one another
      (structural `Meta` equality, not source-text comparison).
- [x] 3.4 `child_module_decls`/`reexport_externs_for`: the item-level companions to
      `child_module_names`/`externs.difference(&child_mods)`, computing the shadow per specific
      `pub use` item against its own cfg-compatible sibling `mod` declarations only.

## 4. Fix — direct head (module_findings)

- [x] 4.1 New `resolve_module_items_with_cfg_tags` in `module_resolve.rs` (parallel to
      `resolve_module_items_with_files`, which stays untouched for its other six callers).
- [x] 4.2 `exposure.rs`'s `module_findings`: `FileScope` carries `mod_decls` instead of a
      precomputed `externs_reexport`; each re-export exposure computes its own externs set via
      `reexport_externs_for` against its own originating item's tag.
- [x] 4.3 Regression tests 1.1/1.2 pass.

## 5. Fix — crate-wide closure (collect_reexports)

- [x] 5.1 `resolve/mod.rs`'s `collect_reexports` takes `items: &[FlatItem]` and
      `child_mods: &[(String, FlatItem)]`, computing the per-item shadow the same way the direct
      head does.
- [x] 5.2 `scan.rs`'s `walk_module` feeds `collect_reexports` the `flat: Vec<FlatItem>` it already
      computes (previously only used for the child-resolution absence check) instead of the
      flattened plain-item list.
- [x] 5.3 Regression test 1.3 passes.

## 6. Verify no regression (round 1)

- [x] 6.1 `cargo test -p hunyi` — full suite green, including every pre-existing
      cfg-branch/child-module-shadow test from rounds 6-8.
- [x] 6.2 `cargo clippy -p hunyi --all-targets --all-features -- -D warnings` clean.
- [x] 6.3 `cargo fmt -p hunyi --check` clean.

## 7. Round 2 — extend to the crate-root rename-alias shadow

- [x] 7.1 Verify (don't just trust) an independent adversarial review's claim that `renames_bare`
      is still cfg-blind: reproduce `extern crate serde as wc;` + `#[cfg(unix)] mod wc;` +
      `#[cfg(not(unix))] pub use wc::Value;` under `must_not_expose("serde")` returning zero
      findings on the round-1 fix. Confirmed real.
- [x] 7.2 `syn_util.rs`: new `reexport_renames_for(renames, child_mods, use_flat)`, the rename-map
      analogue of `reexport_externs_for`, reusing the identical `provably_mutually_exclusive`
      predicate over `HashMap<String, String>` instead of `HashSet<String>`.
- [x] 7.3 `exposure.rs`: a re-export exposure's bare-head fallback and its post-closure
      `apply_bare_alias_rename` both switch to a per-item `Cow<HashMap<String, String>>` computed
      via `reexport_renames_for`; a type-position head keeps the old branch-wide `renames_bare`
      unchanged (explicit Non-Goal, matching `externs_type`'s scope). The dangling
      "see the module doc" comment (no module doc actually carried the claim) is rewritten in place.
- [x] 7.4 `resolve/mod.rs`'s `collect_reexports`: `renames_bare` becomes per-use-item via
      `reexport_renames_for`, dropping the old once-per-call `renames_shadowed` computation.
- [x] 7.5 Regression tests added for both the direct-head and closure/facade forms of the
      rename-alias trigger; both fail on the round-1-only code and pass after 7.2-7.4.
- [x] 7.6 `cargo test -p hunyi` — full suite green again (449 passed).

## 8. Sync

- [ ] 8.1 Fold the MODIFIED deltas into `openspec/specs/semantic-reexport-exposure/spec.md` and
      `openspec/specs/semantic-signature-coupling/spec.md`, stating each shadow half (extern-name
      set, rename-alias map) in its own explicit sentence rather than a shared antecedent, so a
      future partial fix cannot leave the same ambiguity the round-2 review found.
- [ ] 8.2 Prune this change's dated archive copy immediately after sync, keeping only
      `openspec/changes/archive/.gitkeep`.

## 9. Definition of Done (workspace-wide)

- [ ] 9.1 Run the full Definition of Done command list from `AGENTS.md` before reporting the change
      done.
