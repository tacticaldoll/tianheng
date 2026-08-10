## MODIFIED Requirements

### Requirement: The three-way contract SHALL survive as a type, not an exit code

A repository judgement that can both find a disagreement and fail to read its input SHALL carry the distinction in one shared Rust return type. Focused behavior matrices SHALL assert the result kind and actionable message for the externally meaningful shapes they exercise. The shared value and constructors SHALL remain ordinary repository-check code and SHALL NOT carry runtime mutation, reach recording, caller-location identity, or an exemption protocol.

#### Scenario: A repository judgement reaches both outcomes

- **WHEN** a repository judgement can both find a disagreement and fail to read its input
- **THEN** its shared result type names which outcome occurred and its focused directions assert the distinction

#### Scenario: A repository judgement cannot decide

- **WHEN** a repository judgement cannot read enough input to decide
- **THEN** it fails with the shared unverifiable outcome and an actionable message, never reporting agreement over unread input

### Requirement: A repository check that runs only on request SHALL be named where the run is decided

A repository check that does not run in an ordinary workspace test SHALL be named wherever its run is decided: on its own Definition-of-Done and CI line when its cost is accepted for pre-flight, or in the one workflow that invokes it when only that workflow supplies the state it can judge. Retired checks SHALL be removed from both command surfaces rather than retained as inert ritual.

#### Scenario: A retired env-gated check remains in one command surface

- **WHEN** an env-gated repository check is deleted but its command remains in the Definition of Done or CI
- **THEN** command coherence fails or the stale invocation fails, so both surfaces are retired together

## REMOVED Requirements

### Requirement: Every reached refusal site SHALL be distinguished in both its kind and its message

**Reason**: Constructor-site mutation makes internal implementation locations into governance identities and requires runtime instrumentation in every repository-check result.

**Migration**: Keep focused behavior tests for result kinds and messages; do not register or mutate constructor sites.

### Requirement: A perturbation selector SHALL NOT be an exemption identity

**Reason**: Perturbation selectors and refusal-site exemptions exist only for the retired mutation sweep.

**Migration**: Remove exemption slugs and construct ordinary unverifiable results.

### Requirement: The enumerated corpus SHALL be what compiles, and no refusal vocabulary SHALL sit outside it

**Reason**: Compiler-corpus enumeration was support for finding mutation sites, not a repository or product boundary.

**Migration**: Share the typed result through ordinary Rust imports; rely on the compiler and focused tests rather than a source vocabulary scan.

### Requirement: The reach recording SHALL fail loudly, and each guard SHALL be falsified by its own defect

**Reason**: Reach recording exists only to classify mutation sites and has no consumer after the sweep is removed.

**Migration**: None.

### Requirement: A refusal site the suite never reaches SHALL be red unless declared

**Reason**: An unreachable internal constructor is code-coverage information, not a product observation bound requiring a public declaration.

**Migration**: Delete dead branches when found; do not create product-visible exemptions for repository-test coverage.
