## Context

`scripts/test_examples.sh` executes six isolated workspaces and asserts their quality, public shell,
structured projection, baseline, or runtime reactions. Its published-family ledger closes a
different set—the public DSL family inventory—but no reaction relates the script's manually listed
owners to the actual `examples/*/Cargo.toml` directories. The gate can therefore claim “every
example” while silently omitting a newly committed workspace.

The same script writes multiple machine projections to fixed `/tmp` paths. That is safe in one
serial invocation but collides across concurrent checkouts or parallel local runs. All examples are
repository fixtures, so this is CI/repository hygiene rather than a Tianheng observation dimension
or self-law amendment.

## Goals / Non-Goals

**Goals:**

- Close the live repository example set against successfully completed reaction owners.
- Prove both missing-owner and nonexistent-owner failure directions without running Cargo.
- Give each script invocation isolated, automatically cleaned projection artifacts.
- Keep the published-family ledger orthogonal: one closes examples, the other closes API families.
- Correct current-version comments without changing adopter manifests.

**Non-Goals:**

- Require one example per boundary modifier or method.
- Infer an example's family ownership from its manifest or source.
- Turn repository examples into workspace members or Tianheng constitution targets.
- Change any example's declared architectural fault, dependency declaration, or teaching scope.
- Add a generic fixture registry, manifest format, public API, or hosted-CI dependency.

## Decisions

### The filesystem is the example inventory source

A repository example is an immediate child of `examples/` containing `Cargo.toml`. The helper
enumerates that live set deterministically and compares it with names fulfilled by the driver. A
README or support directory is not an executable example; nested Cargo projects are owned by their
top-level example and do not become independent owners.

An alternative checked in a hard-coded second list. That would duplicate the list already embedded
in the driver's control flow and permit both copies to drift together. The directory set is the
independent observation source that makes completeness bite.

### Fulfillment follows the declared reaction

The driver calls `fulfill_example <name>` only after that example's quality gates and stable
reaction assertions have completed. The helper rejects a claim whose live directory is absent, and
the final verifier rejects any live directory with no fulfilled claim. Registration is repository
test state only; it is never projected into product reports or public metadata.

The example ledger remains separate from the published-family ledger because their cardinalities
and change triggers differ. One example may own several families, and a teaching example can remain
important even when another owner covers the same family.

### Focused tests inject an example root

The helper accepts the example-root directory as its only configuration, defaulting to the
repository `examples/` path in the driver. A focused shell test creates temporary directories with
minimal `Cargo.toml` markers and exercises unknown and missing ownership without invoking Cargo or
copying production examples.

### One temporary root owns every emitted artifact

`scripts/test_examples.sh` creates one invocation-local `mktemp -d` at startup, installs one EXIT
trap, and writes every JSON, SARIF, text, and baseline artifact beneath it. No fixed `/tmp` name and
no mid-script trap replacement remain. Cargo's per-example target directories are unchanged.

## Risks / Trade-offs

- **A support Cargo project under `examples/` is mistaken for an independent example.** → Only
  immediate children count; place nested support projects beneath their owning example.
- **An owner is registered before a late assertion.** → Place fulfillment as the final action in
  each example section and cover the helper's failure directions independently.
- **The two ledgers look duplicative.** → Keep their names and diagnostics explicit: example
  workspace completeness versus published boundary-family coverage.
- **A failed run removes evidence useful for debugging.** → Command diagnostics remain in the CI
  log; ephemeral machine files are implementation artifacts, not durable reports.
