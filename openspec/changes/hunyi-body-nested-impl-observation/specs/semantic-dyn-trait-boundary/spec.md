## ADDED Requirements

### Requirement: An impl nested in a const or fn body is observed

`semantic-signature-coupling` states, on behalf of every single-module-anchored semantic capability that observes an inherent impl's public API, that an `impl` block written as a direct statement of the outermost body of a `const` initializer or a `fn`'s own body (the "const-eval trick" idiom and its fn-body-nested sibling) SHALL be observed exactly as if written at the module's own top level, bounded to one level deep and to `const`/`fn` only (never `static`, never a further-nested `impl`, never any OTHER item kind recovered from a body this way). This capability applies that same property to an exposed `dyn` shape in an inherent impl's public method signature or public associated item.

#### Scenario: A const-wrapped inherent impl's dyn-returning method reacts

- **WHEN** a governed module declares `pub struct Svc; const _: () = { impl Svc { pub fn dynamic(&self) -> Box<dyn crate::Port> { … } } };`
- **THEN** the system reports `dyn crate::Port exposed by fn <crate::m::Svc>::dynamic`, rather than reporting zero findings because the impl sits inside a const initializer

#### Scenario: A fn-body-wrapped inherent impl's dyn-returning method reacts

- **WHEN** the identical impl is instead written `fn _also() { impl Svc { pub fn dynamic(&self) -> Box<dyn crate::Port> { … } } }`
- **THEN** the system reports the identical finding, rather than reporting zero findings because the impl sits inside a fn body
