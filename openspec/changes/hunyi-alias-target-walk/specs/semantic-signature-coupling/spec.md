## ADDED Requirements

### Requirement: Non-generic compound type aliases inspect nested nominal targets

The type alias extraction scanner SHALL inspect non-generic type alias declarations (`type Alias = Target;`) and walk nested compound type constructors—including references (`&T`), tuples (`(A, B)`), slices (`[T]`), arrays (`[T; N]`), groups, and parens—extracting all nested nominal target paths (`syn::Path`). Each nested nominal target SHALL be registered into the alias map so signature coupling boundaries react when a forbidden type is exposed through a compound type alias.

#### Scenario: Non-generic tuple type alias target is inspected

- **WHEN** a governed module declares `pub type Pair = (crate::infra::DbConn, String);` and exposes `pub fn get_pair() -> Pair` under `must_not_expose("crate::infra")`
- **THEN** the semantic boundary reacts on `crate::infra::DbConn` exposed via `Pair`

#### Scenario: Non-generic reference type alias target is inspected

- **WHEN** a governed module declares `pub type ConnRef = &'a crate::infra::DbConn;` and exposes `pub fn get_ref() -> ConnRef` under `must_not_expose("crate::infra")`
- **THEN** the semantic boundary reacts on `crate::infra::DbConn` exposed via `ConnRef`

#### Scenario: Non-generic slice type alias target is inspected

- **WHEN** a governed module declares `pub type ConnSlice = [crate::infra::DbConn];` and exposes `pub fn get_slice() -> ConnSlice` under `must_not_expose("crate::infra")`
- **THEN** the semantic boundary reacts on `crate::infra::DbConn` exposed via `ConnSlice`
