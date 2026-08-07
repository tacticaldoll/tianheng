## Context

Removing the unused shell-to-metadata dependency tightened the real self-constitution and was manually proven by adding `xingbiao` back to `crates/tianheng/Cargo.toml`. That observation was recorded in a PR, but no executable fixture repeats it. Existing self-governance fixtures already provide an isolated-workspace path for permanent negative evidence.

## Goals / Non-Goals

**Goals:**

- Make the direct `tianheng` → `xingbiao` violation run on every self-governance test invocation.
- Exercise the accepted shell boundary itself rather than copying its allowlist into test code.
- Keep the deliberate violation outside the production workspace dependency graph.

**Non-Goals:**

- Amend the shell dependency boundary or its reason.
- Test every member boundary against a dedicated dependency fixture.
- Change `GovernanceTest::test_fixture` or introduce a new fixture framework.

## Decisions

- Create a single-package fixture named `tianheng`, made its own workspace by `[workspace]`, with a path dependency on the repository `xingbiao` crate. A path edge avoids registry/network resolution and still presents the dependency name the normal-edge rule observes.
- Select and clone the unique `tianheng` `RestrictDependenciesTo` boundary from `tianheng_constitution()`, then place only that boundary in the fixture constitution. Running the full self-constitution against a one-package fixture would fail on absent unrelated targets; redeclaring the shell allowlist would create the second source the test is meant to avoid.
- Keep the fixture dependency set to `xingbiao` alone. Then `Outcome::Violations` can only be caused by the forbidden shell-to-metadata edge, and removing that edge makes the test fail cleanly instead of being masked by another forbidden dependency.

## Risks / Trade-offs

- **The fixture points at a repository crate** → The test is repo-only and already skips outside a checkout; using the local path keeps it hermetic and version-independent.
- **Boundary selection could silently choose the wrong declaration** → Match target and rule kind, collect the matches, and require exactly one before evaluating the fixture.
