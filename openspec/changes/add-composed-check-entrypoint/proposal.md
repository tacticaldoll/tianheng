## Why

An adopter can declare one composed `Constitution`, but can inspect its combined reaction only by
driving the CLI shell; library tests must split back into static and semantic checks and cannot
exercise the same runtime-audit composition. The composed example documents this workaround, so the
testing friction is observed rather than hypothetical.

## What Changes

- Add `check_constitution(&Constitution, &Path) -> Outcome` as the presentation-free
  library entrypoint for evaluating all three declared dimensions against an explicit manifest.
- Extract the existing runner composition into one shared evaluation path so CLI `run` and the new
  library check cannot drift in dimension ordering, error precedence, or runtime orphan-probe audit.
- Add `check_constitution` to the composed prelude's reaction-inspection tier and document its scope.
- Replace the composed example's per-dimension test workaround with a unified Outcome assertion that
  proves both static and semantic findings are present while runtime probe coverage stays clean.
- Keep baseline I/O, coverage advisories, formats, argument parsing, and process output in `run`.
  No package version or dependency change is part of this change.

## Capabilities

### New Capabilities

- `composed-library-check`: Defines an inspectable library evaluation of a unified Constitution with
  the same three-dimension reaction semantics as the CLI runner.

### Modified Capabilities

- `adopter-surface`: Adds `check_constitution` to the wildcard prelude's reaction-inspection tier.

## Impact

- Affects the Tianheng runner's internal composition seam, its public prelude, unit/integration
  tests, the composed dogfood example, adopter docs, and OpenSpec main specs.
- Adds one public function without renaming or removing the existing pure static `check` or `run`.
- Leaves dimension crates, the Constitution DSL, projections, baseline identity, process verdicts,
  manifests, lockfile, and package versions unchanged.
