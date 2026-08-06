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

**Where a reaction is behind a Cargo feature, the declarations describing it SHALL be gated with it.** A bound is a
property of a *reaction*; a build that compiles none of that reaction and still exports its bounds tells a reader a
limit exists for something the crate does not contain — an unbacked claim, which is the one thing this model exists
to refuse, arriving through the export rather than through the declaration. Measured: 漏刻 declared six bounds
unconditionally while five of them describe `audit_probe_coverage`, a scanner behind its non-default `audit`
feature, and `mod observer` immediately beneath the export was already gated for exactly that reason. A dimension
whose reaction is wholly gated and which therefore has **no** declaration in a given configuration falls under the
rule above and exports nothing there.

#### Scenario: A dimension's declarations are readable from another crate

- **WHEN** a reaction in the composed shell enumerates every declared bound
- **THEN** it reads each dimension's exported declarations directly, with no test-only visibility and no
  duplicated list

#### Scenario: A build compiles none of the reaction a declaration describes

- **WHEN** a dimension's reaction sits behind a Cargo feature and that feature is off
- **THEN** the declarations describing that reaction are absent from the export, so no dependent reads a bound for
  a reaction its build does not contain — while any declaration describing an always-present path stays
