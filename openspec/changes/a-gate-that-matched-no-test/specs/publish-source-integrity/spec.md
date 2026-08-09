## MODIFIED Requirements

### Requirement: A publish SHALL run only from the tagged release commit on the remote's main

`cargo publish` SHALL be reachable only from a source where all of the following hold, **and only through a
gate observed to have run**. Each is committed state; none is about packaged content.

A wrapper that asks for this gate and reads only its exit status cannot tell *judged and clean* from *judged
nothing*: `libtest` exits `0` for a filter matching no test, so a renamed or silenced gate would let the
publish proceed with none of the conditions below checked. Reachability is therefore a property of the run
having happened, not of the command having been issued.

#### Scenario: The publish gate did not run

- **WHEN** the wrapper's gate invocation selects no test, or selects one that is ignored
- **THEN** the publish is refused before `cargo publish` is reached, and the refusal says the gate did not run
  rather than reporting the source clean
