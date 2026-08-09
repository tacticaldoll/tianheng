## ADDED Requirements

### Requirement: A citation answered twice SHALL fail, whichever answer is repeated

A bound carrying more than one `UNPINNED` bullet SHALL fail, naming the bound, exactly as one carrying both a
`PINNED-BY` and an `UNPINNED` does. Two trackers are two answers to the question a citation exists to answer,
and the declaration holds one tracker, so silently keeping one of them records a bound whose owner is whichever
line happened to be last.

Repeated **`PINNED-BY`** SHALL remain accepted. That asymmetry is deliberate and already stated: several
pinning tests are several defences of one bound, while several trackers are several owners of one gap. A repair
that flattened the two would break a live declaration.

#### Scenario: A bound carries two `UNPINNED` citations

- **WHEN** a bound scenario carries more than one `UNPINNED` bullet
- **THEN** the reaction fails naming the bound id, because a bound that answers the citation question twice
  records nothing

#### Scenario: A bound carries two `PINNED-BY` citations

- **WHEN** a bound scenario carries more than one `PINNED-BY` bullet
- **THEN** the reaction accepts it and retains both, because several tests defending one bound is not two
  answers to one question
