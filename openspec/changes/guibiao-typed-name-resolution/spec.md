# Draft delta: inline-symbol-path-confinement

This is a proposed delta only. It is not the active requirement source until the implementation change is accepted and moved into `openspec/specs/inline-symbol-path-confinement/spec.md` with its reaction.

## Requirement: Inline path heads resolve from one typed lexical scope

For each recognized inline path occurrence, the system SHALL resolve the first identifier from the occurrence's lexical scope chain — each enclosing block, then its module and that module's ancestors as Rust makes them visible — using the package edition, namespace when observable, item declarations, named imports and aliases, `pub use` closure, glob-import edges, and external-prelude roots. A `use` or item written inside a block SHALL bind for that whole block and SHALL NOT bind outside it. A glob SHALL carry names visible from its resolved module, including a private import visible to a descendant scope through `use super::*`. In an edition-2015 package, a `use` path and a path beginning with `::` SHALL resolve from the crate root scope. The system SHALL distinguish a local binding from an external-prelude binding and SHALL NOT select among distinct candidate bindings: where a scope holds more than one, as cfg-exclusive imports do, the occurrence SHALL react when any candidate resolves under the prefix. A path beginning with `<` has no crate-root head and is not resolved; it is covered by the receiver-method and type-directed observation bound. Resolution SHALL NOT itself produce exit 2.

The resolved binding SHALL be canonicalized before prefix matching and before a `RuleKey` is formed. Equivalent raw-identifier spellings and equivalent root-qualified spellings SHALL have one identity where Rust resolves them to the same binding. Default and strict-external observation policy SHALL be applied after this resolution: an un-`use`d external dependency call remains outside the default policy and is observed under strict-external, independent of whether it is written `dep::item()` or `::dep::item()`.

#### Scenario: A private import inherited through a parent glob resolves in an inline test

- **WHEN** `crate::clock` privately imports `std::time::SystemTime`, a nested `#[cfg(test)] mod tests` declares `use super::*`, and its test calls `SystemTime::now()`
- **THEN** the system resolves the head to `std::time::SystemTime` through the parent scope and reports the forbidden call under `std::time`
- **PINNED-BY** `inline_private_use_inherited_through_super_glob_resolves`

#### Scenario: An inline module use resolves from its enclosing module

- **WHEN** a nested inline module imports `crate::clock::now` from its actual ancestor scope and calls `now()` under a boundary on `crate::core`
- **THEN** the system resolves the alias to `crate::clock::now` and reports the call
- **PINNED-BY** `an_inline_module_use_resolves_from_its_enclosing_module`

#### Scenario: An inline use does not leak to a sibling

- **WHEN** one inline child imports `crate::clock::now` and a sibling defines and calls its own `now`
- **THEN** the system keeps the two lexical scopes distinct and reports no violation for the sibling
- **PINNED-BY** `an_inline_module_use_does_not_leak_to_sibling_modules`

#### Scenario: A block-local use binds only inside its block

- **WHEN** `crate::core` imports `crate::a::X` at module level, one function calls `X::fa()`, and another function declares `use crate::b::X;` and calls `X::fb()`, in either textual order
- **THEN** under a prefix `crate::a` the system reports `crate::a::X::fa` only, and under `crate::b` it reports `crate::b::X::fb` only
- **PINNED-BY** `inline_block_local_use_binds_only_inside_its_block`

#### Scenario: A block-local item shadows a module import

- **WHEN** `crate::core` imports `std::process::Command` and a function body declares its own `struct Command` and calls `Command::new("x")`
- **THEN** the system resolves the head to the block's item and reports no violation under `std::process`
- **PINNED-BY** `inline_block_local_item_shadows_a_module_import`

#### Scenario: An inline type alias resolves at its inline path

- **WHEN** an inline child defines a type alias to `std::process::Command` and an occurrence calls the alias's `new`
- **THEN** the system resolves the alias to `std::process::Command` and reports the call
- **PINNED-BY** `an_inline_module_type_alias_resolves_under_its_inline_path`

#### Scenario: A local public re-export resolves across modules

- **WHEN** `crate::support` declares `pub use std::time::SystemTime`, `crate::core` imports that name, and calls `SystemTime::now()`
- **THEN** the system resolves the public re-export closure to `std::time::SystemTime` and reports the call
- **PINNED-BY** `inline_resolves_a_cross_module_local_reexport`

#### Scenario: Cfg-exclusive imports of one name are both observed

- **WHEN** `crate::core` writes `#[cfg(unix)] use crate::a::X;` and `#[cfg(not(unix))] use crate::b::X;` and calls `X::f()`
- **THEN** the system reports the call under a prefix `crate::a` and under a prefix `crate::b`, since cfg-gated source is observed as written
- **PINNED-BY** `inline_cfg_alternative_uses_are_both_observed`

#### Scenario: Edition-2015 root paths resolve from the crate root

- **WHEN** an edition-2015 package declares `mod clock` at its root, `crate::core` calls `::clock::now()`, and `crate::other` writes `use clock::now;` and calls `now()`
- **THEN** the system reports both calls under a prefix `crate::clock`
- **PINNED-BY** `inline_edition_2015_root_paths_resolve_from_the_crate_root`

#### Scenario: A local same-name module shadows an external dependency

- **WHEN** the crate root defines `mod md5x` returning a local `md5x::Local`, the package also depends on an external `md5x` returning a different type, and code calls bare `md5x::compute()`
- **THEN** the system resolves the bare head to the local module and does not match an external-only `::md5x` prefix
- **PINNED-BY** `inline_bare_local_module_shadows_same_named_dependency`

#### Scenario: An un-used dependency has the same answer under both root spellings

- **WHEN** the package depends on `md5x`, contains no local `md5x` binding, and writes `md5x::compute()` or `::md5x::compute()` without a `use`
- **THEN** both paths resolve to the same external binding; default mode reports neither, and strict-external reports each
- **PINNED-BY** `inline_external_dependency_root_spelling_is_mode_invariant`

#### Scenario: A qualified associated path is not treated as an extern root

- **WHEN** code declares `impl W { fn md5x() {} }` and calls `<W>::md5x()` under a `md5x` external-prefix boundary
- **THEN** the system does not treat the `::` after `>` as a leading root and reports no external-prefix violation
- **PINNED-BY** `inline_associated_path_after_angle_close_is_not_global_root`

## Requirement amendment: prefix spelling (active requirement "Constitution errors are loud, never silent")

Replace "and not starting with a keyword (`Self`, `self`, `super`, etc.)" and "or a keyword head" with:

> and starting with a head that can name a crate or module. Every segment is read with any `r#` removed, so `r#x` and `x` are one identifier, recorded without the raw prefix. The heads that can name nothing are those the Rust Reference's identifier grammar excludes: `_`, which is not an identifier, and `r#crate`, `r#self`, `r#super` and `r#Self`, which are not raw identifiers; with `self`, `super` and `Self`, which are relative to a module or type a declaration does not have, and `crate` after a leading `::`. Each SHALL be exit 2, suggesting the unraw spelling where that is a valid prefix. Any other head, keyword or not and in any edition, SHALL be accepted written bare or raw.

#### Scenario: A prefix head that names no crate or module is refused

- **WHEN** either builder is given `_`, `_::clock`, `r#_::clock`, `Self::now`, `r#Self::clock`, `self::clock`, `r#super::clock`, `::crate::clock` or `r#crate::clock`
- **THEN** the system exits 2 quoting the written prefix, suggesting `crate::clock` for `r#crate::clock` and nothing for `_`
- **PINNED-BY** `a_prefix_head_naming_no_crate_or_module_is_refused`

#### Scenario: A keyword head is accepted bare or raw

- **WHEN** a package depends on a crate renamed `async`, `crate::core` writes `use r#async::f;` and calls `f()`, and a boundary's prefix is `async` or `r#async` — and likewise for `dyn`, `try`, `gen` and `union` in editions 2018, 2021 and 2024
- **THEN** the system accepts the prefix, reports the call under it, and records one identity for both spellings
- **PINNED-BY** `a_keyword_prefix_head_is_accepted_bare_or_raw`

## Bound amendment: `inline-symbol-path-confinement/a-receiver-method-read-is-a-documented-bound`

Its scenario gains the qualified form, and a second pin with a declared mutation:

- **WHEN** `crate::core` calls `some_instant.elapsed()` on an injected `Instant`, or calls `<std::time::SystemTime>::now()` or `<S as crate::clock::Clock>::now()`
- **THEN** the system does not claim to observe it (no type inference on the receiver or the qualified type) — a stated bound, not a silent assertion of cleanliness
- **PINNED-BY** `inline_receiver_method_read_is_a_bound`
- **PINNED-BY** `inline_qualified_path_is_the_type_directed_bound`

## Bound: a generic parameter named like an import is read as the import

The requirement above records bindings from module bodies and blocks only; a generic parameter's `<…>` list is not read. This SHALL be stated as an over-reaction bound under the capability's observation bounds:

#### Scenario: A generic parameter named like an import is read as the import — a stated bound

- **WHEN** `crate::core` writes `use std::process::Command;` and `pub fn f<Command: Default>() -> Command { Command::default() }` under a boundary forbidding inline calls under `std::process`
- **THEN** the system reports `std::process::Command::default in crate::core`: Rust resolves `Command` to the generic parameter, and the scanner, which does not read generic parameter lists, reads the module's import — an over-reaction declared, not a precision claim
- **PINNED-BY** `inline_generic_parameter_named_like_an_import_is_read_as_the_import`

#### Scenario: An associated item or variant named like an import does not shadow it

- **WHEN** `crate::core` writes `use std::process::Command;` and calls `Command::new("x")` inside a method of `impl S { const Command: u8 = 0; … }`, or beside `pub enum E { Command }`
- **THEN** the system reports `std::process::Command::new in crate::core`: an `impl`, `trait`, `enum`, `struct` or `union` body opens no name scope, so its members bind no bare head
- **PINNED-BY** `inline_associated_item_or_variant_does_not_shadow_an_import`

## Requirement: One use-tree collection feeds all inline path consumers

The system SHALL collect grouped, aliased, public, and glob use-tree facts once and expose them to import reporting, external-import reporting, symbol resolution, and glob-hazard analysis. These consumers SHALL NOT maintain duplicate use-tree walkers.

#### Scenario: Import reports and inline resolution share one collected tree

- **WHEN** a grouped or glob import is read by the package scanner
- **THEN** import reporting, path resolution, and glob-hazard analysis consume the same parsed binding records
- **PINNED-BY** `use_tree_consumers_read_one_collected_scope_graph`
