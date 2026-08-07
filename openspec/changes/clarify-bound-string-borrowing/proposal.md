## Why

`BoundDecl::borrows_every_string()` inspects whether the declaration's string values are borrowed, but its API
documentation, changelog, canonical specification, and one test name broaden that answer into a claim about
everything a governance run allocates. Non-string storage such as a multi-pin `Vec` is outside the method's
observation, so the broader prose is false.

## What Changes

- State the contract as exhaustive string ownership, not whole-declaration or whole-run allocation behavior.
- Correct the canonical observation-bound-model requirement and scenario to preserve that observation limit.
- Rename the affected test and narrow explanatory prose without changing its assertions or implementation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-model`: clarify that literal declarations borrow every string they carry while allocations
  by non-string storage or the surrounding run are outside this answer.

## Impact

The change touches only OpenSpec prose, public Rust documentation, test naming/comments, and the unreleased
changelog. It changes no data structure, method body, public signature, law, dependency, manifest, package
version, reaction, or adopter behavior.
