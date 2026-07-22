# 璇璣 / xuanji

**執璣為度,三儀同歸。** — *Hold the jade-pivot as the measure; the three instruments converge on it.*

**The shared reaction model of [Tianheng](https://github.com/tacticaldoll/tianheng) — the 底 (bedrock).**

璇璣 (the jade pivot of the armillary sphere) is the **dimension-agnostic measure** every
Tianheng observation dimension reacts in. It renders **no verdict** — it holds the measure but
never the react itself (comparing a declared boundary against observed reality lives in the
dimensions and the shell) — and depends on no other workspace member; every dimension sits
*above* it, reusing the reaction vocabulary without dragging in another dimension's engine.

It defines:

- `Severity` — `Enforce` (fails the reaction) or `Warn` (advisory).
- `RuleKey` — a validated semantic rule-family identity with canonical named scalar fields.
- `StructuredFactIdentity` / `Finding` — a validated semantic fact type and shape with canonical
  named scalar fields, paired with human presentation.
- `Violation` — the dimension-agnostic finding: `kind`, `target`, `rule`, human `finding`,
  structured `rule_key` and `fact`, `reason`, `severity`, `baselined`, and optional metadata.
  Identity is exactly `(target, RuleKey, StructuredFactIdentity)`; presentation and diagnostics
  are not identity.
- `Report` — every violation from one evaluation.
- `Baseline` — a generated `tianheng.baseline/structured-facts` snapshot of accepted violations,
  so a dirty project can adopt a boundary and gate only on *new* drift. Numeric and unmarked
  snapshots are intentionally rejected; the model reconstructs no identity from presentation.
- `Outcome` — `Clean` / `Violations(Report)` / `ConstitutionError(String)`, projected by the
  CI dimensions as an exit code (`0` / `1` / `2`) and by the runtime dimension as an event.
- `BoundaryKind` — `Crate`, `Module`, `Semantic`, `Runtime`: which instrument observed it.

`serde_json`-only by law: `Baseline` is a JSON snapshot and the per-type renderings are
intrinsic to those types, so they live here. Document *assembly* (folding in coverage, stale
baseline entries, constitution projection) stays in the dimensions and the shell, never in
the model.

You rarely depend on `xuanji` directly — you get it re-exported through the dimension crates
and the [`tianheng`](https://crates.io/crates/tianheng) shell.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
