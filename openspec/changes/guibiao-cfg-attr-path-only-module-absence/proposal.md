## Why

圭表's module-boundary reachability walk hard-errors a file-form `mod` declaration whose only
backing is one or more `#[cfg_attr(pred, path = "…")]` remaps — even when a remap target
physically exists and the declaration compiles cleanly under real rustc on every configuration
(the standard per-platform shim: two stacked, jointly-exhaustive `cfg_attr(unix, …)` /
`cfg_attr(not(unix), …)` attributes, each naming a file present on disk, with no plain
`name.rs`/`name/mod.rs` and no direct `#[path]`). 渾儀 and 漏刻 already tolerate the identical
shape in their own crate-wide walkers (`has_backing_source`, `crates/hunyi/src/scan.rs`); 圭表's
own walk was never given the matching fix, so a boundary governing such a module returns a
constitution error (exit 2) instead of observing it — a false positive on code that builds, and
a three-dimension divergence from the family's own stated "三儀 ⊥ 三儀: the same rule, not the
same function" policy for this exact outcome.

## What Changes

- `resolve_plain_sources` / `collect_children` (`crates/guibiao/src/module_scan/reachability/walk.rs`)
  now tolerate an absent plain conventional file for a declaration when at least one of its
  `cfg_attr(path)` candidates resolves to a real, on-disk file — the same "might legitimately be
  absent on this build" signal a bare `#[cfg]` or a `cfg_if!` arm already carries, now extended to
  a resolved conditional remap target. This applies uniformly whether the declaration carries one
  `cfg_attr(path)` attribute or several stacked ones.
- The following outcomes are explicitly preserved (not widened):
  - Both conventional forms present (`name.rs` and `name/mod.rs`) is still an unconditional
    ambiguity constitution error, regardless of any `cfg_attr(path)` candidate.
  - When every candidate is absent — no plain file, no resolved `cfg_attr(path)` target, and no
    bare `#[cfg]`/`cfg_if!` arm — the declaration is still a genuine constitution error, matching
    渾儀's own `!has_backing_source && !cfg_conditional` boundary for the identical shape.
  - A direct, unconditional `#[path]` (with or without a `cfg_attr` fallback) is unaffected — it
    already bypasses the plain-file requirement entirely.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `module-boundary`: the "A plain module declaration resolves to exactly one conventional file"
  requirement's cfg-conditional test now also recognizes a resolved `cfg_attr(path)` remap
  candidate as a legitimate absence reason, alongside the existing bare-`#[cfg]` and `cfg_if!`
  arm sources. The requirement's current prose overstates the bound ("a `#[cfg_attr(...)]`
  wrapper SHALL NOT make a declaration cfg-conditional... in every configuration") — that
  statement is correct only when the `cfg_attr` wraps something other than a `path` remap, or
  when none of its `path` remap candidates resolves; the delta narrows it accordingly.

## Impact

- `crates/guibiao/src/module_scan/reachability/walk.rs` — `collect_children` now resolves a
  declaration's `cfg_attr(path)` candidates before deciding whether its plain-file branch is
  required.
- `crates/guibiao/src/module_scan/reachability/tests.rs` — new unit tests pinning the trigger
  shape, the single-target control, the still-must-error absent-candidate control, and the
  still-must-error dual-conventional-forms control.
- `crates/guibiao/tests/cfg_attr_path_only_module_absence.rs` — new integration test exercising
  the same shapes through the real `guibiao::check(&Constitution, &Path)` entry point.
- `openspec/specs/module-boundary/spec.md` — the affected requirement's prose and scenario list.
- No public API change; no breaking change. This is a false-negative closure (a previously
  hard-erroring, cleanly-compiling shape is now observed) — same compatibility class as the
  `656dc111` "observe-both" shipment and the 渾儀/漏刻 fixes this change now matches.
