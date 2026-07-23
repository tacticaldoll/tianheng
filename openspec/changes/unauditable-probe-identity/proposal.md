## Why

`audit_probe_coverage` currently dedups un-auditable-probe findings by **file only**
(`crates/louke/src/audit.rs`'s `unauditable_files` sort+dedup). Two textually distinct non-literal
`assert_boundary!` seam expressions in the same file collapse to one violation and one baseline
identity, so baselining one can silently mask a different, later-added un-auditable probe in the
same file — recorded in `BACKLOG.md` as accepted debt ("Un-auditable-probe finding identity is
file-granular"). The 0.3.0 semantic-identity migration makes closing this affordable: named,
content-derived fields are now the normal shape of a `StructuredFactIdentity`, so this is no longer
a special case requiring a bespoke identity mechanism — it is the same pattern every other dimension
already uses.

## What Changes

- Capture the offending non-literal seam expression's own source text (its first macro-argument
  span, trimmed) at each `capture_probe` construction site, modeling the parse on the existing
  `foreign_macro_body_end`/`skip_literal_or_comment` three-delimiter-depth infrastructure (today used
  for skipping a whole foreign macro body) rather than `balanced_brace_end` (curly-only, and
  therefore wrong for a seam expression containing a nested call or index).
- Add **owner-qualified** enclosing-item tracking to the audit source scanner — a new capability;
  the scanner today is a flat per-file linear scan with no enclosing-scope tracking at all. Not a
  bare innermost name: module path for a free `fn`; `Self` type (+ trait path, for a trait impl) for
  a method — mirroring `hunyi`'s existing owner/`trait_ref` qualification for the identical
  same-named-item collision. Stays `syn`-free, consistent with 漏刻's self-law.
- `RuntimeFact::UnauditableProbe`'s identity gains two named fields (the owner-qualified enclosing
  item, and the expression text) alongside the existing `file`, so distinct non-literal probes in
  the same file — including same-named methods on different owners — are distinct findings.
- `audit.rs`'s dedup key changes from `file` alone to `(file, owner-qualified enclosing item,
  expression text)`.
- Stated bound (deliberate, not silent): two byte-identical expressions in the same file, the same
  owner-qualified enclosing item, still collapse to one finding — at that granularity no further
  source content distinguishes them (mirrors `module-boundary`'s existing "same import repeated is
  one violation" precedent).
- **Acknowledged divergence:** `BACKLOG.md`'s own accepted-debt entry for this gap previously
  sketched "qualify by byte offset / occurrence index" as a future remediation. This change
  deliberately does not follow that sketch — it is superseded by the 0.3.0 identity model's
  no-positional-identity rule, not silently dropped.
- **Compatibility note (not BREAKING):** this is a false-negative closure — a dirty codebase with
  multiple distinct un-auditable probes in one file will see its violation count increase and its
  existing baseline entries go stale, following the same v0.1.3 re-export-exposure precedent
  (`--write-baseline` ratchets it down; no wire-format or public-API change).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `runtime-origin-assertion`: the un-auditable-probe fact's identity requirement changes from
  file-granular to (file, owner-qualified enclosing item, expression-text)-granular, with the
  byte-identical same-scope collision explicitly stated as a bound.

## Impact

- `crates/louke/src/audit/scan.rs`: `Probe::Unauditable` gains fields; `capture_probe` gains
  expression-span parsing; new enclosing-item tracking added to the source walk.
- `crates/louke/src/audit.rs`: `audit_probe_coverage`'s un-auditable-probe dedup key changes from
  `file` to a 3-tuple.
- `crates/louke/src/finding.rs`: `RuntimeFact::UnauditableProbe`'s named identity fields change.
- `openspec/specs/runtime-origin-assertion/spec.md`: new Requirement + scenarios.
- `BACKLOG.md`: the existing "Accepted debt" entry for this gap is resolved and removed/updated
  once this change ships.
- No public API change; existing baselines containing an un-auditable-probe entry go stale and
  require `--write-baseline` (the normal ratchet-down flow), never silently misinterpreted (a
  stale, pre-shape baseline entry simply stops matching, per the existing stale-detection path —
  it is not a "version" bump, since 0.3.0 baselines carry no version field at all).
