# Draft delta: inline-symbol-path-confinement

This is a proposed delta only. It is not the active requirement source until the implementation change is accepted and moved into `openspec/specs/inline-symbol-path-confinement/spec.md` with its reaction.

## Requirement: Inline path heads resolve from one typed lexical scope

For each recognized inline path occurrence, the system SHALL resolve the first identifier from the occurrence's lexical module scope using the package edition, namespace when observable, direct item declarations, named imports and aliases, `pub use` closure, glob-import edges, and external-prelude roots. A glob SHALL carry names visible from its resolved module, including a private import visible to a descendant scope through `use super::*`. The system SHALL distinguish a local binding from an external-prelude binding and SHALL NOT select arbitrarily among distinct competing bindings. An unresolved or ambiguous binding the scanner cannot judge SHALL be refused with exit 2 and a reason naming the scope and candidates.

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

#### Scenario: An inline type alias resolves at its inline path

- **WHEN** an inline child defines a type alias to `std::process::Command` and an occurrence calls the alias's `new`
- **THEN** the system resolves the alias to `std::process::Command` and reports the call
- **PINNED-BY** `an_inline_module_type_alias_resolves_under_its_inline_path`

#### Scenario: A local public re-export resolves across modules

- **WHEN** `crate::support` declares `pub use std::time::SystemTime`, `crate::core` imports that name, and calls `SystemTime::now()`
- **THEN** the system resolves the public re-export closure to `std::time::SystemTime` and reports the call
- **PINNED-BY** `inline_resolves_a_cross_module_local_reexport`

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

#### Scenario: Boundary prefix heads follow their edition category

- **WHEN** a boundary uses a prefix head that is strict/reserved in its edition, weak in that edition, or `_`
- **THEN** the system consults the named edition table: it accepts valid weak/ordinary heads, requires raw spelling for a strict/reserved head when allowed, suggests `r#head`, and refuses `_` without a raw suggestion
- **PINNED-BY** `a_prefix_starting_with_a_keyword_is_refused`
- **PINNED-BY** `inline_prefix_keyword_categories_follow_the_package_edition`

## Requirement: One use-tree collection feeds all inline path consumers

The system SHALL collect grouped, aliased, public, and glob use-tree facts once and expose them to import reporting, external-import reporting, symbol resolution, and glob-hazard analysis. These consumers SHALL NOT maintain duplicate use-tree walkers.

#### Scenario: Import reports and inline resolution share one collected tree

- **WHEN** a grouped or glob import is read by the package scanner
- **THEN** import reporting, path resolution, and glob-hazard analysis consume the same parsed binding records
- **PINNED-BY** `use_tree_consumers_read_one_collected_scope_graph`
