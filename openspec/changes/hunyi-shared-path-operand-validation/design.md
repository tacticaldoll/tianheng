## Context

`has_empty_path_segment` (`crates/hunyi/src/resolve/mod.rs`) already states, in its own doc, that
"a caller comparing a forbidden/allowed operand against a canonical path this crate resolves MUST
reject this before matching." Four call sites obey that for the **forbidden** polarity, each with
the byte-identical inline guard:

```rust
if let Some(bad) = forbidden.iter().find(|f| has_empty_path_segment(f)) {
    return Err(malformed_path_operand_error(bad));
}
```

Read individually rather than assumed identical:

- `exposure.rs::module_findings` — `forbidden: &[String]`, signature-coupling's `must_not_expose`.
- `forbidden_marker.rs::forbidden_marker_findings` — `forbidden: &[String]`, `must_not_acquire`.
- `shape_scan.rs::operand_module_findings` — `forbidden: &[String]`, the dyn/impl-trait operand
  family's **shared** heart (both `dyn_trait.rs` and the non-subtree half of `impl_trait.rs` route
  through it, so they need no guard of their own).
- `impl_trait.rs::impl_trait_operand_subtree_findings` — `forbidden: &[String]`, impl-trait's
  **subtree** operand path, which bypasses the shared heart above (it walks `walk_subtree_modules`
  directly) and so carries its own copy.

All four take `&[String]`, call the identical predicate, and return the identical error shape — no
site's `forbidden` differs in type or origin in a way that would make extraction lossy.

`BACKLOG.md`'s `ACCEPTED DEBT` entry records that `containment.rs::matches_allowed` (backing
`trait_impl.rs`'s `allowed_locations`/`only_implemented_in` and `unsafe_confinement.rs`'s own
`allowed_locations`) has the identical defect shape, left unvalidated because the failure direction
was believed to already fail loud rather than silently pass. Reproducing it directly (a
control/treatment probe against `trait_impl_findings` and `unsafe_findings`, run against
the code exactly as it stood before this change) confirms the direction: `["::crate::api"]` on an
impl genuinely placed under `crate::api`, and `["crate::ffi::"]` on an `unsafe fn` genuinely placed
under `crate::ffi`, both produced a spurious violation (`Ok([finding])`), never a silent `Ok([])`
pass and never a panic. `path_within`'s plain `path == prefix || path.starts_with(prefix + "::")`
can never equal or prefix-contain a real canonical path against a malformed operand — the same
reasoning `has_empty_path_segment`'s doc already gives for the forbidden-operand direction, just
read from the opposite polarity.

## Goals / Non-Goals

**Goals:**
- One shared function behind every forbidden/allowed-operand-shaped DSL method's malformed-`::`
  guard, so wording and checked-before-any-resolution-work timing cannot drift between call sites.
- Reject a malformed `allowed_locations`/`only_implemented_in` entry as a constitution error (exit 2)
  in both `trait_impl.rs` and `unsafe_confinement.rs`, closing the named `BACKLOG.md` entry for both.
- Regression tests pinning the previously-observed spurious-violation behavior is gone, plus a
  well-formed control proving the fix does not touch a correctly-spelled entry.

**Non-Goals:**
- Extending this validation to any DSL method beyond the six now covered (four forbidden-operand
  families in Part 1, two allowed-location families in Part 2). No other allowed/forbidden-shaped
  surface was named by the `BACKLOG.md` entry or found during this investigation.
- Validating at DSL builder time (`only_implemented_in(...)`, `only_under([...])`). The existing
  forbidden-operand family validates at **check time**, in the pure `*_findings` heart, not the
  builder — this change keeps that placement rather than introducing a new validation timing the
  sibling capabilities don't share.
- Changing `matches_allowed`/`path_within` themselves, or the failure direction they produce for any
  *other* input shape. The fix is entirely "reject the malformed input before it reaches them,"
  never a change to their matching semantics.

## Decisions

### Decision 1: `validate_path_operands` lives in `resolve/mod.rs`, beside `has_empty_path_segment`

```rust
pub(crate) fn validate_path_operands(operands: &[String]) -> Result<(), String> {
    if let Some(bad) = operands.iter().find(|op| has_empty_path_segment(op)) {
        return Err(crate::errors::malformed_path_operand_error(bad));
    }
    Ok(())
}
```

`resolve/mod.rs` is `has_empty_path_segment`'s existing home and the crate's shared name-resolution
layer that every one of the six call sites already depends on for other resolution facilities
(`resolve_path_all`, `canonical_path_str`, …), so adding one more `pub(crate)` function here is not a
new dependency edge for any caller. `errors.rs` (home of `malformed_path_operand_error`) has no
dependency on `resolve` in the other direction, and Rust does not treat intra-crate module
references as a cycle the way crate-level dependencies are — there is no split-module workaround
needed here, unlike the `guibiao`↔`hunyi` dimension split `errors.rs`'s own doc comments describe.

The function takes the caller's **raw** operand list (pre-canonicalization) as its parameter name
signals, but nothing about it actually depends on that — see Decision 2.

### Decision 2: `unsafe_findings` validates its already-canonicalized `allowed`, not a second raw copy

`trait_impl_findings` receives `allowed: &[String]` as declared (raw) and canonicalizes it itself,
inline, after the new validation call. `unsafe_findings`, by contrast, receives `allowed` **already**
mapped through `canonical_path_str` by its caller (`check_unsafe_boundary`) — canonicalization
happens in the impure shell here, not the pure heart, an existing asymmetry this change does not
disturb.

Validating the already-canonicalized value is still correct: `canonical_path_str` only strips a
`r#`-raw-identifier prefix per segment (`path.split("::").map(strip_raw).join("::")`); it never
removes, merges, or otherwise collapses an empty segment. `"::crate::api"` canonicalizes to
`"::crate::api"` unchanged, and `has_empty_path_segment` reacts identically before or after that
step. Threading a second, separately-raw allowed list through `check_unsafe_boundary` purely so
`unsafe_findings` could validate "the truly raw" value would add a parameter for a distinction that
produces no observable difference — the drift law's minimalism bound.

Placement inside `unsafe_findings` itself (rather than in `check_unsafe_boundary`) is deliberate:
it keeps the guard in the pure, `cargo`-free heart, alongside the two existing allowed-set guards
(`is_empty`, `== "crate"`) it now sits beside — the same testability property every other guarded
call site in this change already has.

### Decision 3: No spec correction, only addition — neither spec claims tolerance today

Before drafting the delta specs, both `openspec/specs/semantic-trait-impl-locality/spec.md` and
`openspec/specs/semantic-unsafe-confinement/spec.md` were grepped in full for "malformed",
"empty segment", and "tolerate" (the vocabulary-drift discipline `AGENTS.md`'s adversarial-review
section requires whenever a change touches spec prose). Neither file makes any existing claim about
this shape, stale or otherwise — the ACCEPTED DEBT was tracked only in `BACKLOG.md` and the code's
own doc comments, never asserted as spec-level behavior. Both delta specs below are therefore pure
`ADDED Requirements`, mirroring `semantic-signature-coupling`'s existing "A malformed `::`-path
forbidden operand is a constitution error" requirement, reworded at the allowed-location polarity —
no `MODIFIED` section, and no risk of a diff-only sync leaving an older contradicting sentence behind
elsewhere in either file (full-file grep confirmed there is nothing else to find).

### Decision 4: `BACKLOG.md` entry closes fully, not partially

The entry names exactly two call sites — `unsafe_confinement`'s and `trait_impl`'s
`allowed_locations` — and both are fixed by the identical mechanism verified against both pure
hearts (Decisions above, plus the regression tests). There is no third named site left open, so this
closes the entry outright (moved to `BUILT / HISTORY`) rather than a partial reclassification.

## Risks / Trade-offs

- **[Trade-off] Adopter-facing effect on an already-malformed declaration.** A project that somehow
  shipped a malformed `allowed_locations`/`only_implemented_in` entry today gets a spurious per-site
  violation; after this change it gets a constitution error instead. Both are non-zero exit codes —
  no working CI run silently starts failing that previously passed — but the *shape* of the failure
  changes, which is exactly why this is a proposal rather than a silent patch. No legitimate
  declaration has a reason to write this shape (per `has_empty_path_segment`'s own doc: no canonical
  path this crate ever produces carries an empty segment), so the realistic population affected is
  "adopters who already had a confusing, unexplained CI failure," not "adopters with working config."
- **[Closed] No fixture or example in the repo carries this shape.** Checked across `crates/`,
  `examples/`, and the self-governance boundaries in `crates/tianheng/tests/self_governance.rs` —
  none declares an `allowed_locations`/`only_implemented_in` entry with an empty `::`-segment, so the
  Definition of Done should not surface an existing-fixture regression from this change.
