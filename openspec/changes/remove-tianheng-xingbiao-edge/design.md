## Context

The self-constitution currently permits `tianheng` to depend directly on `xingbiao`, and the
manifest declares that edge, but the shell source does not use it. The shell obtains the metadata
work it needs through the dimension crates it composes. Because the allowlist is the enforcement
surface, deleting only the manifest entry would leave the architectural permission behind.

The boundary source is steward-owned and its Markdown projection is generated. The amendment must
therefore preserve accepted-state evidence, demonstrate the changed reaction, migrate the product,
and refresh the projection from the law source rather than editing it by hand.

## Goals / Non-Goals

**Goals:**

- Make the shell's normal-dependency allowlist match its actual direct dependency shape.
- Observe the tightened boundary reject the existing `xingbiao` edge before removing that edge.
- Demonstrate after migration that an unrelated disallowed direct edge still reacts.
- Keep the generated self-law projection byte-aligned with the accepted candidate.

**Non-Goals:**

- Change any dimension crate's dependency boundary or use of `xingbiao`.
- Remove `xingbiao` from the workspace or alter its public API.
- Change a baseline, package version, public Tianheng API, or release process.

## Decisions

### Tighten the law and migrate the manifest together

The self-constitution will remove `xingbiao` from only the `tianheng` normal-dependency allowlist,
and the manifest will remove the corresponding unused direct edge. This makes the reaction the
durable source of the architectural constraint. Removing only the manifest edge was rejected
because a later direct dependency would remain permitted.

### Keep the reason inside the observed perimeter

The boundary reason will describe only the allowed direct normal-dependency shape: the shell
directly composes `guibiao`, `hunyi`, and `louke`, with `serde_json` as its other normal dependency.
Historical motivation and the absence of source-level use stay in this design and git history;
the dependency observer cannot prove either claim from source semantics.

### Prove the reaction before and after migration

First, the law alone will be tightened while the old manifest edge remains, and the focused
self-governance test must fail with an enforced dependency violation. After removing that edge, the
same test must pass. A temporary direct normal dependency on another forbidden workspace crate will
then be introduced and must fail, showing that the candidate remains precise and active; the
temporary witness will not be committed.

## Risks / Trade-offs

- **The law could be weakened merely to match a cleanup.** → Preserve the accepted-state pass and
  require the law-first negative witness before product migration.
- **The candidate could accidentally alter adjacent dimension permissions.** → Restrict the source
  edit and projection diff to the `tianheng` boundary and review the generated output.
- **A manifest-only check could pass while the projection is stale.** → Regenerate through the
  blessed projection test, then run the unblessed freshness reaction.
- **A future direct metadata use would require another amendment.** → That friction is intentional:
  a new shell-to-substrate edge must be justified and reviewed rather than silently admitted.

## Migration Plan

1. Record the accepted self-law passing with the current direct edge.
2. Tighten the candidate boundary and observe the current edge fail.
3. Remove the manifest edge and align comments; observe the candidate pass.
4. Run and remove an independent forbidden-edge precision witness.
5. Regenerate the projection, run the full Definition of Done, and present the candidate for
   steward acceptance.

Rollback is the inverse reviewed amendment plus restoration of the dependency; no data migration or
external rollout is involved.

## Open Questions

None. The user explicitly authorized the stated tighten amendment; steward acceptance remains the
final authority gate after the candidate and its reaction evidence are visible.
