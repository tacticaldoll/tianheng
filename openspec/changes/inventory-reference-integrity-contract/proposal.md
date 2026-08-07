## Why

The reference-integrity capability spec records only the two policy-isolation properties added most
recently, while the shipped gate and its failure matrix already enforce the larger reference-resolution,
failure, and read-only contract. The incomplete inventory makes established behavior invisible to future
changes and review.

## What Changes

- Inventory the gate's existing inspected corpus, recognized reference forms, and tracked-content rules.
- Record its existing ignore, OpenSpec-plan, illustrative-crate, glob, and ambiguous-basename bounds.
- Record fail-loud enumeration/read/normalization behavior and the 0/1/2, read-only output contract.
- Map every new scenario to an already-running failure-matrix or positive-gate direction; change no gate code.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reference-integrity`: Complete the specification inventory of behavior already enforced by
  `check_reference_integrity.sh` and `test_reference_integrity.sh`.

## Impact

Only the reference-integrity OpenSpec contract and lifecycle artifacts change. Scripts, CI behavior,
public APIs, manifests, package versions, dependencies, baselines, and Tianheng law remain unchanged.
