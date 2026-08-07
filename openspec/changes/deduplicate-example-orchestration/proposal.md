## Why

The Definition of Done and CI explicitly run all three example failure matrices before the positive example
driver, as required by the gate-shape contract. The driver also invokes those same matrices internally, so every
full run executes each proof twice without gaining another observation.

## What Changes

- Keep the focused example matrices as explicit top-level DoD and CI commands.
- Remove their duplicate invocations from the positive example driver.
- Correct the DoD commentary so it describes the actual ownership and ordering.

## Capabilities

### Modified Capabilities

- `governance-dogfood`: the focused failure matrices are separate top-level gates that precede the positive
  example driver; the driver does not recursively rerun them.

## Impact

CI and local DoD perform the same checks with less duplicate work. Example behavior, manifests, versions, and
public APIs are unchanged.
