## ADDED Requirements

### Requirement: A live remote read SHALL preserve whether it failed

The publish-source gate SHALL distinguish failure to execute the live `refs/heads/main` read from a successful read that returns no such ref. A failed read SHALL be a cannot-judge naming the remote and preserving the read's cause. A successful response without `refs/heads/main` SHALL be a cannot-judge naming the absent ref. Neither condition SHALL be reported as a wrong-source violation or collapsed into the other's diagnostic.

#### Scenario: The live remote cannot be read

- **WHEN** `git ls-remote <remote> refs/heads/main` fails
- **THEN** the gate refuses as a cannot-judge naming the remote and the Git failure, rather than treating the response as empty

#### Scenario: The live remote has no main ref

- **WHEN** the live remote read succeeds but returns no `refs/heads/main`
- **THEN** the gate refuses as a cannot-judge naming the absent ref, distinct from a command that could not run
