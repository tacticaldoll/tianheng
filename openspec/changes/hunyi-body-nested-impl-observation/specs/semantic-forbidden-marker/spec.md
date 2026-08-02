## ADDED Requirements

### Requirement: A hand-impl nested in a const or fn body is observed

The system SHALL observe the hand-`impl T for X` acquisition form when the `impl` is written as a direct statement of the outermost body of a `const` initializer (a bare `{ … }` block expression) or of a `fn`'s own body — the "const-eval trick" idiom and its fn-body-nested sibling, the identical shape `semantic-trait-impl-locality` states this property for, since both capabilities read the crate-wide `impl` collection this requirement's observation is drawn from. Recovery carries the identical bounds: only an `impl` that is a DIRECT statement of the `const`/`fn`'s own outermost block is recovered — one level further in, or a `static` initializer, is NOT — stated rather than left silent.

#### Scenario: A const-wrapped hand-impl reacts

- **WHEN** `crate::wire` declares `const _: () = { impl serde::Serialize for crate::domain::Order {} };` under a boundary forbidding `serde::Serialize` on `crate::domain`, and `Order` is defined under that subtree
- **THEN** the system emits a violation identifying `impl serde::Serialize for crate::domain::Order in crate::wire`, rather than reporting zero findings because the impl sits inside a const initializer

#### Scenario: A fn-body-wrapped hand-impl reacts

- **WHEN** `crate::wire` declares `fn _also() { impl serde::Serialize for crate::domain::Order {} }` under the identical boundary
- **THEN** the system emits the identical violation, rather than reporting zero findings because the impl sits inside a fn body

#### Scenario: An impl nested one level further, or static-wrapped, is a stated bound

- **WHEN** the hand-impl is written one level further inside the body (inside an `if`/`loop`/closure/nested `fn`), or the wrapping binding is a `static` rather than a `const`
- **THEN** the system does not claim to observe it, a stated coverage bound shared with `semantic-trait-impl-locality`'s identical bound on the same underlying observation, rather than a silent claim of cleanliness
