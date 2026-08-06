## ADDED Requirements

### Requirement: A bound declaration SHALL carry owned-or-borrowed strings, so a bound may be declared at runtime

Every string a bound declaration carries SHALL be owned-or-borrowed (`Cow<'static, str>`) — its id, the shape it
names, its rationale, and the layer an inherited ownership names — so a declaration whose value is computed is
expressible while one written as a literal allocates nothing.

`Observer::bounds` carries no default body, so declaring bounds is a condition of implementing the protocol. An
implementor whose bounds are not compile-time literals — an observer over a discovered plugin set, or over roots it
scanned — was therefore mandated to declare its limits and given no way to name them. That is the shape this
requirement exists to remove.

The family's own declarations SHALL remain literals. A bound is a property of the *reaction*, and this family's
reactions know their limits when they are written; the owned form exists for implementors whose reactions do not.

The constructors SHALL accept anything convertible, so a declaration written as a literal reads the same as before.
The accessors SHALL lend `&str` borrowed from the declaration rather than promising `&'static str`, which is what an
owned-or-borrowed value can honestly lend.

#### Scenario: A bound whose id and rationale are computed

- **WHEN** an implementor declares a bound whose id, shape or rationale is built at runtime
- **THEN** the declaration is expressible, and the bound behaves exactly as a literal one — the same extent, the
  same derived evidence, and the same refusal of a duplicate id

#### Scenario: A literal declaration is unchanged and allocates nothing

- **WHEN** a bound is declared from string literals, as every one of this family's own is
- **THEN** the call site is written exactly as it was, and the declaration borrows rather than allocating
