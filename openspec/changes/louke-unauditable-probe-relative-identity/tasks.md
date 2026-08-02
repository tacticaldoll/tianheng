## 1. Root-cause reproduction

- [x] 1.1 Reproduced directly: `audit_probe_coverage` on the byte-identical file at two different
      absolute temp locations yields two different `unauditable-probe` fact identities, differing
      only in the `file` field's absolute prefix — confirmed via a scratch test inspecting the raw
      `Outcome::Violations` debug output.

## 2. Checkout-independent labeling

- [x] 2.1 Added `common_ancestor(paths: &[PathBuf]) -> PathBuf` to `crates/louke/src/audit/scan.rs`:
      a file input's own directory (not the file itself) is the per-root candidate; the shared
      prefix across every candidate's path components is the anchor.
- [x] 2.2 Added `labeled(path, anchor) -> String`: `path.strip_prefix(anchor)`, falling back to the
      absolute form when stripping fails.
- [x] 2.3 Threaded `anchor: &Path` through `collect_probes_with_markers`, `collect_directory_probes`,
      `collect_reachable_probes`; both `.display().to_string()` call sites now use `labeled`.
- [x] 2.4 Computed the anchor once in `audit_probe_coverage_with_markers`
      (`crates/louke/src/audit.rs`), before the per-root scan loop, via `common_ancestor(source_inputs)`.
- [x] 2.5 No public function signature changed.

## 3. Regression

- [x] 3.1 `unauditable_probe_identity_is_stable_across_checkout_locations` — the byte-identical-file-
      at-two-locations repro, asserting `Violation::id()` is identical across both, the identity's
      `file` field is `"lib.rs"` (not absolute), and it never starts with `/`.
- [x] 3.2 `multi_root_probe_identity_is_relative_to_the_common_ancestor` — two workspace-member-
      shaped roots sharing one temp base, asserting each label is relative to that shared ancestor
      (`crate-a/src/lib.rs`, `crate-b/src/lib.rs`), distinguishing same-named files by their own
      member path.
- [x] 3.3 Non-vacuous verification: reverted the anchor computation to an empty `PathBuf` (mimicking
      the pre-fix unconditional-absolute behavior), confirmed both new tests fail exactly as
      predicted (raw absolute paths in the assertion failure output), restored. Full `louke --features
      audit` suite green after restore (119 tests, up from 117).

## 4. Documentation

- [x] 4.1 Updated `finding.rs`'s `UnauditableProbe` field doc comment to state the relative-labeling
      rule instead of leaving `file` undocumented as potentially absolute.
- [x] 4.2 Updated `audit.rs`'s public `audit_probe_coverage_with_markers` doc to state the same rule
      for the un-auditable-probe bullet.
- [x] 4.3 Added a CHANGELOG `[Unreleased]` entry with a **BREAKING** marker (baseline compatibility
      for `unauditable-probe` violations only, not a version bump).
- [x] 4.4 Added a `MODIFIED Requirements` delta to `runtime-origin-assertion`'s "An un-auditable
      probe's identity distinguishes distinct offending expressions" requirement, plus a proving
      scenario.

## 5. Definition of Done

- [x] 5.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [x] 5.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste
      call.

## 6. Adversarial review follow-up

- [x] 6.1 Review found a narrow, real gap: a file reached only through an ABSOLUTE
      `#[path = "/…"]` literal still falls back to the raw absolute label, since `Path::join`
      discards the receiver entirely when the joinee is itself absolute — the resolved path has no
      textual relationship to `anchor` at all (confirmed for both a target genuinely outside the
      anchor's tree and one coincidentally nested inside it — both fall back, since `strip_prefix`
      only succeeds by literal string-prefix match, never directory-tree nesting).
- [x] 6.2 Reviewed and accepted as a stated, documented bound rather than fixed further: an
      absolute-literal `#[path]` is already non-portable/machine-specific on its own. The realistic
      relative sibling-share idiom (`#[path = "../../shared/thing.rs"]`) was separately confirmed to
      already work correctly (identical label across two checkouts), since `join` never collapses
      `..` components.
- [x] 6.3 Added `an_absolute_path_literal_falls_back_to_the_absolute_label_a_stated_bound`, pinning
      that the violation still reacts (never silently dropped) with the absolute label.
- [x] 6.4 Updated `finding.rs`'s and `audit.rs`'s doc comments and the `runtime-origin-assertion`
      spec delta to scope the "never absolute" claim to the realistic relative case, with this bound
      stated explicitly rather than silently claimed as full coverage.
