## Context

Four independent surfaces currently lose declared context between construction and reaction:
Guibiao stores one depth on every generic module boundary but three evaluation branches rebuild a
subtree scope; Louke qualifies named items but drops anonymous closure blocks; the testing harness
asks Guibiao for static-only coverage; and the external compile contract does not name several
already-promised prelude exports. A backlog pruning pass also removed a still-live architectural
trigger while retaining shipped history.

## Goals / Non-Goals

**Goals:**

- Make every public `.depth(Shallow)` modifier alter the corresponding observation.
- Preserve distinct Louke facts across anonymous lexical scopes without absolute byte positions.
- Define workspace coverage from every boundary that actually declares a crate target.
- Turn shipped wildcard-prelude promises into compilation reactions.
- Restore the live sink decision as a governed WATCH entry.

**Non-Goals:**

- Giving runtime seam boundaries a synthetic crate target.
- Changing legacy `Subtree` behavior, baseline identity, or projection bytes.
- Adding `syn` to Guibiao or Louke.
- Reopening the `xuanji-sink` design before its recorded multi-instrument trigger fires.

## Decisions

- For inbound module rules, depth controls matching of the protected target: `Shallow` matches the
  exact governed module, while `Subtree` retains prefix containment. For external confinement,
  depth controls the permitted importer scope in the same way. Crate-wide source discovery remains
  necessary for both rule directions.
- Louke will track anonymous brace scopes that are not already consumed as named
  `mod`/`impl`/`trait`/`fn` bodies. Each scope receives a normalized structural header and a
  discriminator only among equal-header siblings under the same lexical parent. This distinguishes
  equal closure bodies without using absolute byte offsets; inserting a differently-shaped
  unrelated item does not re-key the existing scope.
- `GovernanceTest` will read workspace member names once and subtract the union of Guibiao crate
  targets and every Hunyi boundary's `crate_package()`. Runtime boundaries do not participate:
  their observable target is a seam, so treating one as a crate would invent identity absent from
  the declaration.
- `GovernanceTest`, `ScanDepth`, and `NoExistentialLeak` remain in the wildcard prelude. The
  integration test will name each type and construct the composed profile without executing a
  scan.
- The backlog restoration will use the current evidence schema rather than restoring the old
  milestone narrative verbatim.

## Risks / Trade-offs

- [Risk] Anonymous-scope scanning can misclassify braces in expressions. → Reuse the existing
  literal/comment skipping and named-body consumption, then test equal closures and insertion
  stability.
- [Risk] Depth semantics differ by inbound/outbound direction. → Express matching through one
  exact-or-subtree helper and add Shallow/Subtree pairs for all five generic rule kinds.
- [Risk] Coverage performs an additional metadata read in the test harness. → Keep this in the
  repository-test utility; correctness is more important than avoiding one test-only process.
- [Risk] Semantic target enumeration can drift as capabilities grow. → Centralize iteration on
  `SemanticBoundaries` rather than hand-enumerating in the harness.
