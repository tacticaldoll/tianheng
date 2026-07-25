# proposal: Hunyi Non-Generic Type Alias Target Walk

## Why

`hunyi`'s semantic type alias extraction (`alias_nominal_target`) currently extracts nominal targets only from plain `syn::Type::Path` expressions (`pub type TargetAlias = Target;`). When developers define non-generic type aliases wrapping nominal targets inside compound Rust type constructors—such as references (`type Ref = &Target;`), tuples (`type Pair = (A, B);`), slices (`type Slice = [Target];`), or arrays (`type Arr = [Target; 4]`)—the nested nominal paths inside the compound type are ignored, allowing forbidden types to escape semantic signature coupling boundaries when aliased through compound constructors.

## What Changes

- Refactor `alias_nominal_target` (or introduce a compound-walk helper `alias_nominal_targets`) in `crates/hunyi/src/resolve/mod.rs` to recursively walk non-generic Rust type structures (`Type::Reference`, `Type::Tuple`, `Type::Slice`, `Type::Array`, `Type::Group`, `Type::Paren`) and extract all nested nominal target paths (`syn::Path`).
- Update `crates/hunyi/src/scan.rs`'s `syn::Item::Type` handling to register all extracted nominal targets into the alias map `scan.aliases`.
- Add unit tests in `crates/hunyi/src/tests.rs` and `scan.rs` verifying that non-generic tuple, reference, slice, array, and grouped type aliases correctly resolve nested nominal targets.

## Capabilities

### Modified Capabilities

- `semantic-signature-coupling`: Update specification to require that non-generic type alias resolution inspects nested nominal targets within reference, tuple, slice, array, and grouped type constructors.

## Impact

- `crates/hunyi/src/resolve/mod.rs`: `alias_nominal_target` helper / target extraction.
- `crates/hunyi/src/scan.rs`: `syn::Item::Type` alias extraction loop.
- `crates/hunyi/src/tests.rs`: Conformance unit tests.
- Non-breaking extension; existing plain nominal path aliases remain unchanged.
