## ADDED Requirements

### Requirement: An enumeration SHALL NOT pass over content it failed to read

Every enumeration this judgement makes SHALL distinguish **absent** from **unreadable**, and SHALL refuse as a
cannot-judge on the second. Skipping is reserved for what genuinely is not there.

A directory entry that fails to yield SHALL be propagated rather than dropped, and a manifest that exists and
cannot be read SHALL refuse rather than be skipped. Collapsing the two lets the remaining readable members
satisfy the counters, so the run reports clean over the very file it could not read — and the counters are what
the judgement then reasons from.

Where propagating produces a refusal no fixture can construct, it SHALL be declared out of reach with a slug of
its own. A slug shared between two sites excuses whichever one was looked at.

#### Scenario: A manifest exists and cannot be read

- **WHEN** an example manifest is present but is not readable as text
- **THEN** the judgement refuses as a cannot-judge naming the path, rather than skipping it as though the
  directory held no manifest

#### Scenario: A directory holds no manifest at all

- **WHEN** a directory under the enumerated root has no `Cargo.toml`
- **THEN** it is skipped, because absence is not a failed read

#### Scenario: A directory entry cannot be yielded

- **WHEN** iterating an enumerated directory fails part-way
- **THEN** the judgement refuses rather than continuing over the entries it did receive
