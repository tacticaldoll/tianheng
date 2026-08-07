## MODIFIED Requirements

### Requirement: A bound declaration SHALL carry owned-or-borrowed strings, so a bound may be declared at runtime

Every string a bound declaration carries SHALL be owned-or-borrowed (`Cow<'static, str>`) — its id, the shape it
names, its rationale, and the layer an inherited ownership names — so a declaration whose value is computed is
expressible while one written from literals borrows every string value it carries. This contract observes string
ownership only; it SHALL NOT be presented as measuring allocations by non-string storage or by the surrounding
governance run.

`Observer::bounds` carries no default body, so declaring bounds is a condition of implementing the protocol. An
implementor whose bounds are not compile-time literals — an observer over a discovered plugin set, or over roots it
scanned — was therefore mandated to declare its limits and given no way to name them. That is the shape this
requirement exists to remove.

The family's own declarations SHALL remain literals, and **a reaction SHALL measure that** rather than the
requirement asserting it in prose. A bound is a property of the *reaction*, and this family's reactions know their
limits when they are written; the owned form exists for implementors whose reactions do not. Since every
constructor accepts anything convertible, a family declaration rewritten as a computed string compiles and
allocates on every run of the register and the projection, and nothing would name it — a normative rule with no
reaction, in the capability that exists to refuse exactly that.

A declaration SHALL therefore be able to answer, of itself, whether every string it carries borrows. The answer
SHALL reach every string, including those nested in the extent and in an inherited owner's layer name, and SHALL be
decided by exhaustive matching **within the declaring crate**, so a variant added later carrying a string of its own
fails to compile rather than being silently unmeasured.

The reaction SHALL be shown able to answer **`false`**, for each string position independently. A discriminant that
is a constant `true` measures nothing, and one written as a single short-circuiting chain can pass while examining
only its first field.

The constructors SHALL accept anything convertible, so a declaration written as a literal reads the same as before.
The accessors SHALL lend `&str` borrowed from the declaration rather than promising `&'static str`, which is what an
owned-or-borrowed value can honestly lend.

#### Scenario: A bound whose id and rationale are computed

- **WHEN** an implementor declares a bound whose id, shape or rationale is built at runtime
- **THEN** the declaration is expressible, and the bound behaves exactly as a literal one — the same extent, the
  same derived evidence, and the same refusal of a duplicate id

#### Scenario: A literal declaration is unchanged and borrows every string

- **WHEN** a bound is declared from string literals, as every one of this family's own is
- **THEN** the call site is written exactly as it was, and every string value is borrowed rather than owned,
  without making a claim about other allocations

#### Scenario: One of the family's own declarations is rewritten as a computed string

- **WHEN** any string in any of this family's declarations becomes an owned value
- **THEN** the reaction fails, naming that declaration, because the rule that they stay literal would otherwise
  hold only for as long as nobody tested it

#### Scenario: A declaration carries a computed string in exactly one position

- **WHEN** a declaration owns its id, or its shape, or its pin, or its extent's rationale, or its inherited layer
  name, and borrows the rest
- **THEN** it answers that it does not borrow every string, whichever position it was — so the discriminant cannot
  pass by examining only the first
