## 1. Reproduce and pin the trigger

- [ ] 1.1 Reproduce the trigger shape (two stacked `#[cfg_attr(pred, path=…)]` on one `pub mod imp;`, no plain file, no direct `#[path]`) through the real `guibiao::check` entry point and confirm the current hard-error outcome (`ConstitutionError` exit 2, "source file could not be located").
- [ ] 1.2 Confirm the two control shapes behave as expected today: (a) a single `cfg_attr(path)` with no plain file also hard-errors (not stacked-specific); (b) a direct `#[path]` plus a `cfg_attr` fallback already works (the `656dc111` "observe-both" shipment's actual scope).
- [ ] 1.3 Read `crates/hunyi/src/scan.rs`'s `has_backing_source` boundary and confirm its exact behavior on the sub-case where the `cfg_attr(path)` target itself is absent and no plain file exists either (must still be a scan error) — do not assume, verify by reading the guard (`!has_backing_source && !cfg_conditional`).

## 2. Implement the fix

- [ ] 2.1 In `crates/guibiao/src/module_scan/reachability/walk.rs`'s `collect_children`, resolve a declaration's `conditional_path_eqs` candidates before deciding its plain-file branch, and widen the pushed `PlainSource.is_cfg_conditional` to `declared.is_cfg_conditional || has_backing_conditional_target`.
- [ ] 2.2 Verify the dual-conventional-forms ambiguity branch (`flat.is_file() && nested.is_file()`) is untouched by construction (it runs before `is_cfg_conditional` is ever consulted).
- [ ] 2.3 Verify per-arm ancestor isolation is preserved: the new check reads only the current `declared`/`loaded` values already scoped to one source, never merging across other `PlainSource`/`ConditionalPathSource` entries for the same child name.

## 3. Regression tests

- [ ] 3.1 Add unit tests in `crates/guibiao/src/module_scan/reachability/tests.rs`: stacked resolved candidates tolerate absence; a single resolved candidate tolerates absence; an unresolved candidate with nothing else present still errors; both conventional forms present alongside a resolved candidate still errors.
- [ ] 3.2 Add an integration test (`crates/guibiao/tests/cfg_attr_path_only_module_absence.rs`) exercising the same shapes through the real `guibiao::check(&Constitution, &Path)` entry point, plus the direct-`#[path]`-plus-fallback control.
- [ ] 3.3 Run `cargo test -p guibiao` and confirm all new and pre-existing reachability/module-boundary tests pass.

## 4. Documentation and spec sync

- [ ] 4.1 Update `openspec/specs/module-boundary/spec.md` via the delta in this change (narrow the "cfg_attr never tolerates absence" prose; add/rename scenarios for the resolved/unresolved/dual-form-alongside-resolved cases) — grep the WHOLE file for other stale mentions of the old blanket claim, not only the touched requirement.
- [ ] 4.2 Update `PROJECT.md`'s Decisions entry on module-import observation, which still describes the pre-`656dc111` "a cfg_attr-wrapped path stays a cfg-conditional exclusion... fails loud" policy — stale relative to both the existing union-scan shipment and this fix.
- [ ] 4.3 Add a `CHANGELOG.md` `[Unreleased]` `### Fixed` entry describing the false-negative closure, in this repo's established voice (matching the neighboring 渾儀/漏刻 entries this fix now matches).
- [ ] 4.4 Sync this change's spec delta into `openspec/specs/module-boundary/spec.md` and archive/prune the change directory per `AGENTS.md`'s lifecycle (keep only `openspec/changes/archive/.gitkeep`).

## 5. Definition of Done

- [ ] 5.1 Run the full Definition of Done command list from `AGENTS.md` and confirm every command passes clean.
- [ ] 5.2 Commit the work through the lifecycle's own commit shape (propose / apply / sync), Conventional Commits, no AI attribution.
