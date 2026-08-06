# observation-bound-model (delta)

## MODIFIED Requirements

### Requirement: A dimension SHALL export its declarations as library items

Each dimension owning declared bounds SHALL expose them from its library, not from `#[cfg(test)]` code. A
declaration compiled only when its own crate is under test is invisible to every other crate, so no single
reaction could hold the specs and the code in bijection — and the protocol that follows this change requires an
observer to declare its bounds as part of joining a run, which a test-only item cannot satisfy.

The crates owning declared bounds are the three dimensions **and the shell**, which owns the bounds of the
capabilities whose reactions live in it — this capability's among them, since a capability that exempted itself from
its own bijection would count everyone else's unclassified bounds while hiding its own. Their number is deliberately
not written here: a census belongs to whatever enumerates the set, and `check_bound_register.sh` prints it on every
clean run. A crate with no declared bound SHALL gain no export: an empty accessor would be a name with nothing
behind it.

#### Scenario: A dimension's declarations are readable from another crate

- **WHEN** a reaction in the composed shell enumerates every declared bound
- **THEN** it reads each dimension's exported declarations directly, with no test-only visibility and no
  duplicated list
