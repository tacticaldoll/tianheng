## Context

`crates/louke/src/finding.rs`'s `UnauditableProbe` fact bakes its `file` field straight into the
`StructuredFactIdentity` used for baseline matching (`ViolationId` — see `crates/xuanji/src/violation.rs`'s
`Violation::id()`, built from `target`, `rule_key`, and `fact`). That `file` value comes from
`crates/louke/src/audit/scan.rs`'s `collect_directory_probes`/`collect_reachable_probes`, which
previously called `path.display().to_string()` directly on whatever absolute path the caller passed
in (in the real `tianheng` CLI caller, `cargo_metadata`'s always-absolute `src_path`).

Reproduced directly (see the new regression tests in `crates/louke/src/audit/tests.rs`):

```
audit_probe_coverage(&boundaries, &[<checkout-1>/src/lib.rs]) → fact.file = "<checkout-1>/src/lib.rs"
audit_probe_coverage(&boundaries, &[<checkout-2>/src/lib.rs]) → fact.file = "<checkout-2>/src/lib.rs"
```
Byte-identical source, byte-identical everything else in the fact — only the absolute prefix
differs, so `ViolationId` differs, so a baseline recorded against one never matches the other.

## Goals / Non-Goals

**Goals:**
- An un-auditable-probe's `file` identity component is checkout-independent for the real caller
  shape (every workspace member's absolute root sharing one actual checkout root).
- No public API signature change.
- The fix degrades gracefully (falls back to the pre-existing absolute form) rather than fabricating
  a misleading relative path when no shared ancestor exists.

**Non-Goals:**
- No change to any OTHER fact's identity (圭表's `ModuleFact`, 渾儀's `SemanticFact` already carry no
  raw absolute path in their own identities — this is specific to 漏刻's runtime-audit `file` field).
- No baseline-migration tooling — an existing baseline naming an `unauditable-probe` violation simply
  goes stale and is regenerated once, exactly like every other identity-shape change in this
  project's history (see the crate-omitted-identity fix, `CHANGELOG.md`).

## Decisions

- **Common ancestor of `source_inputs`, not the process's current working directory.** A CWD-based
  heuristic (`std::env::current_dir()`) was considered and rejected: `tianheng check --manifest-path
  <path>` can be invoked from anywhere, so CWD is not reliably the workspace root. The common
  ancestor of the roots THEMSELVES makes no assumption about invocation location at all — it is a
  pure property of the input list, and for the real caller (every workspace member's own root) it
  provably equals the actual checkout root, since cargo emits every member's `src_path` under one
  shared workspace directory.
- **Computed once per `audit_probe_coverage` call, not per-root.** All roots passed to one call
  share one anchor — computing it once (in `audit_probe_coverage_with_markers`, before the
  per-root loop) is both correct (the anchor is a property of the whole input SET) and cheaper than
  recomputing per input.
- **A file input's own directory is the ancestor candidate, not the file itself.** Using the file
  path directly as a candidate would make `strip_prefix` on itself yield an empty label for a
  single-root call — using its parent directory instead means a single-root scan's own file labels
  relative to a sensible base (its own filename, or a reachable child module's relative path under
  it), never collapsing to nothing.
- **No API signature change.** The anchor is computed and threaded entirely through private
  (`pub(super)`) internals (`collect_probes_with_markers`, `collect_directory_probes`,
  `collect_reachable_probes`); `audit_probe_coverage`/`audit_probe_coverage_with_markers` keep their
  existing public signature. This keeps the fix additive at the API level even though it changes an
  identity VALUE (hence **BREAKING** for baseline compatibility specifically, not for any caller's
  compile-time contract).

## Adversarial review follow-up (round 1)

Independent review found a narrow, real gap: a file reached only through an ABSOLUTE
`#[path = "/…"]` literal WHOSE TARGET DOES NOT LIE UNDER THE ANCHOR falls back to the raw absolute
label. Root cause: `resolve_path_module` does `base.join(rel)`, and `Path::join`'s documented
semantics discard the receiver entirely when `rel` is itself absolute — so the resolved path has no
textual relationship to `anchor` unless it happens to share one. Reviewed and accepted as a stated
bound: an absolute-literal `#[path]` is already non-portable and machine-specific with or without
this identity concern (unlike the realistic relative sibling-share idiom —
`#[path = "../../shared/thing.rs"]` — which round 1 separately confirmed DOES produce an identical,
checkout-independent label across two checkouts, since `join` never collapses `..` components so
the anchor's own prefix text survives). Documented in `finding.rs`'s and `audit.rs`'s doc comments,
and pinned with a dedicated regression test asserting the violation still fires (never silently
dropped) with the absolute label.

## Adversarial review follow-up (round 2)

Round 1's own claim was itself incomplete, caught by round 2: it asserted "both repros fall back to
absolute" as if that were the whole story, but a target that happens to lie textually UNDER the
anchor does NOT fall back — `strip_prefix` succeeds by pure text match regardless of whether that
nesting is a real, portable directory relationship or a coincidence of one particular checkout's own
absolute path. Reproduced directly: the SAME hardcoded absolute `#[path]` literal, committed
verbatim into two different checkouts, produces a clean relative label in the checkout whose own
anchor happens to be a textual prefix of the literal, and falls back to the full absolute path (that
first checkout's own path, visible in the second checkout's output) in the other — the two
checkouts' identities still disagree, reproducing the exact checkout-dependent-identity problem this
whole two-round fix exists to close, for this one already-non-portable construct.

This residual inconsistency is NOT fixed in this change — doing so properly requires threading
"was this file reached via an absolute `#[path]` literal" as extra state through
`resolve_path_module`/`external_module_files`/`collect_scope_modules`/`collect_reachable_probes`'s
whole `(PathBuf, PathBuf)` pipeline (so `labeled()` can unconditionally skip relativizing such a
file, making its behavior deterministic rather than checkout-coincidental), which is a real,
separate-scoped refactor, not a mechanical follow-up. Recorded as a new, explicit finding in
`docs/audit/0.3.1-adversarial-sweep.md`'s 漏刻 identity section rather than silently left as an
inaccurate "stated bound," and pinned with a dedicated regression test
(`a_nested_absolute_path_literal_still_disagrees_across_checkouts_a_known_residual_gap`) proving the
two checkouts' identities differ, so a future fix has a failing case to work against and this test
itself fails loud if that fix changes the behavior without updating the assertion.

## Risks / Trade-offs

- **[Risk] Existing baselines naming an `unauditable-probe` violation go stale.** → **Mitigation**:
  documented as a **BREAKING** CHANGELOG entry with an explicit regeneration instruction, matching
  the project's own precedent for the crate-omitted-identity fix. No baseline for any OTHER fact
  shape is affected.
- **[Risk] A lone, disconnected standalone path (no real shared ancestor with anything) could
  produce a confusing label if `common_ancestor` guessed wrong.** → **Mitigation**: for a single
  input, the ancestor is exactly that input's own directory — `strip_prefix` always succeeds and
  produces a sensible, non-empty label; the only fallback-to-absolute path is a genuinely
  cross-filesystem-root/no-common-prefix case, which is no worse than today's unconditional absolute
  behavior.

## Migration Plan

1. Add `common_ancestor` and `labeled` to `crates/louke/src/audit/scan.rs`; thread `anchor: &Path`
   through `collect_probes_with_markers`, `collect_directory_probes`, `collect_reachable_probes`.
2. Compute the anchor once in `audit_probe_coverage_with_markers` (`crates/louke/src/audit.rs`)
   before the per-root scan loop.
3. Regression: a byte-identical-file-at-two-absolute-locations test asserting identical
   `Violation::id()` across both; a multi-root test asserting each member's label is relative to
   their shared ancestor, never absolute.
4. Non-vacuous verification: reverted the anchor computation (empty anchor, mimicking the old
   unconditional-absolute behavior), confirmed both new regression tests fail exactly as predicted,
   restored.
5. Updated doc comments (`finding.rs`'s `UnauditableProbe` field doc, `audit.rs`'s public
   `audit_probe_coverage_with_markers` doc) to state the relative-labeling rule.
6. CHANGELOG `[Unreleased]` entry with a **BREAKING** marker for baseline compatibility (not a
   version bump — campaign-wide constraint).
7. `runtime-origin-assertion` spec delta: extend the existing un-auditable-probe identity
   requirement with the checkout-independence rule and a proving scenario.

## Open Questions

None outstanding.
