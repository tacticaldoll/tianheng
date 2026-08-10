## ADDED Requirements

### Requirement: The squash wrapper SHALL judge the complete live pull-request commit set

Before invoking the squash-message gate, the sanctioned merge wrapper SHALL resolve the accepted pull-request selector to one canonical numeric pull-request identity, then obtain every commit subject from that live pull request rather than deriving the set from local remote-tracking refs. The acquisition SHALL include all pages, SHALL derive the subject from the first line of each full commit message without headline truncation, and SHALL work when the pull request head belongs to a fork. Failure to resolve the identity or acquire the live set, or an acquired set containing no subjects, SHALL stop the workflow before the gate and before `gh pr merge`; it SHALL NOT construct an endpoint from the unresolved selector or fall back to a local subset.

#### Scenario: Local remote-tracking refs are stale

- **WHEN** the live pull request contains a commit absent from the local base-to-head ref range
- **THEN** the wrapper supplies the live commit's full subject to the squash-message gate, so a default body containing it cannot escape as an unrecognized shape

#### Scenario: Pull-request commits span multiple API pages

- **WHEN** the live pull request's commits require more than one response page
- **THEN** the wrapper supplies subjects from every page to the gate in pull-request order

#### Scenario: The live commit set cannot be acquired

- **WHEN** the pull-request commits read fails or yields no commit subjects
- **THEN** the wrapper exits non-zero before invoking the squash-message gate or `gh pr merge`, without substituting local refs

#### Scenario: The accepted selector does not resolve to one canonical number

- **WHEN** `gh pr view` does not return a positive numeric pull-request identity for the accepted selector
- **THEN** the wrapper exits non-zero before constructing the commits endpoint, invoking the squash-message gate, or invoking `gh pr merge`
