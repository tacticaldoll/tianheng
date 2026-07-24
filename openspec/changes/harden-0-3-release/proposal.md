## Why

The pre-0.3.0 adversarial review found two confirmed false negatives plus several gaps between the
implemented public surface, the synced specifications, and the adopter-facing release notes. The
breaking window is still open, so the identity and projection contracts should be made internally
consistent before release rather than repaired through another post-release migration.

## What Changes

- Close 圭表's mixed direct/conditional `#[path]` false negative by observing every physically
  existing candidate that can be selected by the written attributes, independent of attribute
  order.
- Make 漏刻's un-auditable-probe owner identity include the full lexical item context so identical
  nested functions or local impls in distinct enclosing functions cannot collide.
- **BREAKING**: make `Violation::target` read-only like the other identity components and migrate
  callers to an accessor.
- Correct module-boundary `scan_depth` projection so legacy subtree boundaries remain
  byte-compatible and explicit shallow scope is visible.
- Dogfood `GovernanceTest` for self-law projection freshness, give all projection gates one
  `BLESS=1`/`BLESS=true` interpretation, and add executable coverage for both fresh and blessing
  paths.
- Reconcile the 0.3.0 CHANGELOG with the public testing harness, ScanDepth, cfg-aware path
  observation, and the actual identity migration; consolidate duplicate headings.
- Restore the deferred baseline debt-ratchet WATCH decision lost during backlog pruning.
- Keep all workspace and internal dependency versions at 0.2.3 during this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `module-boundary`: clarify mixed direct and conditional path-remap observation and projection of
  non-default shallow scan depth.
- `runtime-origin-assertion`: require lexical owner identity to distinguish equal nested items in
  distinct enclosing items.
- `structured-violation-identity`: require every identity component, including target, to remain
  read-only after construction and clarify that this capability does not define the separately
  specified testing harness.
- `reusable-testing-harness`: require one explicit BLESS interpretation and executable projection
  freshness coverage.
- `governance-dogfood`: require Tianheng's own projection freshness reaction to execute through
  `GovernanceTest`.

## Impact

Affected code spans `guibiao` module reachability/projection, `louke` audit identity, the `xuanji`
violation model and its downstream readers, and `tianheng` testing/self-governance. Specifications,
CHANGELOG, BACKLOG, and focused regression tests change with the implementation. No dependency or
version changes are introduced.
