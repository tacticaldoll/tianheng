## ADDED Requirements

### Requirement: Un-auditable probe identity includes lexical ownership
An un-auditable runtime probe fact SHALL identify its complete enclosing lexical item context within
the source file. Equal nested function names, methods on equal local type names, or local impl
contexts in distinct enclosing functions SHALL remain distinct without using byte offsets,
traversal ordinals, or collection positions.

#### Scenario: Equal nested functions in distinct outer functions remain distinct
- **WHEN** two outer functions in one file each define the same-named nested function containing byte-identical non-literal probes
- **THEN** the audit emits two distinct structured fact identities so baselining either cannot suppress the other

#### Scenario: Unrelated insertion preserves lexical identity
- **WHEN** an unrelated item is inserted before a nested un-auditable probe
- **THEN** the probe retains the same structured fact identity

