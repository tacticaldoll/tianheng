## Why

An un-auditable-probe fact's identity (`crates/louke/src/finding.rs:107`) embeds its `file` field as
the raw absolute filesystem path the scanner happened to read the source from. Reproduced directly:
scanning the byte-identical file at two different absolute locations (the same relocation a
different clone path or CI runner produces) yields two DIFFERENT `unauditable-probe` identities,
differing only in the `file` field's absolute prefix (`fact.fields = {"file":
"/tmp/.../probefix1/src/lib.rs", ...}` vs `".../probefix2/src/lib.rs"`, everything else identical).
Since `ViolationId` (the baseline-matching key) is built directly from this `fact`, a baseline
recorded in a dev checkout matches nothing on CI: the accepted violation re-fires as new (exit 1)
and the recorded baseline entry is simultaneously reported stale — the exact false-negative-on-one-
side, false-positive-on-the-other contradiction a baseline exists to prevent.

## What Changes

- The `file` component of an un-auditable-probe's identity is now labeled relative to the **common
  ancestor** of every root passed to one `audit_probe_coverage` call, rather than the raw absolute
  path. In the real `tianheng` CLI caller, `source_inputs` is every workspace member's `cargo_metadata`
  `src_path` (always absolute) — all of which share the actual checkout root as their common
  ancestor, by construction, regardless of the process's working directory or how it was invoked. A
  new `common_ancestor` (in `crates/louke/src/audit/scan.rs`) computes this once per
  `audit_probe_coverage` call; a new `labeled` helper strips it from each observed file before the
  label becomes part of `Probe::Unauditable`'s `file` field (and, downstream, the fact identity and
  the presentational `Violation.file`/finding text).
- Falls back to the previous absolute form only when no shared ancestor exists at all (e.g. a lone,
  unrelated standalone path in a direct test) — never worse than before this fix, and no caller loses
  information it previously had.
- No public function signature changed (`audit_probe_coverage`/`audit_probe_coverage_with_markers`
  keep their existing `source_inputs: &[PathBuf]` parameter); the anchor is computed entirely
  internally.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `runtime-origin-assertion`: the "An un-auditable probe's identity distinguishes distinct offending
  expressions" requirement gains the checkout-independence rule for its `file` component, plus a new
  scenario proving it.

## Impact

- Affected code: `crates/louke/src/audit/scan.rs`, `crates/louke/src/audit.rs`, `crates/louke/src/finding.rs` (doc comment only).
- **BREAKING** for baseline compatibility: any existing `--write-baseline` output naming an
  `unauditable-probe` violation is now stale (the `file` field's value changes from an absolute path
  to a checkout-relative one) and must be regenerated; every previously accepted one reappears as new
  exactly once. No DSL, builder, or CLI surface change — only the identity `fact` payload's `file`
  value changes shape.
- No version bump (campaign-wide constraint) — lands under `CHANGELOG.md`'s `[Unreleased]` section.
