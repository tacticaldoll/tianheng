## Why

The examples CI claims that every adopter-shaped example reacts, but its six owners are invoked by
hand with no comparison against the live `examples/*` set. A newly added example can therefore
remain entirely untested while the completeness claim stays green; the final 0.2.x audit also found
shared fixed `/tmp` outputs and stale 0.1 dependency comments in that same workflow.

## What Changes

- Make the examples gate compare every repository example workspace with an owner registered only
  after its quality and declared reaction assertions succeed.
- Fail loudly when a repository example has no fulfilled owner or when the script claims an example
  that does not exist.
- Add focused temporary-tree tests for both completeness failure directions and for unique,
  failure-cleaned artifact roots.
- Keep all example-run artifacts inside one invocation-local temporary directory and clean it on
  every exit, so concurrent runs cannot collide or leave shared files behind.
- Refresh Cargo/CI commentary to name the current 0.2 adopter dependency form.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `governance-dogfood`: Extend the existing example dogfood requirement from behavior checks over
  known owners to a closed completeness reaction over the live repository example set.

## Impact

The change affects repository-only example test helpers, `scripts/test_examples.sh`, focused shell
tests, CI/Cargo comments, BACKLOG, CHANGELOG, and the governance-dogfood specification. It adds no
Tianheng constitution boundary, public Rust API, dependency, report/baseline/identity wire change,
manifest dependency change, package-version bump, tag, or release action.
