## ADDED Requirements

### Requirement: An impl nested in a const or fn body is observed

The system SHALL observe a trait `impl` block that is written as a direct statement of the outermost body of a `const` initializer (a bare `{ … }` block expression) or of a `fn`'s own body — the "const-eval trick" idiom (`const _: () = { impl Trait for Type { … } };`, commonly used for a compile-time trait assertion or a doctest/dogfooding scratch impl) and its fn-body-nested sibling (`fn _also() { impl Trait for Type { … } }`) — exactly as if it were written at the enclosing module's own top level. Rust binds a trait `impl` to its self type's coherence set regardless of where the `impl` is lexically written, so wrapping it in a body does not change what it makes real; a walker that stops at a module's own top-level items therefore has a genuine observation gap here, distinct from the correct treatment of a body-nested `mod` (whose contents genuinely are unreachable as `crate::…`, an existing bound this requirement does not disturb). Recovery is bounded to exactly this shape: only an `impl` that is a DIRECT statement of the `const`/`fn`'s own outermost block is recovered — an `impl` nested one level FURTHER inside that body (inside an `if`/`loop`/closure/nested `fn`) is NOT recovered, and a `static` initializer is NOT inspected (the const-eval trick is specifically about `const`, which forces compile-time evaluation even when the binding is never read; no audited idiom uses `static` for it). Both bounds are stated rather than left silent.

#### Scenario: A const-wrapped disallowed trait impl reacts

- **WHEN** the boundary allows `impl Command` only under `crate::commands`, and `crate::rogue` declares `pub struct Rogue; const _: () = { impl Command for Rogue { fn run(&self) {} } };`
- **THEN** the system emits a violation identifying the offending impl by its location `crate::rogue` and the implemented-for type `Rogue`, rather than reporting zero findings because the impl sits inside a const initializer

#### Scenario: A fn-body-wrapped disallowed trait impl reacts

- **WHEN** the boundary allows `impl Command` only under `crate::commands`, and `crate::rogue` declares `pub struct Rogue2; fn _also() { impl Command for Rogue2 { fn run(&self) {} } }`
- **THEN** the system emits a violation identifying the offending impl by its location `crate::rogue` and the implemented-for type `Rogue2`, rather than reporting zero findings because the impl sits inside a fn body

#### Scenario: An impl nested one level further inside the body is a stated bound

- **WHEN** a disallowed module declares `fn _also() { if true { impl Command for Foo { fn run(&self) {} } } }`
- **THEN** the system does not claim to observe it — recovery covers only a direct statement of the const/fn's own outermost block, and this impl is one level further in, a stated coverage bound rather than a silent claim of cleanliness

#### Scenario: A static-wrapped impl is a stated bound

- **WHEN** a disallowed module declares `static S: () = { impl Command for Foo { fn run(&self) {} } };`
- **THEN** the system does not claim to observe it — only a `const` initializer or a `fn` body is inspected, never a `static` initializer, a stated coverage bound rather than a silent claim of cleanliness
