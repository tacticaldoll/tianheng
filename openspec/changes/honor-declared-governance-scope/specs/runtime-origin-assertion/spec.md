## ADDED Requirements

### Requirement: Anonymous lexical scopes distinguish un-auditable probes

An un-auditable probe's complete lexical owner SHALL include anonymous block scopes that enclose a
named item, including closure bodies. Equal nested function names and expression text under
distinct closures in the same named owner SHALL remain distinct facts. The discriminator MUST NOT
use an absolute byte offset; equal structural siblings MAY use a parent-local discriminator that
is stable when a differently-shaped unrelated item is inserted.

#### Scenario: Equal nested functions under distinct closures stay distinct

- **WHEN** one function contains two closure bodies that each declare `fn inner()` with the same
  non-literal `assert_boundary!` expression
- **THEN** the audit emits two distinct un-auditable-probe identities

#### Scenario: Unrelated insertion preserves anonymous ownership

- **WHEN** a differently-shaped unrelated statement or item is inserted before one of those
  closures
- **THEN** the pre-existing closure probe retains its structured fact identity
