# Change: the observation protocol gets a consumer and a third-party implementor

## Why

`Observer::bounds` has **no consumer**. Measured: `git grep '\.bounds()'` over `crates/`, `examples/` and
`docs/` finds no call site outside a comment. The protocol's whole justification is that a participant cannot
join a run without declaring what it does not observe — and nothing reads the declaration, so a dimension could
answer anything and no verdict would move.

It is worse than a name with no reaction, because it looks answered. Three dimensions implement the method and
the register counts 54 classified bounds, so a reader concludes the answers are load-bearing. They reach the
register through each dimension's **free function**; the trait method is a parallel door nobody walks through.
The reaction that appeared to read it compared it against that same free function and could not fail — closed in
the change immediately before this one, which removed the last call site and made the gap plain.

Second, nothing in this repository has ever been a **third party** to the protocol. All three implementors are
family crates in the same workspace, each returning a literal list from its own module. Two claims therefore
stand untested: that an outside crate can join a run at all, and that an implementor whose bounds are
*discovered* rather than written can express them. The second is exactly what `BoundId`'s owned-or-borrowed form
was added for, and no code outside the family has ever used it — a capability shipped for a caller that does not
exist.

## What Changes

- **`observation-bound-model`'s bijection reads each dimension through `Observer::bounds`.** The register's
  verdict now depends on the trait method's answer. Verified: returning `Vec::new()` from one dimension's
  `bounds()` fails the bijection with 25 unclassified bounds. The shell's own declarations keep coming from its
  free function, because the shell composes dimensions rather than being one — stated rather than left to look
  like an oversight.
- **A new example, `observer-participant`**, is a crate outside the family that implements `Observer` and joins a
  composed run. Its bounds are **computed**, with ids built by `format!` over what it scanned, so the
  owned-or-borrowed `BoundId` gets its first caller that is not a literal. The examples gate runs it and requires
  it to react.
- **A COOKBOOK entry** under *Cross-cutting* shows the same shape at teaching size, so the adoption surface
  documents joining a run rather than only declaring boundaries.

## Impact

- Affected specs: `observer-protocol`, `observation-bound-model`
- Affected code: `crates/tianheng/tests/observation_bound_model.rs`, `examples/observer-participant/**`,
  `examples/README.md`, `scripts/test_examples.sh`, `COOKBOOK.md`
- No public API change: every type the example uses is already published. That is the point — if the example
  needed a new export, the protocol was not usable by a third party in the first place.
