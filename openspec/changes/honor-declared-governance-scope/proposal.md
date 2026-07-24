## Why

The final 0.3.0 adversarial review found places where Tianheng records declared scope but does not
honor it in the reaction: three module-rule families ignore `ScanDepth::Shallow`, anonymous closure
ownership can collapse distinct runtime audit findings, and workspace coverage ignores semantic
crate targets. The same sweep found public-surface and backlog prose drifting from shipped facts.

## What Changes

- Apply module-boundary scan depth consistently to all five generic module-rule families.
- Include anonymous closure scopes in Louke's complete lexical owner so distinct probes cannot
  collapse under equal nested function names.
- Count every static and semantic crate target in `GovernanceTest` workspace coverage, while
  keeping runtime seams out because they declare no crate identity.
- Compile-react every promised wildcard-prelude type and the `NoExistentialLeak` composed builder.
- Resolve the stale adopter-surface contradiction in favor of the already-published
  `GovernanceTest` harness.
- Restore the live `xuanji-sink` WATCH trigger with its evidence, risk, compatibility class,
  promotion trigger, and authority.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rule-model-surface`: Require `ScanDepth` to affect every generic module rule that exposes it.
- `runtime-origin-assertion`: Extend complete lexical ownership through anonymous closure scopes.
- `reusable-testing-harness`: Define coverage over crate-targeting static and semantic boundaries.
- `adopter-surface`: Make the shipped harness, depth selector, and composed profile explicit
  prelude promises with an external compile reaction.

## Impact

The implementation touches Guibiao module evaluation and tests, Louke's audit scanner and identity
tests, Tianheng's public testing harness and external-view compile contract, four main capability
specifications, and `BACKLOG.md`. It adds no dependency, changes no manifest or package version,
and remains at 0.2.3 until release preparation.
