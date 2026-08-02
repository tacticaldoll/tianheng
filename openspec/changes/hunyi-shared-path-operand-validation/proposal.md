# proposal: Hunyi Shared Path-Operand Validation

## Why

Four call sites in 渾儀 guard a forbidden-operand list against the identical malformed `::`-path
shape (a leading, trailing, or doubled `::`, or the empty string) with the byte-identical 3-line
check, copy-pasted verbatim: `exposure.rs::module_findings`, `forbidden_marker.rs::
forbidden_marker_findings`, `shape_scan.rs::operand_module_findings` (the dyn/impl-trait operand
family's shared heart), and `impl_trait.rs::impl_trait_operand_subtree_findings` (the one operand
path that does not route through the shared heart, since the subtree walk bypasses it). Reading
each site's actual surrounding code confirms the guard is genuinely identical in shape at all
four — same `forbidden: &[String]`, same `has_empty_path_segment` predicate, same
`malformed_path_operand_error` — never merely similar, so extraction loses nothing.

Separately, `BACKLOG.md`'s `ACCEPTED DEBT` records that `unsafe_confinement`'s and `trait_impl`'s
`allowed_locations` (backing `only_implemented_in`/`allowed_locations` and unsafe-confinement's own
`allowed_locations`) have the identical malformed-`::`-path defect shape as the forbidden-operand
family did before `hunyi-forbidden-operand-colon-validation`, but were left unvalidated because the
failure direction was believed safe: a malformed *allowed* entry makes every real site look
disallowed, producing a spurious violation (fails loud, if noisily) rather than the silent,
permanent non-reaction the forbidden-operand fix closed.

Reproducing this directly against the pure hearts (`trait_impl_findings`, `unsafe_findings`) before
writing any fix confirms both halves of that characterization: an allowed entry spelled
`"::crate::api"` or `"crate::ffi::"` never matches any real module location in `matches_allowed`
(`path_within`'s plain equality/prefix check can never equal or prefix-contain a real canonical
path against a malformed operand, the identical reasoning `has_empty_path_segment`'s own doc already
states for the forbidden-operand direction), so a genuinely-in-place impl or `unsafe` site is
reported as a spurious violation instead of a clear, named constitution error pointing at the typo.
The direction is indeed the safe one (a false positive, never a false negative — the core contract's
one forbidden bug never occurs here), but the debt is real: an adopter with a typo'd
`allowed_locations` entry gets a confusing, unexplained violation instead of a diagnosis.

This is a genuine, if narrow, behavior change on that malformed-input path — a previously
noisily-failing declaration now fails a different, more honest way (a constitution error naming the
operand) instead of a spurious per-site violation — so, per this project's own OpenSpec-vs-plain-
commit line, it earns the full explore→propose→apply→sync lifecycle rather than a plain commit. This
is not the same shape as the sibling `refactor/hunyi-flatten-body-nested-impls-shared` change on this
branch history, which touched no declared boundary's observable reaction at all (a pure code-sharing
refactor) and was correctly a plain commit; this change's Part 2 half genuinely widens what a
constitution error now catches.

## What Changes

- Extract the shared guard into `resolve::validate_path_operands` (`crates/hunyi/src/resolve/mod.rs`,
  beside `has_empty_path_segment` itself, its natural home), and swap all four existing forbidden-
  operand call sites (`exposure.rs`, `forbidden_marker.rs`, `shape_scan.rs`, `impl_trait.rs`) to call
  it instead of repeating the inline check. No behavior change at these four sites — same message,
  same timing (checked before any resolution work), same `Result<(), String>` shape.
- Reuse that same function in `trait_impl.rs`'s `trait_impl_findings` (over `allowed_locations` /
  `only_implemented_in`) and `unsafe_confinement.rs`'s `unsafe_findings` (over its own
  `allowed_locations`), so a malformed allowed-location entry is now rejected as a constitution error
  (exit 2) before any scanning work — the behavior change this proposal exists for.
- Add regression tests in `crates/hunyi/src/tests.rs` pinning both halves: the fix itself (a
  malformed allowed entry now names itself in a constitution error) and a well-formed control (the
  identical genuinely-in-place impl/`unsafe` site still passes clean with a correctly-spelled entry).
- Add a matching `Requirement` to `openspec/specs/semantic-trait-impl-locality/spec.md` and
  `openspec/specs/semantic-unsafe-confinement/spec.md`, mirroring
  `semantic-signature-coupling`'s existing "malformed forbidden operand" requirement at the
  allowed-location polarity — neither spec currently makes any claim (stale or otherwise) about
  tolerating this shape, so this is a pure addition, not a correction of stale prose.
- Close the `BACKLOG.md` `ACCEPTED DEBT` entry for this gap (moved to `BUILT / HISTORY`), now that
  both named call sites (`trait_impl.rs` and `unsafe_confinement.rs`) are confirmed fixed.
- Add the adopter-facing `CHANGELOG.md` `[Unreleased]` entry.

## Capabilities

### Modified Capabilities

- `semantic-trait-impl-locality`: a malformed `::`-path `allowed_locations` entry is now a
  constitution error (exit 2), never a silent pass-through into `matches_allowed`.
- `semantic-unsafe-confinement`: the identical requirement for its own `allowed_locations`.

## Impact

- `crates/hunyi/src/resolve/mod.rs`: one new `pub(crate) fn validate_path_operands`.
- `crates/hunyi/src/exposure.rs`, `forbidden_marker.rs`, `shape_scan.rs`, `impl_trait.rs`: refactor
  their existing guard to call the shared function — no behavior change.
- `crates/hunyi/src/trait_impl.rs`, `unsafe_confinement.rs`: new validation call on
  `allowed_locations` — the behavior change.
- `crates/hunyi/src/tests.rs`: new regression tests for both fixed call sites, each with a
  well-formed control.
- `openspec/specs/semantic-trait-impl-locality/spec.md`,
  `openspec/specs/semantic-unsafe-confinement/spec.md`: one new requirement each.
- `BACKLOG.md`: close the named `ACCEPTED DEBT` entry.
- `CHANGELOG.md`: `[Unreleased]` entries.
- Non-breaking: no public API, DSL, or wire-format change — `TraitImplBoundary`/`UnsafeBoundary`'s
  builder surface and violation identity are untouched. The adopter-facing effect is narrow: an
  *already-malformed* `allowed_locations`/`only_implemented_in` declaration (one no legitimate
  project has a reason to write, per `has_empty_path_segment`'s own doc) now fails fast with a named
  constitution error instead of a confusing spurious violation. Every well-formed declaration's
  behavior is unchanged.
