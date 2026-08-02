## Context

`crates/hunyi/src/scan.rs::resolve_child_modules` is the crate-wide traversal `scan_crate` runs once
per capability invocation, collecting the `pub use` re-export closure, the resolvable type-alias map,
crate-root extern renames, locally-defined trait paths, every trait-impl site, and every type
definition (with its `#[derive]`s). It already correctly follows an **unconditional** `#[path = "…"]`
remap (both for an inline module's relocated children-base and a file module's relocated file) — that
fix predates this change. The bug is specifically in the OTHER branch: `has_path_attr` matches BOTH
the unconditional form and the `cfg_attr`-wrapped (conditional) form, and after the unconditional
branch above it falls through (`continue`s) without matching, the remaining `if has_path_attr(...) {
continue; }` treated ANY `cfg_attr`-wrapped `#[path]` — inline or file-based — as a bound to skip
entirely, dropping the module's own observation, not just the alternate target.

Reproduced directly, matching the audit's own two findings:

```rust
// Inline — no x.rs needed or read; #[path] has no effect on an inline mod at all.
#[cfg_attr(windows, path = "x.rs")]
pub mod inner {
    pub fn f() { unsafe {} }
}
```
```rust
// File — any() is always false, so imp.rs (present) is what every build actually compiles;
// never.rs (absent) is never needed.
#[cfg_attr(any(), path = "never.rs")]
pub mod imp;
// imp.rs: pub fn f() { unsafe {} }
```

Both silently vanished from `check_unsafe_confinement`'s reaction (exit 0 Clean) while 圭表
(`guibiao::check`) and 漏刻 (`audit_probe_coverage`) both reacted (exit 1) on the identical files.

Traced `scan_crate`'s consumers to find the true blast radius, since the bug is in a **crate-wide**
data structure, not a single capability's own logic: `exposure.rs` (signature-coupling),
`trait_impl.rs` (trait-impl-locality), `forbidden_marker.rs` (forbidden-marker), and
`crate_scope.rs::extern_resolution` (feeding `resolve_principal`, the shared principal-trait resolver
dyn-trait's and impl-trait's *operand-scoped* boundaries use) all call `scan_crate` directly.
Independently reproduced the identical gap for each: a `cfg_attr`-wrapped `#[path]`-hidden module's
own `pub use` re-export silently missing from `scan.reexports` let a forbidden type reached through it
escape signature-coupling's exposure query and dyn-trait's operand-scoped principal-trait match, and
let a misplaced impl escape trait-impl-locality, exactly as it escaped unsafe-confinement.

## Goals / Non-Goals

**Goals:**
- An inline module reached only via a `cfg_attr`-wrapped `#[path]` is always descended — `#[path]` has
  no effect on an inline module's own content, cfg-wrapped or not.
- A file module reached via a `cfg_attr`-wrapped `#[path]` has BOTH its conventional file and its
  `cfg_attr` target read, as separate unioned sources, whichever exist — cfg-blind observation cannot
  know which one a given build actually compiles, so neither is silently preferred over the other.
- Neither candidate existing, with no other cfg-conditional gate on the declaration, remains a genuine
  scan error (exit 2) — `cfg_attr` never removes the `mod` item, so something must back it on every
  configuration.
- Every direct consumer of `scan_crate` benefits automatically from this one fix location — no
  separate per-capability code change needed, since the bug lives in the shared crate-wide data
  structure, not in any one capability's own matching logic (unlike the previous
  `hunyi-cfg-branch-use-reexport-merging` change, where each consumer's own single-candidate
  resolution needed its own code fix). Verified for all five reachable consumers individually
  (unsafe-confinement, trait-impl-locality, forbidden-marker, signature-coupling, dyn-trait's
  operand-scoped boundary) rather than assumed by category.

**Non-Goals:**
- `crates/hunyi/src/module_resolve.rs`'s single-module-anchored descent (`descend`,
  `resolve_module_items_with_files`) — used by signature-coupling's own module-anchor resolution,
  visibility, async-exposure, and dyn/impl-trait's module-scoped (non-operand) variant. This walker
  ALREADY, deliberately, fails loud (exit 2 "cannot judge") on a `cfg_attr`-wrapped `#[path]` rather
  than silently passing — a narrower, already-correct, accepted bound (documented in its own
  `direct_path_value`/`has_path_attr` call sites), not the false-negative class this change closes.
  Widening it to also union candidates is a separate, smaller follow-up if ever motivated by a
  reproduced gap — not attempted here, since a fail-loud bound is not itself a bug.

## Decisions

- **Union, not "pick one."** Mirrors 圭表's own already-fixed policy for the identical shape
  (`module-boundary` spec: "the scanner SHALL collect all candidate remapped targets and perform a
  union-scan across all candidate target files that physically exist on disk"). 渾儀's crate-wide walk
  now agrees with 圭表 on this shape, closing a cross-dimension divergence the same way
  `hunyi-cfg-branch-use-reexport-merging` closed the `use`/re-export map one.
- **A new `cfg_attr_path_value`, not reusing `has_path_attr`.** `has_path_attr` stays a pure boolean
  predicate (still used correctly by `module_resolve.rs`'s own, unchanged, fail-loud bound); extracting
  the VALUE needed its own function, mirroring `direct_path_value`'s existing NameValue-matching
  pattern rather than repurposing the boolean one.
- **`module_resolve.rs` is not touched.** Its own doc comments already state, correctly, that it fails
  loud on this shape rather than silently passing — that is not a bug this change's scope covers
  (a fail-loud bound is an accepted, narrower policy, not a false negative). Widening it is a distinct,
  separately-motivated question.
- **No new per-capability code.** Once `scan_crate`'s own walk is fixed, every capability that reads
  its output (`scan.reexports`, `scan.aliases`, `scan.impls`, `scan.type_defs`, `scan.trait_defs`)
  benefits without further code changes — verified per-consumer with a live reproduction for each,
  not assumed transitively.

## Risks / Trade-offs

- **[Risk] A crate-wide consumer NOT yet identified still misses the fix's benefit.** →
  **Mitigation**: enumerated every direct caller of `scan_crate` (`grep -rln "scan_crate("`) and
  independently reproduced the gap-then-fix for each of the five reachable capabilities
  (unsafe-confinement, trait-impl-locality, forbidden-marker, signature-coupling, dyn-trait's
  operand-scoped boundary via `crate_scope.rs`) rather than trusting the shared-mechanism argument
  alone.
- **[Risk] Unioning a `cfg_attr` target with the conventional file could double-count or
  false-positive when both happen to exist and are the SAME real file (e.g. a symlink).** →
  **Mitigation**: both sources are inserted through the existing `seen_files: HashSet<(String,
  PathBuf)>` dedup keyed on the module NAME and the resolved file's CANONICAL path — already in place
  for the analogous two-mutually-exclusive-plain-declarations case; unchanged by this fix.
- **[Risk] Widening the "conventional file absent" tolerance to also cover the
  cfg_attr-target-present case could mask a genuinely broken module (neither file exists, but some
  OTHER cfg gate makes the absence look legitimate).** → **Mitigation**: the new tolerance
  (`has_backing_source`) is additive, not a broadening of the EXISTING `cfg_conditional` tolerance —
  a module with neither a conventional file, nor an existing `cfg_attr` target, nor any other
  cfg-conditional gate still fails loud (verified with its own regression test).

## Migration Plan

1. Add `cfg_attr_path_value` (`syn_util.rs`), mirroring `direct_path_value`'s pattern for the
   conditional form (including nested `cfg_attr`).
2. Replace `resolve_child_modules`'s blanket `has_path_attr` skip with: inline content always
   descended; file content's conventional-file-and-cfg_attr-target union, each independently
   deduped/error-checked exactly like the existing single-candidate paths they extend.
3. Regression tests: inline-body-dropped (the first audit finding), conventional-file-dropped under
   an always-false predicate (the second audit finding), cfg_attr-target-used-when-conventional-absent
   (the symmetric case), and neither-candidate-present-fails-loud (confirming the fix didn't turn a
   genuine scan error into a silent pass) — plus one reproduction each for signature-coupling,
   dyn-trait's operand-scoped boundary, and trait-impl-locality's own `cfg_attr`-remapped-module test
   (which asserted the OLD, now-incorrect, "out of scope" behavior and needed updating).
4. Non-vacuous verification: reverted `scan.rs` to the pre-fix `has_path_attr` skip, confirmed every
   new/updated regression test fails in the predicted way, restored.
5. CHANGELOG `[Unreleased]` entry. No **BREAKING** marker — false negatives closing, not an identity
   shape; no existing baseline is invalidated. No version bump (campaign-wide constraint).

## Round 2 (adversarial review of round 1)

An independent adversarial review re-examined round 1's "all five consumers, `module_resolve.rs`
correctly out of scope" narrative rather than accepting it, and found it incomplete (not wrong about
the fix itself, which held up against every constructed counter-example):

- **A sixth and seventh consumer, undercounted.** `async_exposure_subtree_findings`
  (`async_exposure.rs`) and `impl_trait_subtree_findings`/`impl_trait_operand_subtree_findings`
  (`impl_trait.rs`) — the subtree-scope opt-in (`including_submodules()`) both capabilities support —
  call `walk_subtree_modules` (`scan.rs`), which calls `collect_subtree`, which calls the EXACT SAME
  `resolve_child_modules` this change patched. Round 1's commit message attributed these capabilities'
  correctness to `module_resolve.rs`'s "already correct, fails loud" behavior — factually wrong for
  this pathway; they never touch `module_resolve.rs`. Live-reproduced: both were broken pre-fix
  (`Ok([])` on a cfg_attr(path)-hidden submodule's own async fn / returned `impl Trait`) and correctly
  fixed post-fix, purely as a side effect of sharing one function — never independently reproduced,
  tested, or named as a consumer until this review.
- **Two stale doc comments**, left describing the pre-fix "skip" behavior after the code changed:
  `scan.rs`'s `walk_subtree_modules` doc (claimed `cfg_attr`-wrapped `#[path]` "is the actual stated
  coverage bound (skipped...)" — false now) and `syn_util.rs`'s `has_path_attr`/`direct_path_value`
  docs (claimed "the whole-crate **walks** do not follow" it, plural — only true of
  `module_resolve.rs`'s descent now, `has_path_attr`'s only remaining caller).
- **No functional bug** in six additional constructed counter-examples: union of two different
  existing files (both read, neither shadows the other); conventional-name/`cfg_attr`-target
  resolving to the identical canonical file via a relative-path trick (deduped correctly via the
  existing `seen_files` guard); deeply nested `cfg_attr(cfg_attr(path))` (correctly extracted);
  an inline module's OWN nested file-children under a cfg_attr-wrapped `#[path]` (resolve from the
  correct directory); `has_backing_source` combined with a co-occurring bare `#[cfg]` (correctly
  tolerated, matching the intended OR-gate).

Fixed: the two doc comments; added explicit regression tests for both newly-identified consumers
(`async_subtree_reacts_through_a_cfg_attr_wrapped_path_submodule`,
`impl_trait_subtree_reacts_through_a_cfg_attr_wrapped_path_submodule`), each independently
non-vacuously verified (reverted to pre-fix `scan.rs`/`syn_util.rs`, confirmed failure, restored).
Added `MODIFIED Requirements` deltas to `semantic-async-exposure-boundary` and
`semantic-impl-trait-boundary`'s own "Subtree scope opt-in" requirements, which had independently
stated the same now-incorrect "`cfg_attr`-wrapped `#[path]` SHALL remain unfollowed" claim.

## Round 3 (adversarial review of round 2)

A third independent review re-examined round 2's own restated claim (inherited from round 1) that
`module_resolve.rs`'s single-module-anchored descent was "already correct, fails loud" on a
`cfg_attr`-wrapped `#[path]` — and disproved it with a live fixture, from a different angle than the
prior two reviews used (they had tested only the LONE-declaration case, which genuinely does fail
loud; this review tested the SIBLING case):

```rust
pub mod infra;
#[cfg(windows)]
#[cfg_attr(target_arch = "x86", path = "foo_x86.rs")]
mod foo;
#[cfg(not(windows))]
mod foo;
```

`descend()`'s `if has_path_attr(&module_item.attrs) { continue; }` only skips the ONE declaration
carrying the attribute; the surrounding loop still processes the sibling `#[cfg(not(windows))] mod
foo;`, which resolves fine and populates `next_branches`. The `next_branches.is_empty()` check that
would otherwise raise `unknown_module_error` (exit 2) therefore never fires — the `cfg_attr`
branch's own file (`foo_x86.rs`, which under a windows+x86 build is what genuinely compiles) is
silently dropped with **exit 0**, not exit 2. Reproduced for both a bare-`#[cfg]` sibling pair and a
`cfg_if!` arm pair.

This is a **pre-existing** gap (present before this whole change; none of the two prior commits
touched `module_resolve.rs`), but this change's own commit messages twice asserted the opposite about
this specific function. Fixing it turned out to be MORE thorough than "close the sibling-absorption
gap" alone: testing revealed even a LONE `cfg_attr`-wrapped declaration with an EXISTING target file
(no sibling at all) previously never followed it — the "fail loud" behavior only fired for the
narrower case of NEITHER file existing. The fix therefore makes `descend()`'s handling of a
`cfg_attr`-wrapped `#[path]` module fully identical in spirit to `resolve_child_modules`'s: read the
conventional file AND the `cfg_attr` target when either exists on disk, unioned; fail loud only when
truly nothing backs the declaration on any configuration.

Fixing `descend()` closes the SAME false-negative class for every one of its callers:
signature-coupling's own anchor resolution (`exposure.rs`'s module-items path, distinct from its
`scan_crate`-backed alias/re-export closure already fixed), visibility (`visibility.rs`), dyn-trait's
shape-only module-scoped resolution (`shape_scan.rs`'s `resolve_module_items_with_files`), and
trait-impl-exposure (which shares signature-coupling's resolver). `has_path_attr` and its supporting
`is_path_remap`/`applied_metas_remap`/`meta_is_path_remap` are now fully dead (their only remaining
caller adopted `cfg_attr_path_value`) and are deleted.

## Open Questions

None outstanding.
