## MODIFIED Requirements

### Requirement: A declared observation bound SHALL use one bare marker grammar

An observation bound SHALL be declared only by a scenario heading containing either the bare singular phrase `a stated bound` or `a documented bound`. Every repository consumer that enumerates declared bounds SHALL call the register parser's canonical marker predicate rather than reproduce that grammar. Article-less fragments, plural forms, and forms carrying an interposed qualifier SHALL NOT declare a bound.

#### Scenario: Either canonical bare singular marker declares a bound

- **WHEN** a scenario heading contains `a stated bound` or `a documented bound`
- **THEN** every bound enumerator includes the scenario through the same canonical predicate

#### Scenario: A near-miss marker does not declare a bound

- **WHEN** a scenario heading contains `stated bound`, `stated bounds`, `documented bounds`, or an interposed qualifier but not either canonical bare singular phrase
- **THEN** every bound enumerator excludes the scenario, so one gate cannot classify a population the register never declared
