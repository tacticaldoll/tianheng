## Context

`hunyi` scans Rust source files using `syn` to construct an `AliasMap` (`scan.aliases`) mapping type aliases (`type Alias = Target`) to their resolved nominal target paths.
Currently, `alias_nominal_target` in `crates/hunyi/src/resolve/mod.rs` only recognizes plain nominal paths (`syn::Type::Path`). Non-generic aliases that wrap target paths inside Rust compound type constructors—such as references (`type Ref = &A;`), tuples (`type Pair = (A, B);`), slices (`type Slice = [A];`), arrays (`type Arr = [A; 4]`), groups, or parens—are skipped.

## Goals / Non-Goals

**Goals:**
- Extend type alias extraction to recursively collect all bare nominal target paths nested within non-generic compound type constructors (`Type::Reference`, `Type::Tuple`, `Type::Slice`, `Type::Array`, `Type::Group`, `Type::Paren`).
- Ensure every nested nominal path in a non-generic compound type alias is resolved and registered into `scan.aliases`.
- Preserve the explicit stated bound skipping generic type aliases (`type Alias<T> = ...`).

**Non-Goals:**
- Supporting generic type aliases (`type Alias<T> = ...`).
- Evaluating trait bounds or macro-expanded type constructors inside type aliases.

## Decisions

### Decision 1: Recursive nominal target extraction helper (`alias_nominal_targets`)
Instead of restricting target extraction to a single `Option<&syn::Path>`, `alias_nominal_targets(ty: &syn::Type, acc: &mut Vec<&syn::Path>)` recursively visits compound type nodes that contain nested types:
- `Type::Path`: if `qself.is_none()` and all segments have `PathArguments::None`, push `&tp.path`.
- `Type::Reference`: recurse on `tr.elem`.
- `Type::Tuple`: recurse on each `elem` in `tt.elems`.
- `Type::Slice`: recurse on `ts.elem`.
- `Type::Array`: recurse on `ta.elem`.
- `Type::Group` / `Type::Paren`: recurse on `elem`.

### Decision 2: Multi-target alias map insertion
In `crates/hunyi/src/scan.rs`, `syn::Item::Type` calls `alias_nominal_targets` to collect all target paths and iterates through them. Each resolved nominal path is inserted into `scan.aliases` under the alias key `"{module}::{ident}"`.

## Risks / Trade-offs

- **[Risk] Multiple targets for a single alias key** → `AliasMap` uses standard insertion. In Rust, a tuple alias like `type Pair = (A, B);` exposes both `A` and `B`. When checking signature coupling, if either `A` or `B` resolves to a forbidden type, the alias coupling check correctly reacts on the forbidden target.
