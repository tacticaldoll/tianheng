## Why

圭表 and 渾儀 observe **one** crate root per package — the first library-kind target, else the first
`bin` — so a violation written in any other compiled root of that package passes silently. That is the
false negative the Core Contract forbids, and it lands on the most ordinary Rust package shape: a
library beside its binary. The 0.4.0 window recorded it as a stated bound; this closes it.

Three facts found by measuring, each of which changes the shape of the work:

1. **漏刻 already does it.** The 天衡 shell passes `xingbiao::member_root_files` — every library and
   `bin` root — to the probe-coverage audit, and a probe living only in `main.rs` counts as coverage.
   So the enumeration half is already built, public, and in production use; two dimensions diverge from
   the third rather than the family sharing a bound. (The 0.4.0 record said the scope question was
   shared because 渾儀 uses the same single-root function as 圭表 — true of 渾儀, and 漏刻 was never
   checked. That is corrected here.)
2. **The corpus model tolerates N roots.** A spike composed the existing per-root machinery once per
   root and merged the results: 316 of 圭表's 317 tests passed, and the one failure was the spike
   dropping a documented fallback for synthetic metadata that omits `targets`, not an identity
   collision. No distinctness test failed.
3. **A target's name is not unique within a package.** This repository proves it: `tianheng` has a
   `lib` target named `tianheng` and a `bin` target named `tianheng`. So the identity role cannot be the
   target name.

## What Changes

- **BREAKING**: the governed corpus of a package becomes **every** compiled root — each library-kind
  and `bin` target — and the modules reachable from each. A violation in `main.rs`, `src/bin/*.rs`, or a
  custom `[[bin]] path` now reacts.
- **BREAKING**: a module fact and a semantic fact gain a **compilation-unit** identity role: the root's
  path relative to the package's manifest directory (`src/lib.rs`, `src/main.rs`, `tools/x.rs`). Without
  it, the same violation written in two roots of one package carries one identity, so a baseline
  accepting it in one root silently masks it appearing in the other — the baseline-masking false
  negative this window closed six times by owner-qualification, arriving through the corpus instead of
  the renderer.
- An unknown-module constitution error becomes "**no** root has this module" rather than "this root does
  not", since a module legitimately exists in one root's graph and not another's. Found by the spike;
  without it a boundary on a lib-only module would exit 2 for the bin root.
- The role's own bound is stated rather than left implicit: a root whose path does not lie under the
  manifest directory keeps the path as given, exactly as 漏刻's absolute-`#[path]` rule does — the same
  family rule, not a new one.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `structured-violation-identity`: the compilation unit joins the declaring crate as an identity-bearing
  observed value when it can vary.
- `module-boundary`: the single-governed-root requirement added earlier in this window is replaced by
  the per-target corpus it recorded as design work.

## Impact

- `crates/xingbiao/src/lib.rs` — a per-package `crate_root_files`, mirroring `crate_root_file`.
- `crates/guibiao/src/module_check.rs` — per-root composition; the unknown-module semantics.
- `crates/hunyi/src/file_scope.rs` — the same.
- `crates/guibiao/src/finding.rs`, `crates/hunyi/src/finding/*` — the identity role.
- Specs, `BACKLOG.md` (the entry closes), `CHANGELOG.md`.
- **Baseline**: every module and semantic entry re-keys. This is absorbed by the regeneration 0.4.0
  already requires — identity gained fields in five places this window, and an adopter regenerates once
  either way. After release the same addition would cost a second forced regeneration.
