# Change: Unify observation-bound marker parsing

## Why

The observation-bound register and the typed bound-model gate independently decide whether a scenario heading declares a bound. Their predicates have already diverged: the register admits only the two bare singular markers, while the model also admits unqualified and plural fragments. The two gates can therefore enumerate different spec sets before comparing or projecting them.

## What Changes

- Make the typed bound-model gate consume the register's canonical marker predicate.
- Pin both admitted bare singular markers and representative near-misses at the shared predicate.
- Keep slug derivation independently checked; only the declaration grammar becomes single-source.

## Capabilities

### Modified Capabilities

- `observation-bound-model`: the spec-side enumeration uses the canonical register marker grammar.
- `observation-bound-register`: the marker predicate is explicitly reusable by every consumer that enumerates declared bounds.

## Impact

- Affects `crates/kanhe/src/bound_register_parse.rs` and `crates/kanhe/tests/observation_bound_model.rs`.
- Does not change any published crate or adopter-facing API.
- Headings previously admitted only by the model cease to count as declared bounds; the register's existing grammar remains authoritative.
