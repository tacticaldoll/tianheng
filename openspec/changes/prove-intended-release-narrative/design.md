## Context

In development state, `check_release_coherence.sh` verifies that `[Unreleased]` has an adopter-facing list item and that its comparison link starts at the current workspace version. It intentionally does not parse version literals inside the item. The matrix exercises ordinary prose only, so an implementation that begins equating prose literals with the workspace version would not be distinguished.

## Goals / Non-Goals

**Goals:**

- Exercise a future intended version literal inside `[Unreleased]` while every enumerated version-bearing surface remains on the current release.
- Prove the fixture is classified as coherent development.
- Demonstrate that a temporary prose-version equality check makes this exact case fail.

**Non-Goals:**

- Adding any prose-number parser or detector to the release gate.
- Advancing workspace, dependency, example, or lockfile versions.
- Changing release-ready or snapshot behavior.

## Decisions

Build the case from the existing development fixture helper, then replace its generic adopter item with prose naming the next minor version. This holds every non-prose surface constant and makes the intended-version literal the only discriminating input.

Keep the assertion in `test_release_coherence.sh` rather than production code. The contract is an allowance: the gate's correct implementation is precisely to continue judging only its enumerated mutable surfaces. Adding parsing code merely to recognize and discard prose versions would create the detector the governance rule rejects.

Use an explicit future version different from both the current workspace version and its compatible minor requirement. This prevents the case from passing accidentally because two compared literals happen to agree.

## Risks / Trade-offs

- [A passing-only case could look vacuous] → Temporarily add the forbidden prose-equality check and record the focused fixture failing before removing it.
- [The case could accidentally become release-ready] → Retain the current workspace, lockfile, internal pins, example requirement, and `[Unreleased]` link; only commit a changelog prose edit after the latest release snapshot.
- [A general detector could be inferred from the test] → State explicitly in the spec and PR that the reaction does not parse this literal; the fixture defends its non-authority.
