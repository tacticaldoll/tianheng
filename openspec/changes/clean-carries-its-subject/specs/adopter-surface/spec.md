# adopter-surface delta

## MODIFIED Requirements

### Requirement: The prelude supports reaction inspection

`tianheng::prelude::*` SHALL expose the existing boundary, rule, baseline, report, violation, and
Outcome inspection surface, plus the vocabulary-neutral `RuleKey` and `StructuredFactIdentity`
types used by live `ViolationId`. The obsolete public `FindingKey` SHALL be removed as an
intentional 0.3.0 break. These names SHALL form an inspection tier, not a second construction path around
validated identity or builder-owned rules. Standalone instrument APIs SHALL expose the same reaction model
without requiring the composed facade.

**A clean outcome SHALL be inspectable for the subject it was reached over.** The Outcome inspection surface
therefore includes the `Subject` a clean verdict carries, and `Subject` SHALL be a promised prelude member like
the outcome that carries it. A consumer that can read a violation's target, rule key and structured fact but
can read nothing at all from a clean verdict cannot tell a workspace that was observed and found sound from one
that was never reached — and this surface exists so that judgement never requires decoding CLI text.

The public surface SHALL NOT promise a `Dimension`/`ObservedFact` plugin trait or runtime plugin
loading. Rust architecture tests MAY use the promised `GovernanceTest` harness or invoke the
existing pure standalone/composed checks and inspect structured `Outcome` values.

#### Scenario: A consumer inspects a composed reaction

- **WHEN** an external crate checks a unified `Constitution`
- **THEN** it can inspect target, rule key, structured fact, presentation, metadata, and outcome without decoding CLI text

#### Scenario: A consumer inspects a clean reaction

- **WHEN** an external crate checks a workspace and the reaction is clean
- **THEN** it can read what was declared and how much of the workspace was reached, so a sound workspace is
  distinguishable from an unreached one without decoding CLI text
