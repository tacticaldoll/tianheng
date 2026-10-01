# module-boundary Specification

## Purpose

Govern the intra-crate module import graph that Cargo cannot see — the
differentiated value over `cargo tree` / `cargo-deny`. A module boundary forbids
one module from importing another ("the kernel must not import a projection"),
observed from the target crate's source `use` declarations (use-only, file-based;
see the scanner decision in `PROJECT.md`). Module violations flow through severity
and the baseline exactly like crate violations.

## Subject

- `crates/guibiao/src/module_scan/**/*.rs`
- `crates/guibiao/src/tests/module_boundary.rs`
- `crates/guibiao/src/tests/module_path.rs`

## Requirements
### Requirement: Module boundary declared in Rust

A module boundary SHALL be declared in Rust, targeting a crate and a module path within it and forbidding an import of another module path. It SHALL be declared as `ModuleBoundary::in_crate("app").module("crate::kernel").must_not_import("crate::projection").because("…")`, and SHALL carry a severity (default enforce, `warn` available) like a crate boundary. The umbrella `Boundary` SHALL accept both crate and module boundaries.

#### Scenario: Module boundary holds its target, module, and forbidden import

- **WHEN** a developer declares `ModuleBoundary::in_crate("app").module("crate::kernel").must_not_import("crate::projection").because("…")`
- **THEN** the constitution holds a module boundary on crate `app`, governing module `crate::kernel`, forbidding imports of `crate::projection`, with a non-empty reason

### Requirement: Module imports observed from source use declarations

The system SHALL observe module imports by scanning the target crate's source `use` declarations; a `use` written inside an attribute is that attribute's input and SHALL NOT be read as one, and a use tree holding a token no path segment is, or a path ending in `::`, SHALL be refused (exit 2) rather than read with that leaf dropped. It SHALL resolve `crate`, `self`, and `super` paths to absolute `crate::…` module paths, expand grouped (`{a, b}`) and glob (`::*`) forms, and ignore paths whose first segment is an external crate. In edition 2018 and later a `use` path is a uniform path: a bare first segment SHALL be read as what the scope the `use` stands in binds or declares under that name — a `mod`, an item, another import, or a name a glob brings — and only a head that scope does not bind SHALL be an external crate. The head SHALL be read through the same resolver the inline confinements read, over every file of the compilation unit, and only the head: a path is internal as its head names it, and a re-export the rest of the path runs through is not followed, since an import of `crate::support::X` imports `crate::support` whatever `X` re-exports, and a head a glob brings is bound as the glob's module followed by the head, whatever that module binds it as. So at the crate root a sibling `mod foo` makes a bare `use foo::…` the local module, and in a submodule a bare head reaches that submodule's own children and imports but not the crate root's modules. A module name enters a scope by its **declaration** — a `mod name;` or `mod name { … }` — not by the mere existence of a like-named source file: an undeclared orphan source file (e.g. a stray `src/foo.rs` that no `mod foo;` declares) is in no scope, and a bare `use foo::…` then resolves through the extern prelude. In edition 2018 and later, a path written with a leading `::` (`use ::name::…`) is the explicit external/global form and SHALL be treated as external even when its first segment matches a crate-root module; in edition 2015 a `use` path, with or without the `::`, is read from the crate root. Text inside comments and string literals SHALL NOT be treated as a `use` (or `mod`) declaration: a comment is dropped and a literal is one token when the file is read, so neither a `//` inside a string nor a `use …;` written inside a string affects the result. Bare path expressions and macro-generated imports SHALL be out of scope (see the scanner decision in `PROJECT.md`); the rule enforces only what real `use` declarations observe. In particular, a `use` written inside a macro body — a `macro_rules!` definition OR a macro invocation (`ident! {…}` / `(…)` / `[…]`) — is a macro-generated import: the `macro_rules!` definition (its name and balanced body) and any macro invocation's balanced `{}`/`()`/`[]` body are one node whose group no `use` enumeration reads, so such a `use` SHALL NOT be observed. A `use` token that is **not an import statement** — specifically a **precise-capturing bound** (`-> impl Trait + use<'a, T>`, stable Rust), where the `use` token is immediately followed (after optional whitespace) by `<` — SHALL NOT be treated as an import and SHALL NOT consume a following real `use` declaration; a `use` *statement* is always followed by a path (an identifier, `{`, `*`, `::`, or `crate`/`self`/`super`), never `<`, so the following-token `<` is the discriminator, and skipping the bound keeps the next real `use` observable (never a silent drop). Comments SHALL be dropped, and string literals and char literals — normal, byte, and raw string forms, and a char literal's full scalar value regardless of its UTF-8 byte length — SHALL each be read as one token, so that a character a char literal contains (including `{` or `}`) is never mistaken for a real structural brace by the reachability walk. Modules SHALL be file-based **and reachable from the crate root via `mod` declarations**: a source file that no `mod` declaration brings into scope — an undeclared orphan, at the crate root or anywhere in a subtree — is not a module of the crate, is not governed, and its imports SHALL NOT be observed, matching the compiler (which never compiles it). A governed module path that matches no reachable source file SHALL be a constitution error (exit 2), never a silent pass. A governed source file that exists but cannot be read SHALL likewise be a scan error (exit 2), never silently skipped — an unreadable file is "cannot judge", not "nothing to judge", and skipping it could hide a real violation. A governed source directory that cannot be traversed SHALL likewise be a scan error (exit 2), naming the directory, never silently skipped — the same "cannot judge, not nothing to judge" rule, because a skipped subtree could hide a real violation. A file-form `mod` a block declares SHALL be followed through its direct or `cfg_attr` path attribute, since it has no conventional file, and one whose every path names no file SHALL be a scan error (exit 2) naming the file declaring it. An inline `mod` a block declares SHALL be read as a module, its children's path attributes resolving from the declaring file's directory joined with the inline modules' names, and every file-form `mod` inside it SHALL be held to the same rule as one written in the block itself, unless a path attribute on that inline module, or on one enclosing it within the block, gives it a directory of its own, as rustc reads it, where a file-form `mod` is read from that directory as anywhere else; where that attribute is a `cfg_attr` path, the directories it names SHALL be the only bases, since without it rustc accepts no file-form `mod` there. A `#[path]` SHALL resolve from the directory of the path the declaring file was opened by, so a file reached through a symlink resolves its paths beside the symlink rather than beside its target. A crate root's tokens SHALL start past a UTF-8 byte-order mark and past a shebang line, which a file opening with `#!` holds unless the next token past whitespace and comments — nested block comments included — is a `[`.

#### Scenario: A cfg_attr path module a block declares is governed
- **WHEN** a crate root writes `fn f() -> u32 { #[cfg_attr(unix, path = "x.rs")] mod m; m::h() }` and `x.rs` calls `std::process::id()`; or the same with no `x.rs`
- **THEN** the system reports `std::process::id in crate::{block}::m` for the first, and a scan error (exit 2) for the second: a block's file-form `mod` names no conventional file, so a `cfg_attr` path no file backs leaves no configuration that builds it
- **PINNED-BY** `a_cfg_attr_path_module_a_block_declares_is_governed`

#### Scenario: A path in a symlinked file resolves from the symlink's directory
- **WHEN** `src/a.rs` is a symlink to `../elsewhere/a.rs`, which holds `#[path = "x.rs"] pub mod k;`, and `src/x.rs` exists
- **THEN** the system governs `src/x.rs` as `crate::a::k`: rustc 1.96.0 resolves the `#[path]` from the directory of the path it opened `a.rs` by, and refuses the crate when `x.rs` lies beside the symlink's target instead
- **PINNED-BY** `a_path_in_a_symlinked_file_resolves_from_the_symlinks_directory`

#### Scenario: A path module a block declares is governed
- **WHEN** a crate root writes `fn f() -> u32 { #[path = "x.rs"] mod m; m::h() }` and `fn g() -> u32 { #[path = "y.rs"] mod m; m::h() }`, `x.rs` calls `std::process::id()`, or instead holds `pub use std::process::id as h;`
- **THEN** the system governs `x.rs` as `crate::{block}::m` and `y.rs` as `crate::{block 2}::m`, numbered among the modules of that name the blocks of `crate` declare in source order, reports `std::process::id in crate::{block}::m` for the call in `x.rs`, and `std::process::id in crate` for `m::h()` through the re-export: rustc compiles a file-form `mod` a block declares with a `#[path]`, and a path through it from its block reads that file's scope
- **PINNED-BY** `a_path_module_a_block_declares_is_governed`
- **PINNED-BY** `block_declared_path_modules_are_numbered_apart`

#### Scenario: An inline module a block declares is read
- **WHEN** a crate root writes `pub fn f() { mod k { #[path = "y.rs"] pub mod m; } k::m::s(); }` and `src/k/y.rs` calls `std::process::id()`; or `src/a.rs` writes the same with `k` holding `pub mod j { #[path = "y.rs"] pub mod m; }` and `src/k/j/y.rs` makes the call; or a crate root writes `pub fn f() { mod k { pub mod m; } }`
- **THEN** the system reports the call in `src/k/y.rs` and in `src/k/j/y.rs`, and a scan error naming `src/lib.rs` for the third: rustc 1.96.0, edition 2021, reads an inline module a block declares from the declaring file's directory — `src/k`, not `src/a/k`, from `a.rs` — and refuses a file-form `mod` with no path attribute anywhere inside a block
- **PINNED-BY** `an_inline_module_a_block_declares_is_read`

#### Scenario: A crate root after a shebang line declares its modules
- **WHEN** a crate root opens with `#!/usr/bin/env run` above `pub mod core;` and `pub mod x;`, `src/core.rs` imports `crate::x::Y`, and a boundary forbids `crate::core` to import `crate::x`
- **THEN** the system reports `crate::x::Y`: rustc strips a shebang line, which a file opening with `#!` holds unless the next token past whitespace and comments is a `[`
- **PINNED-BY** `a_crate_root_after_a_shebang_line_declares_its_modules`

#### Scenario: A crate root after a byte-order mark or a commented inner attribute declares its modules
- **WHEN** a crate root opens with a UTF-8 byte-order mark before `pub mod m;`, or writes `#! /* /* n */ */ [allow(dead_code)] pub mod m;`, and `src/m.rs` calls `std::process::id()`, under a boundary confining `std::process` over `crate::m`
- **THEN** the system reports `std::process::id in crate::m` for each: rustc strips the mark before anything else, and a block comment between `#!` and `[` nests, so that line is an inner attribute and no shebang
- **PINNED-BY** `a_crate_root_after_a_byte_order_mark_or_a_commented_inner_attribute_declares_its_modules`

#### Scenario: A use tree holding what no path is is refused
- **WHEN** a module writes `use crate::forbidden::{1};` or `use crate::forbidden::;`
- **THEN** the system refuses the file (exit 2), quoting the token or the trailing `::`: rustc refuses both, and a leaf dropped is an import no rule sees
- **PINNED-BY** `a_use_tree_holding_what_no_path_is_is_refused`

#### Scenario: A use inside an attribute is no import
- **WHEN** `src/m.rs` writes `#[cfg_attr(any(), my_attr(use crate::forbidden::Thing;))] pub fn f() {}`, or `#[allow(unused_imports)] use crate::forbidden::Thing;`, under a boundary forbidding `crate::m` to import `crate::forbidden`
- **THEN** the system reports nothing for the first and `crate::forbidden::Thing` for the second: rustc 1.96.0, edition 2021, compiles the first importing nothing, and an attribute written on a `use` leaves the `use` read
- **PINNED-BY** `a_use_inside_an_attribute_is_no_import`

#### Scenario: A use aliased to underscore is an import
- **WHEN** `crate::core` declares `use crate::forbidden as _;`, or `use crate::forbidden as f;`, under a boundary forbidding `crate::core` to import `crate::forbidden`
- **THEN** the system reports `crate::forbidden` for each: an `as _` alias binds no name and imports its path like any other
- **PINNED-BY** `a_use_aliased_to_underscore_is_an_import`

#### Scenario: A grouped use of crate paths is observed

- **WHEN** a file in the governed module declares `use crate::projection::{A, B};`
- **THEN** both `crate::projection::A` and `crate::projection::B` are observed as imports of `crate::projection`

#### Scenario: A root-relative bare use of a declared local module is observed

- **WHEN** a file at the crate root declares `use kernel::Thing;` and the crate root declares `mod kernel;` (so `kernel` is a crate-root module of the target crate)
- **THEN** the system observes the import `crate::kernel::Thing`, rather than dropping it as an external crate

#### Scenario: An undeclared orphan source file does not create a crate-root module

- **WHEN** a file at the crate root declares `use serde::Deserialize;`, `serde` is an external crate, and a source file `src/serde.rs` exists that no `mod serde;` declaration brings into scope
- **THEN** the system treats the import as external and does NOT observe `crate::serde::Deserialize`, because an undeclared orphan file is not a crate-root module

#### Scenario: An undeclared orphan submodule file is not governed

- **WHEN** a crate declares `mod kernel;`, the file `src/kernel/orphan.rs` exists that `kernel` never declares with `mod orphan;`, that orphan file contains `use crate::projection::Thing;`, and a boundary governs `crate::kernel` forbidding `crate::projection`
- **THEN** the system reports no violation, because only files reachable from the crate root via `mod` declarations are modules of the crate — the orphan file is not compiled, is not governed, and its import is not observed

#### Scenario: A file-backed child reached only through an inline parent is governed

- **WHEN** a crate-root file declares `mod parent { mod child; }` (inline, with no file of its own), the file `src/parent/child.rs` exists and contains `use crate::projection::Thing;`, and a boundary governs `crate::parent::child` forbidding `crate::projection`
- **THEN** the system reports the violation, because `crate::parent::child` is reachable — declared inside `parent`'s own inline body, which the walk re-scans for its nested `mod` declarations, not only the crate root's own top level

#### Scenario: A plain child reached only through a symlinked directory is still governed

- **WHEN** a crate declares `mod parent;`, `parent.rs` declares a plain `mod child;`, `parent/child.rs` does not physically exist but `parent` itself is a symlink to a real directory elsewhere containing `child.rs` (a forbidden import), and a boundary governs `crate::parent` forbidding the import
- **THEN** the system reports the violation, attributed to the real file behind the symlink — a symlinked directory component along a module's resolved path does not make its content invisible to governance, even though the crate-wide file walk itself does not recurse into a symlinked directory (a separate, cycle-safety concern)

#### Scenario: A symlink-aliased module is governed under its own path, not dropped for matching another's file

- **WHEN** a crate declares `mod real;` (backed directly by `src/real/mod.rs`, containing a forbidden import) and a separate `mod kernel;`, where `src/kernel` is a symlink to `src/real` (so `crate::kernel` and `crate::real` are two distinct, separately-declared modules that happen to resolve to the identical physical file), and a boundary governs `crate::kernel` forbidding the same import
- **THEN** the system reports the violation for `crate::kernel` — the aliasing with `crate::real`'s own file does not make `crate::kernel` invisible; two on-disk paths resolving to the same physical content are never treated as the same module merely because their canonical (symlink-resolved) identity coincides

#### Scenario: A bare use in a submodule is external even when it matches a crate-root module

- **WHEN** a file in a submodule (not the crate root) declares `use serde::Deserialize;` and `serde` is also a crate-root module of the target crate
- **THEN** the system treats the import as external and does not observe `crate::serde::Deserialize`, because a submodule's bare first segment names what its own scope binds, and the crate root's modules are not in it

#### Scenario: An import through a re-export imports the module it names
- **WHEN** `crate::support` declares `pub use crate::forbidden::x;`, `crate::core` declares `use crate::support::x;`, and a boundary restricts `crate::core`'s imports to `crate::support`; or `crate::support` declares `pub use crate::forbidden as fmod;` and `crate::core` declares `use crate::support::*;` and `use fmod::x;`
- **THEN** the system reports nothing for either: the import names `crate::support` — for the second, `crate::support::fmod::x`, the head the glob brings bound as the glob's module followed by the head — and the re-export its path runs through is not followed
- **PINNED-BY** `an_import_through_a_re_export_imports_the_module_it_names`

#### Scenario: A name bound in one namespace and brought by a glob in the other imports both
- **WHEN** `crate::sub` declares `pub type x = crate::m::T;`, `pub use crate::g::*;` for `crate::g` holding `pub fn x() {}`, and `pub use x as y;`, and a boundary restricts `crate::sub`'s imports to `crate::g`
- **THEN** the system reports `crate::m::T`: the import names the alias in the type namespace beside the glob-brought function in the value namespace, and both are read
- **PINNED-BY** `a_name_bound_in_one_namespace_and_brought_in_the_other_imports_both`

#### Scenario: A bare use in a submodule names that submodule's own child
- **WHEN** `crate::a` declares `pub mod inner { pub struct X; }` and `use inner::X;`, under a boundary forbidding `crate::a` to import `crate::a::inner`
- **THEN** the system reports `crate::a::inner::X`: a uniform path's head names what the scope it stands in declares
- **PINNED-BY** `a_use_heads_its_path_from_the_scope_it_stands_in`

#### Scenario: A leading-colon path is external even when its head is a crate-root module

- **WHEN** a file declares `use ::serde::Deserialize;` and `serde` is also a crate-root module of the target crate
- **THEN** the system treats the import as external and does not observe `crate::serde::Deserialize`, because the leading `::` is the explicit external/global form

#### Scenario: An external import is ignored

- **WHEN** a file declares `use serde::Deserialize;` and `serde` is not a crate-root module of the target crate
- **THEN** the system does not treat it as an internal module import

#### Scenario: A use written inside a string literal is not observed

- **WHEN** a file contains a string literal whose text is `use crate::projection::Thing;`, and no real `use` of that path
- **THEN** the system does not observe an import of `crate::projection`

#### Scenario: A use written inside a macro_rules body is not observed

- **WHEN** a file declares `macro_rules! m { () => { use crate::projection::Thing; }; }` and no real `use` of that path outside the macro
- **THEN** the system does not observe an import of `crate::projection`, because the `macro_rules!` body is a macro-generated import and is not read for `use` statements

#### Scenario: A use written inside a macro invocation body is not observed

- **WHEN** a file declares `some_macro! { use crate::projection::Thing; }` and no real `use` of that path outside the macro
- **THEN** the system does not observe an import of `crate::projection`, because a macro invocation body is a macro-generated import and is not read for `use` statements

#### Scenario: A precise-capturing use bound is not an import and does not swallow the next use

- **WHEN** a file declares `fn iter() -> impl Iterator<Item = u8> + use<> { … }` (a precise-capturing bound) immediately followed by a real `use crate::projection::Thing;`
- **THEN** the system does not treat the `use<>` bound as an import and still observes `crate::projection::Thing`, because the bound (a `use` followed by `<`) is skipped rather than consumed to the next `;`

#### Scenario: A string containing `//` does not hide a real use

- **WHEN** a file declares a string literal containing `//` followed, later on the same line, by a real `use crate::projection::Thing;`
- **THEN** the system observes the import `crate::projection::Thing`

#### Scenario: A constitution error emits exit 2
- **WHEN** a module boundary targets a module path that matches no reachable file
- **THEN** the runner exits with status 2 and names the unknown module

#### Scenario: A non-ASCII char literal adjacent to a brace literal does not leak a structural brace

- **WHEN** a source file contains a non-ASCII char literal immediately adjacent to a `'{'` or `'}'` char literal (e.g. `['«','{']`, no separating space)
- **THEN** neither literal's payload is mistaken for a real structural brace, and every `mod` declared after it remains reachable and governed exactly as if the literals were not present

### Requirement: A transparent control-flow macro's body is read as items

The system SHALL recognize transparent control-flow macros (specifically `cfg_if!`) and SHALL read their inner structural body contents. Enclosed `use` import declarations, `mod` module declarations, and inline symbol call paths inside `cfg_if!` macro bodies SHALL be observed by `use_scan`, `reachability`, and `symbol_scan` as real items, matching the system's cfg-blind union-scanning policy. Other code-generating or declarative macro bodies (`macro_rules!` definitions and non-transparent macro invocations) SHALL be passed over as macro-generated items.

#### Scenario: A use declaration inside a cfg_if macro body is observed

- **WHEN** a governed file contains a `cfg_if!` macro invocation containing `use crate::projection::Thing;` and a boundary forbids `crate::projection`
- **THEN** the system observes `crate::projection::Thing` inside the `cfg_if!` body and emits an enforced violation (exit 1), rather than passing over the import as macro-generated

#### Scenario: An inline mod declaration inside a cfg_if macro body is reachable

- **WHEN** a governed file contains a `cfg_if!` macro invocation declaring `mod child;` and `child.rs` exists containing a forbidden import
- **THEN** the system reaches `child.rs` through the `cfg_if!` declaration and reports the forbidden import violation

### Requirement: Ancestor glob import fail-closed hazard detection

The system SHALL detect when an observed glob import's base path (`crate::a`) is an ancestor of a forbidden target path (`crate::a::b`) under a `must_not_import` module boundary. When an observed glob import base path is equal to or an ancestor of the forbidden target path (`path_within(forbidden_target, glob_base)` is true), the system SHALL treat the wildcard import as a Glob Hazard violation and emit an enforced violation (exit 1), preventing bypass of module boundaries via wildcard ancestor imports.

#### Scenario: An ancestor glob import of a forbidden module is a violation

- **WHEN** a file in a governed module declares `use crate::a::*;` and a boundary governs the module forbidding `crate::a::b`
- **THEN** the system emits an enforced Glob Hazard violation (exit 1), because `crate::a` is an ancestor of the forbidden path `crate::a::b`

#### Scenario: A glob import of an unrelated module is not a glob hazard

- **WHEN** a file in a governed module declares `use crate::c::*;` and a boundary governs the module forbidding `crate::a::b`
- **THEN** the system reports no violation for that glob import, because `crate::c` is not an ancestor of `crate::a::b`

#### Scenario: A plain non-glob ancestor import of a forbidden module is clean

- **WHEN** a file in a governed module declares `use crate::a;` (non-glob) and a boundary governs the module forbidding `crate::a::b`
- **THEN** the system reports no violation for that import, because `use crate::a;` does not bring `crate::a::b` into scope

### Requirement: Mid-path relative segment normalization

The system SHALL normalize embedded `self` and `super` segments appearing anywhere in an observed module import or symbol path (e.g. `crate::a::b::super::c` -> `crate::a::c`, `crate::a::self::b` -> `crate::a::b`). Over-popping `super` segments past the `crate` root SHALL resolve to `None` (an invalid path) and SHALL NOT produce a false-positive or false-negative boundary finding.

#### Scenario: A mid-path super import of a forbidden module is observed

- **WHEN** a governed module declares `use crate::a::b::{super::forbidden::Thing};` and a boundary forbids `crate::a::forbidden`
- **THEN** the system normalizes the import path to `crate::a::forbidden::Thing` and emits an enforced violation (exit 1)

#### Scenario: A mid-path self import is normalized to its parent module

- **WHEN** a governed module declares `use crate::a::{self::b::Thing};` and a boundary governs `crate::a::b`
- **THEN** the system normalizes the import path to `crate::a::b::Thing` and evaluates boundary rules against the canonical path

#### Scenario: An unreadable governed source file is a scan error

- **WHEN** a governed module resolves to a source file that exists but cannot be read
- **THEN** the system reports a scan error naming the file and exits 2, rather than skipping the file

#### Scenario: An unreadable governed source directory is a scan error

- **WHEN** a governed module's source subtree contains a directory that cannot be traversed
- **THEN** the system reports a scan error naming the directory and exits 2, rather than skipping the subtree

### Requirement: Forbidden module import is a violation

The system SHALL emit a violation when a file in the governed module imports the forbidden module or any module beneath it. The violation SHALL name the governed module as its target and the offending import path as its finding, and SHALL react according to its severity (enforce fails, warn is advisory) and any baseline, exactly as a crate violation does.

#### Scenario: Kernel importing projection violates

- **WHEN** a file in `crate::kernel` declares `use crate::projection::Thing;` and the boundary forbids importing `crate::projection`
- **THEN** the system emits a violation naming `crate::kernel` and the import `crate::projection::Thing`, and exits 1 at enforce severity

#### Scenario: The allowed direction is clean

- **WHEN** the boundary forbids `crate::kernel` from importing `crate::projection`, and only `crate::projection` imports `crate::kernel`
- **THEN** the system reports no violation for that boundary

### Requirement: Module imports restricted to a closed allowlist

A module boundary SHALL support a closed-allowlist rule restricting which internal modules the governed module may import: `ModuleBoundary::in_crate(p).module(m).restrict_imports_to([...]).because(...)`. Any internal `use` from the governed module to a module that is neither within the governed module's own subtree (`m` or beneath, i.e. `m` or a path beginning `m::`) nor within an allowlist entry (an entry or beneath, i.e. the entry or a path beginning `entry::`) SHALL be a violation; an empty allowlist forbids every outward internal import (only the module's own subtree is permitted). The "or beneath" test SHALL be `::`-delimited, so an allowlist entry `crate::types` does not cover a sibling `crate::types_extra`. Relative imports (`self::`/`super::`) SHALL be resolved to absolute paths before the check. External imports SHALL remain out of scope. The rule SHALL carry severity (default enforce, `warn` available) and flow through the baseline exactly as `must_not_import`. Like the crate-level restrict-to, its JSON projection SHALL use the key `only`. Declaring the rule on `crate` itself SHALL be a constitution error (exit 2), self-describing and distinct from a violation, because the crate root has no outward internal edge to observe.

#### Scenario: An internal import outside the allowlist violates

- **WHEN** the governed module `crate::kernel` declares `use crate::io::Sink;` and the boundary is `restrict_imports_to(["crate::types"])`
- **THEN** the system emits a violation naming `crate::kernel` and the import `crate::io::Sink`, and exits 1 at enforce severity

#### Scenario: An allowlisted import is clean

- **WHEN** `crate::kernel` imports only `crate::types::Id` and the boundary is `restrict_imports_to(["crate::types"])`
- **THEN** the system reports no violation for that boundary

#### Scenario: The governed module's own subtree is allowed without listing

- **WHEN** `crate::kernel` declares `use crate::kernel::detail::Thing;` and the boundary is `restrict_imports_to(["crate::types"])`
- **THEN** the system reports no violation, because a module importing its own subtree is not an outward edge

#### Scenario: An empty allowlist forbids every outward internal import

- **WHEN** `crate::kernel` imports `crate::types::Id` and the boundary is `restrict_imports_to([])`
- **THEN** the system emits a violation for `crate::types::Id`, because the empty allowlist permits only the module's own subtree

#### Scenario: A prefix-colliding sibling of an allowlist entry violates

- **WHEN** `crate::kernel` declares `use crate::types_extra::Y;` and the boundary is `restrict_imports_to(["crate::types"])`
- **THEN** the system emits a violation, because the "or beneath" test is `::`-delimited: `crate::types_extra` is neither `crate::types` nor beneath `crate::types::`

#### Scenario: An external import is never flagged, even under an empty allowlist

- **WHEN** `crate::kernel` declares `use serde::Deserialize;` and the boundary is `restrict_imports_to([])`
- **THEN** the system reports no violation, because external imports are out of scope (only internal `crate::…` edges are observed)

#### Scenario: A `super::`-reaching-outward import is governed

- **WHEN** `crate::kernel::inner` declares `use super::super::other::Thing;` (resolving to `crate::other::Thing`, outside the governed subtree and the allowlist) and the boundary is `restrict_imports_to(["crate::types"])` on `crate::kernel`
- **THEN** the system emits a violation, because relative imports are resolved to absolute paths and an outward edge is governed regardless of how it was written

#### Scenario: Importing the governed module itself or via `self::` is clean

- **WHEN** `crate::kernel` declares `use self::detail::Thing;` (resolving to `crate::kernel::detail::Thing`) and the boundary is `restrict_imports_to(["crate::types"])`
- **THEN** the system reports no violation, because the import is within the governed module's own subtree

#### Scenario: A raw-identifier allowlist entry is canonicalized

- **WHEN** the boundary is `restrict_imports_to(["crate::r#type"])` and `crate::kernel` declares `use crate::type::Thing;`
- **THEN** the system reports no violation, because allowlist entries are canonicalized (`r#type` and `type` are one module) exactly like the governed and forbidden paths

#### Scenario: Declaring the rule on `crate` itself is a constitution error

- **WHEN** a boundary declares `restrict_imports_to([...])` on `crate` (the crate root)
- **THEN** the system emits a self-describing constitution error and exits 2 — naming `crate` and that it has no outward internal edge — distinct from a boundary violation, never a silent pass

### Requirement: A module may forbid being imported by another module

A module boundary SHALL support an inbound rule: `ModuleBoundary::in_crate(p).module(m).must_not_be_imported_by(x).because(...)` declares that the protected module `m` must not be imported by module `x` or anything beneath it. The system SHALL observe this from the crate's source `use` declarations across all reachable files: a file whose enclosing module is `x` or beneath `x` that imports `m` or anything beneath `m` SHALL be a violation, naming `m` as the target and the offending importing module as the finding. The "or beneath" test SHALL be `::`-delimited on both sides, so a forbidden importer `crate::http` does not match a sibling `crate::httpx`, and a protected `crate::internal` does not match a sibling `crate::internal_util`. A file whose enclosing module is `m` or beneath `m` SHALL NOT be treated as an importer — a module importing its own subtree is not an inbound edge — so the rule never flags `m`'s own files even when `x` is an ancestor of `m`. External imports SHALL remain out of scope. The finding SHALL be the importing module path, deduplicated so one offending importer yields one violation per protected target; the rule SHALL carry severity (default enforce, `warn` available) and flow through the baseline like the other module rules. The protected module `m` SHALL be a reachable file-based module, with the same inline/unknown constitution-error handling as other module targets. Declaring the rule with `m` = `crate` (the crate root) SHALL be a constitution error (exit 2), self-describing and distinct from a violation, because every internal import is then "m or beneath" and the rule could never react as an inbound rule.

#### Scenario: An import from the forbidden importer violates

- **WHEN** `crate::internal` is protected by `must_not_be_imported_by("crate::http")` and a file in `crate::http` declares `use crate::internal::Secret;`
- **THEN** the system emits a violation naming `crate::internal` as the target and the offending importer `crate::http`, and exits 1 at enforce severity

#### Scenario: An import from outside the forbidden importer is clean

- **WHEN** the same boundary holds and only `crate::core` (not beneath `crate::http`) imports `crate::internal`
- **THEN** the system reports no violation for that boundary

#### Scenario: The rule applies beneath the importer

- **WHEN** a file in `crate::http::v1` declares `use crate::internal::Secret;` and the boundary is `must_not_be_imported_by("crate::http")`
- **THEN** the system emits a violation naming the importer `crate::http::v1`, because it is beneath the forbidden importer

#### Scenario: The rule applies beneath the protected module

- **WHEN** a file in `crate::http` declares `use crate::internal::deep::Thing;` and the boundary is `must_not_be_imported_by("crate::http")` protecting `crate::internal`
- **THEN** the system emits a violation, because the imported path is beneath the protected module

#### Scenario: A prefix-colliding importer sibling is clean

- **WHEN** a file in `crate::httpx` declares `use crate::internal::Secret;` and the boundary is `must_not_be_imported_by("crate::http")`
- **THEN** the system reports no violation, because `crate::httpx` is neither `crate::http` nor beneath `crate::http::`

#### Scenario: A prefix-colliding protected sibling is clean

- **WHEN** a file in `crate::http` declares `use crate::internal_util::X;` and the boundary protects `crate::internal` via `must_not_be_imported_by("crate::http")`
- **THEN** the system reports no violation, because `crate::internal_util` is neither `crate::internal` nor beneath `crate::internal::`

#### Scenario: The protected module's own subtree is not an importer

- **WHEN** `crate::a::b` is protected by `must_not_be_imported_by("crate::a")` and a file in `crate::a::b` declares `use crate::a::b::detail::Thing;`
- **THEN** the system reports no violation, because a file within the protected module is not an inbound importer even though it is beneath the forbidden importer `crate::a`

#### Scenario: An external import is ignored

- **WHEN** a file in `crate::http` declares `use serde::Deserialize;` and the boundary is `must_not_be_imported_by("crate::http")` protecting `crate::internal`
- **THEN** the system reports no violation, because external imports are out of scope

#### Scenario: Forbidding the crate root as importer forbids every outside importer

- **WHEN** `crate::internal` is protected by `must_not_be_imported_by("crate")` and `crate::http` imports `crate::internal`
- **THEN** the system emits a violation naming `crate::http`, because every module outside `crate::internal`'s own subtree is beneath `crate`; `crate::internal`'s own files remain clean

#### Scenario: Protecting the crate root is a constitution error

- **WHEN** a boundary declares `must_not_be_imported_by(x)` on `crate` (the crate root)
- **THEN** the system emits a self-describing constitution error and exits 2 — distinct from a boundary violation, never a silent pass — because every internal import would match and the rule could never react as an inbound rule

### Requirement: A boundary reports each violation once

A module boundary SHALL report each distinct violation at most once: its violations SHALL be deduplicated by identity `(target, rule_key, fact)`. When the governed module's subtree spans multiple source files that produce the same finding — a parent and a child file importing the same path, or one module backed by two files in one root, as the per-platform shim's `#[cfg]` arms remapping `#[path]` are — the system SHALL emit a single violation, not one per file. Deduplication SHALL be performed per boundary at the point findings are produced, so a duplicate arising from any other source is not silently suppressed.

#### Scenario: A finding produced by two files in the governed subtree is reported once

- **WHEN** the governed module `crate::kernel` spans `kernel.rs` and `kernel/sub.rs`, both declaring `use crate::forbidden::Thing;`, and the boundary forbids `crate::forbidden`
- **THEN** the system emits exactly one violation for `crate::forbidden::Thing`, not two

#### Scenario: Identical findings collapse but distinct findings are kept

- **WHEN** one file in the governed subtree imports `crate::forbidden::A` and another imports both `crate::forbidden::A` and `crate::forbidden::B`, under a boundary forbidding `crate::forbidden`
- **THEN** the system emits exactly two violations — `crate::forbidden::A` (once) and `crate::forbidden::B` — never a duplicate of `A`

### Requirement: Raw identifiers in module paths are canonicalized

The system SHALL treat a raw identifier (`r#name`) and its plain form (`name`) as the same module segment when observing module declarations, module file paths, and `use` paths, and when comparing them against a boundary's declared module and forbidden paths. Because Rust resolves `mod r#name;` to the source file `name.rs`, the file-derived path and the declaration must canonicalize to the same module identity; a boundary MAY be declared with either form and SHALL match the observed module regardless of which form the source uses. A module whose name is a raw identifier SHALL therefore be governable, and a forbidden import written with a raw identifier SHALL be observed.

#### Scenario: A raw-identifier module is governed and its imports observed

- **WHEN** a crate declares `mod r#type;` (resolving to `src/type.rs`), that file imports `use crate::r#mod::Thing;`, and a boundary governs `crate::type` forbidding `crate::mod`
- **THEN** the module `crate::type` is found (not an unknown-module constitution error) and the import `crate::mod::Thing` is observed as a violation, the raw and plain forms having been canonicalized to one identity

**One identity means the recorded one too, not only the matched one.** A rule's key is what a baseline files
a violation under, so a boundary whose declaration is rewritten between the two spellings SHALL keep its key:
matching them while keying them apart makes a pure rename move every recorded finding, which is work an
adopter did not choose and cannot see the reason for. The canonicalization SHALL therefore reach every rule
field carrying a module path — the governed path, an allowlist's entries, and a confined crate name alike.

#### Scenario: A boundary rewritten between the two spellings keeps its recorded identity

- **WHEN** a boundary's declared module, allowlist entry, or confined crate name is rewritten from `r#name`
  to `name`, or the reverse
- **THEN** the rule key is unchanged, so a recorded baseline still describes the tree

### Requirement: A module path has one canonical spelling and names a declared module

Every module path a module boundary carries SHALL be accepted only as `crate`, or `crate::` followed by
`::`-separated identifiers — the spelling `semantic-signature-coupling` states for a 渾儀 module anchor. That
covers the governed module passed to `.module(...)`, whatever rule follows it, and each module a rule names:
the forbidden module of `must_not_import`, the forbidden importer of `must_not_be_imported_by`, and every entry
of `restrict_imports_to([...])` and `must_only_be_imported_by([...])`. A segment SHALL be an identifier as the
Rust Reference reads one: `_` or a Unicode `XID_Start` character, then `XID_Continue` characters, behind at most
one `r#`, and not a raw spelling of `crate`, `self`, `super`, `Self` or `_`. Any other spelling SHALL be a constitution error
(exit 2), judged before the package is read, quoting what was written and naming the canonical spelling where
the text determines one; `r#x` and `x` remain one identifier, as the raw-identifier requirement above states.
A path rooted at `self` or `super` SHALL be refused without a suggestion, because a declaration has no module
for it to be relative to.

A named module SHALL be one that some compiled root of the package declares via `mod`, file-based or inline.
One that no root declares SHALL be a constitution error (exit 2) naming the module and the builder that named
it. A forbidden module or importer that is not declared can never appear in an edge, so without the refusal
the rule is silently inert; an allowlist entry that is not declared can never match, so without it every edge
the entry was written to permit is reported. A package's roots are separate module graphs, so a named module
declared in one root and absent from another SHALL be accepted. The governed module's own existence is the
file-based-target requirement below.

The inline-symbol-path prefix of `must_not_call_inline` and `confine_inline_call`, and the crate name of
`confine_external_crate`, name paths outside the crate's module graph and are not held to this spelling; the
prefix is held to its own, which `inline-symbol-path-confinement` states.

#### Scenario: A forbidden module written without its crate root is a constitution error

- **WHEN** `crate::other` imports `crate::kernel::K` and a boundary on `crate::other` declares `must_not_import("kernel")`, or `must_not_be_imported_by("kernel")` while `crate::kernel` imports `crate::other`
- **THEN** the system emits a constitution error (exit 2) quoting `kernel` and suggesting `crate::kernel`, never exit 0 over the edge it was written to forbid
- **PINNED-BY** `a_forbidden_module_written_without_its_root_is_refused_not_judged_clean`

#### Scenario: Every non-canonical spelling is refused wherever a module path is taken

- **WHEN** the governed module, a forbidden module, a forbidden importer, or any entry of either allowlist is written `crate::kernel::`, `""`, `kernel`, `::crate::kernel`, `crate::::kernel`, `self::kernel`, `super::kernel`, `crate:: kernel`, `crate::kernel `, `crate ::kernel`, `r#crate::kernel` or `crate::kernel::*`
- **THEN** each is a constitution error (exit 2) quoting the written path and carrying its repair: the canonical spelling to write — `crate::kernel`, or `crate` for the empty path — or, where the text determines none, the instruction to start at `crate::`
- **PINNED-BY** `every_module_path_role_refuses_a_non_canonical_spelling`

#### Scenario: A named module no root declares is a constitution error

- **WHEN** `must_not_import`, `must_not_be_imported_by`, `restrict_imports_to` or `must_only_be_imported_by` names `crate::nope`, which the crate does not declare
- **THEN** the system emits a constitution error (exit 2) naming `crate::nope` and the builder, rather than judging a rule no edge can match
- **PINNED-BY** `every_module_path_role_refuses_a_module_that_does_not_exist`

#### Scenario: A named module declared in one root, or inline, is not absent

- **WHEN** a binary root declares `mod tooling;` that the library does not, a boundary forbids `crate::tooling`, and another forbids the inline `crate::kernel::detail` the library declares
- **THEN** both are judged, and each reacts to the import its root holds
- **PINNED-BY** `a_named_module_present_in_one_compilation_unit_or_inline_is_not_absent`

#### Scenario: 圭表 and 渾儀 answer one module path alike

- **WHEN** one table of module-path spellings is given to 渾儀's module anchor and to each 圭表 builder that takes a module path
- **THEN** each row is accepted by both and recorded in one form, or refused by both — for its spelling with one message, or as absent: a no-break space before `kernel`, a zero-width space after it and an emoji segment are refused by both for their spelling, with one message, since both read an identifier by Unicode's `XID_Start` and `XID_Continue`, and `crate::kérnel` is an identifier to both and refused as absent
- **PINNED-BY** `guibiao_and_hunyi_accept_and_record_a_module_path_alike`

### Requirement: Imports are attributed to their enclosing inline module

The system SHALL attribute each `use` declaration to the module that lexically encloses it, including an inline `mod name { … }` submodule, rather than to the containing file's module. A `self`/`super` path SHALL be resolved against that enclosing module, and in edition 2018 and later a bare first segment inside an inline submodule SHALL be read in that submodule's scope, so a crate-root module's name there is external unless the submodule itself binds it, even when the file itself is the crate root, matching how the compiler resolves it. In edition 2015 a `use` path is read from the crate root, so a first segment naming a crate-root module SHALL be read as that module wherever the `use` stands. A block encloses no module of its own, so a `use` in a function body SHALL be attributed to the module the function stands in; a module declared in a block SHALL be named through that block, written `{block}` (`crate::a::{block}::m`), and SHALL NOT be attributed to a same-named module its file declares at module level, which rustc keeps apart from it. Two such modules of one file that would be named alike SHALL be two importers, each after the first in source order numbering its last block, `crate::a::{block 2}::m`, since an importer is part of a finding's identity and one identity for two would let a baseline accepting one hide the other. A `mod name;` declaration with no inline body does not enclose any `use` and SHALL NOT change attribution. Every reader of a file SHALL read it in the edition of the target whose compilation unit it belongs to, which is the target's own `edition` where its `[lib]` or `[[bin]]` table declares one and the package's otherwise, so in edition 2015, where `dyn` is an identifier, `mod dyn { … }` encloses the `use`s written in it. Targets sharing one root in editions read apart — 2015 beside a later one — SHALL be a constitution error (exit 2), since rustc compiles that root once in each; editions read alike, 2018 beside 2021, SHALL be one reading.

#### Scenario: A self import inside an inline submodule resolves against that submodule

- **WHEN** the crate-root file declares `mod inner { use self::leaf::Thing; }`
- **THEN** the import is observed as `crate::inner::leaf::Thing`, not `crate::leaf::Thing`, because it is attributed to the enclosing inline module `crate::inner`

#### Scenario: An import in a block's module is not its same-named file module's
- **WHEN** `crate::a` declares `pub mod m;` and, in one function body, `mod m { use crate::x::Y; }`, in another `mod n { use super::super::x::Y; }` and in a third `mod m { use crate::x::Y; }`, under a boundary forbidding `crate::x` to be imported by `crate::a::m` or by `crate::a`
- **THEN** the system reports nothing under `crate::a::m`, and `crate::a::{block}::m`, `crate::a::{block}::n` and `crate::a::{block 2}::m` under `crate::a`
- **PINNED-BY** `an_import_in_a_block_module_is_not_its_same_named_file_modules`

#### Scenario: An import is attributed in the target's edition
- **WHEN** an edition-2015 crate root declares `mod dyn { use crate::a::X; }` and a boundary forbids `crate::dyn` from importing `crate::a`
- **THEN** the system reports the import, attributed to `crate::dyn`
- **PINNED-BY** `an_import_scan_reads_the_targets_edition`

#### Scenario: The module graph is walked in the target's edition
- **WHEN** an edition-2015 crate root declares `mod dyn;`, `src/dyn.rs` imports `crate::a::X`, and a boundary forbids `crate::dyn` from importing `crate::a`
- **THEN** the system governs `src/dyn.rs` as `crate::dyn` and reports `crate::a::X`
- **PINNED-BY** `the_module_graph_is_walked_in_the_targets_edition`

#### Scenario: A target declaring its own edition is read in it
- **WHEN** an edition-2024 package's `[lib]` declares `edition = "2015"`, `src/sub.rs` writes `use clock::now;` beside a crate-root `pub mod clock;`, and a boundary forbids `crate::sub` from importing `crate::clock`; or a `[[bin]]` in the package's edition shares that library's root
- **THEN** the system reports `crate::clock::now` for the first, and exits 2 naming both editions for the second; a `[lib]` in 2018 and a `[[bin]]` in 2021 sharing one root are judged, in 2018, since the scanner reads their paths alike and the 2018 lexing reads as code all the 2021 one does: `cargo check --all-targets` under rustc 1.96.0 builds that manifest, the binary holding a `main`, which is a measurement of the fixture rather than a step its pin runs
- **PINNED-BY** `a_target_is_read_in_its_own_edition`

#### Scenario: An edition-2015 use path is read from the crate root
- **WHEN** an edition-2015 crate root declares `pub mod kernel;` and `pub mod core;`, `src/core.rs` writes `use kernel::Thing;` or `use ::kernel::Thing;`, and a boundary forbids `crate::core` from importing `crate::kernel`
- **THEN** the system reports `crate::kernel::Thing`, because a 2015 `use` path starts at the crate root rather than at the extern prelude
- **PINNED-BY** `an_edition_2015_use_path_is_read_from_the_crate_root`

### Requirement: Module declarations inside macro bodies are not observed

The system SHALL NOT observe a `mod` declaration written inside a macro body — a `macro_rules!` definition or a macro invocation (`ident! {…}` / `(…)` / `[…]`) — as a real module of the crate, the same out-of-scope rule already applied to a `use` inside a macro body. A macro invocation SHALL be read as one node whose group is passed over when `mod` declarations are scanned, so a `mod` token inside a `()`/`[]`-delimited macro invocation is not mistaken for a crate-root module declaration. A braced macro invocation standing as an item SHALL end what stands before the next item, whatever its path's segments are — an identifier, `crate`, `self`, `super` or `Self` — so the `mod` declaration after it is declared.

#### Scenario: A mod inside a macro invocation is not a declared module

- **WHEN** the crate-root file declares `some_macro!( mod ghost; );` and no real `mod ghost;` outside the macro
- **THEN** the system does not treat `crate::ghost` as a declared, reachable module, so a bare `use ghost::…` elsewhere stays external

#### Scenario: A mod after a braced macro called through a keyword-headed path is declared
- **WHEN** a crate root writes `crate::m!{}`, `self::m!{}` or `#[allow(unused)] crate::m!{}` before `mod q;`, or an inline `mod z { super::m!{} mod q; }`, and a boundary forbids the declared `q` from importing `crate::a`
- **THEN** the system governs `q`'s file and reports its `use crate::a::X;`
- **PINNED-BY** `a_mod_after_a_braced_macro_called_through_a_keyword_headed_path_is_declared`

### Requirement: A governed target is a file-based module

A module boundary's governed target SHALL be a file-based module — one backed by a source file reachable from the crate root via `mod` declarations. An inline module (declared with a body, `mod name { … }`, rather than its own file) is reachable for import attribution but owns no source file, and SHALL NOT be a governable target. When a boundary targets a module path that is reachable but file-less (inline), the system SHALL report a constitution error (exit 2) that is self-describing — naming the inline cause and the file-based-target rule — distinct from the unknown-module error used when the path is not reachable at all (e.g. a typo). Both are constitution errors and exit 2; neither is a silent pass.

A same-named conventional source file (`name.rs` / `name/mod.rs`) that sits beside a module path declared **inline-only** — declared with an inline body `mod name { … }` and NOT also declared plain file-form (`mod name;`) in the same crate — is an orphan: Rust never compiles it as that module, because the inline body is the module. Such an orphan SHALL NOT make the inline target appear file-backed: the system SHALL treat the file as inline-occupied and SHALL NOT scan it in place of the inline body, nor mine it for child `mod` declarations. The inline target therefore remains the self-describing inline constitution error (exit 2), never a silent pass over the orphan and never governance of a file Rust does not compile. A path declared **both** inline and plain file-form — which in valid source arises only under mutually-exclusive `#[cfg]` (a same-scope dual declaration is a compile error) — is NOT inline-only, so the plain file is the governable target for that path (the same-named-orphan rule above does not apply, since the file is genuinely declared, not stray); this does not make the inline body's own declarations invisible, though — the system SHALL still observe the inline body for its own nested `mod` declarations (an inline body and a plain file, or an unconditional `#[path]` remap, of the same name are additive with each other, cfg-blind, never mutually exclusive: the scanner does not evaluate `#[cfg]`, so a real declaration under any one arm must be observed regardless of what the other arms declare).

#### Scenario: An inline module target is a self-describing constitution error

- **WHEN** a crate-root file declares `mod kernel { use crate::projection::Thing; }` and a boundary governs `crate::kernel` forbidding `crate::projection`
- **THEN** the system reports a constitution error (exit 2) explaining that `crate::kernel` is declared inline and owns no source file, so module boundaries — which govern file-based modules — cannot target it, rather than reporting it as an unknown module

#### Scenario: An inline target with a same-named orphan file is still a constitution error

- **WHEN** a crate-root file declares `mod kernel { use crate::secret::Thing; }`, an undeclared same-named file `src/kernel.rs` also exists (which Rust does not compile, the inline body being the module), and a boundary governs `crate::kernel` forbidding `crate::secret`
- **THEN** the system reports the inline-module constitution error (exit 2), rather than treating `src/kernel.rs` as the module's backing file, scanning that orphan in place of the inline body, or silently passing — the orphan does not make the inline target file-backed

#### Scenario: An inline body declared alongside a plain-file sibling is still observed for its own children

- **WHEN** a crate-root file declares `mod x;` under one `#[cfg]` arm (backed by a clean conventional `x.rs`) and `mod x { mod y; }` under a mutually-exclusive `#[cfg]` arm, `src/x/y.rs` exists and contains `use crate::secret::Thing;`, and a boundary governs `crate::x::y` forbidding `crate::secret`
- **THEN** the system reports the violation — the plain file backs `crate::x` for governance purposes (it is genuinely declared, not stray), but the inline arm's own nested `mod y;` is still observed, so `crate::x::y` is reachable and governable; the scanner does not silently drop it merely because a plain-file sibling of `crate::x` also exists

#### Scenario: An orphan beside an inline module contributes no phantom child module

- **WHEN** a crate-root file declares `mod kernel { … }` inline, an undeclared `src/kernel.rs` exists declaring `mod deep;`, and a boundary governs `crate::kernel::deep`
- **THEN** the system reports the module as not found (exit 2), because the orphan `src/kernel.rs` is not compiled as `crate::kernel` and its `mod deep;` therefore declares no reachable module — never a silent pass over a phantom child

#### Scenario: A genuinely unknown module is still reported as not found

- **WHEN** a boundary governs a module path that is not reachable in the crate at all (e.g. a typo)
- **THEN** the system reports a constitution error (exit 2) that the module was not found among the crate's reachable modules

### Requirement: A plain module declaration resolves to exactly one conventional file

A plain `mod name;` declaration SHALL resolve to exactly one conventional source file — `name.rs` or `name/mod.rs` — and the system SHALL react rather than guess in every other outcome, never silently dropping the module from the reachable set (which would hide every import beneath it, the false negative the core contract forbids). When **both** forms are present the system SHALL report a constitution error (exit 2) naming both resolved paths and the exactly-one-file rule, regardless of any `cfg_attr(path)` candidate also present on the same declaration — the ambiguity test SHALL precede the absent-file tolerance below and SHALL NOT be overridden by it, so a declaration whose predicate is off — which rustc strips before module resolution, leaving a crate that compiles cleanly and raises no E0761 — is still a constitution error: the scanner is cfg-blind and cannot know which arm is live, and treating one arm's ambiguity as resolvable would require evaluating `cfg`. When **neither** form is present the system SHALL report a constitution error (exit 2) naming both expected paths, EXCEPT when the declaration is **cfg-conditional**, in which case the module may legitimately have no file in the current configuration and SHALL be skipped rather than errored. A declaration SHALL be cfg-conditional from any of three sources, which the system SHALL treat identically because they express one intent — "this declaration may legitimately have no conventional file in the active configuration": a `#[cfg(...)]` attribute, bare or applied through `cfg_attr`, preceding the item, preceding an item, statement, match arm or field that encloses it in its file — read back from the enclosing group to the previous `;`, `,`, brace group or attribute, so a `cfg` standing before one of those within its own statement is not read, a declared bound — or preceding a `mod` declaration the file is reached through — rustc loads nothing beneath what a `cfg` removes; membership in a transparent control-flow macro arm (a `mod` written anywhere inside a `cfg_if!` arm, whose predicate lives in the macro's `if #[cfg(..)]` header rather than on the item — every such arm is conditionally compiled by construction, the trailing `else` on its predicate's negation); or the declaration carrying one or more `cfg_attr(..., path = "…")` remap attributes of which **at least one candidate physically resolves to a real file on disk**. A `#[cfg_attr(...)]` wrapper that carries no `path` meta at all, or whose every `path` remap candidate is absent from disk, SHALL NOT make a declaration cfg-conditional on that basis alone: `cfg_attr` removes the item only where the attribute it applies is a `cfg`, and otherwise only conditionally applies its wrapped attribute, so with no resolved candidate to back it a missing conventional file beneath it is a genuine compile error (E0583) in every configuration — the same blind, existence-only test the plain-file check itself already uses, extended to a resolved conditional remap target, never a predicate-exhaustiveness proof the scanner cannot perform. The same cfg-conditional test SHALL govern an absent `#[path]` remap target, so the two absence outcomes cannot drift apart. Either constitution error SHALL abort the whole reachability walk rather than excluding one module, since a crate whose module graph cannot be resolved cannot be judged, and SHALL name the crate and the compilation unit whose walk refused and the file whose declaration it refuses — for a missing file, every declaring source that found neither form, with the paths each expected — so a package of several roots, or a module several sources declare, says which line to repair. This is the static dimension's own independently-implemented policy for these outcomes; the runtime dimension states the same rules for its own probe-coverage walker (三儀 ⊥ 三儀: the same rule, not the same function), except that it reads a `cfg` from the declaration and its arm alone, and not from what encloses the declaration or the declaration its file is reached through, which `BACKLOG.md` tracks.

**An attribute is the name it spells, and the built-in is the single-segment path.** A raw-identifier spelling — `r#path`, `r#cfg`, `r#cfg_attr`, at the attribute's own name position or inside a `cfg_attr`'s argument list — SHALL be read as the built-in it names, because `r#` changes an identifier's lexical spelling and not the name it spells and none of those three is a keyword. A **keyword** is the opposite case and SHALL NOT be folded into this rule: `r#mut` is an identifier named `mut` and is precisely not the keyword, so a reader matching Rust keywords compares as written. Conversely a segment reached through `::` is somebody else's attribute and SHALL carry no module target, and that narrowing governs the applied `path` meta as much as the `cfg_attr` wrapping it.

#### Scenario: A module backed by both conventional forms is a constitution error

- **WHEN** a crate declares a plain `mod child;` and both `src/child.rs` and `src/child/mod.rs` exist
- **THEN** the system reports a constitution error (exit 2) naming both resolved paths and the exactly-one-file rule, rather than accepting either form as the module's source or treating the two as separate sources of one module path

#### Scenario: A cfg-gated dual-backed declaration is still an ambiguity, though the crate compiles

- **WHEN** the dual-backed `mod child;` declaration carries a bare `#[cfg(...)]` gate whose predicate is off, so rustc strips the declaration before module resolution and the crate compiles
- **THEN** the system still reports the ambiguity constitution error (exit 2) — cfg-conditionality covers an *absent* conventional file and never two present ones

#### Scenario: A dual-backed declaration inside a cfg_if arm is still an ambiguity

- **WHEN** a `mod child;` declared inside a `cfg_if!` arm resolves to both `src/child.rs` and `src/child/mod.rs`
- **THEN** the system still reports the ambiguity constitution error (exit 2) — arm membership makes an absence tolerable, never two present files resolvable

#### Scenario: A dual-backed declaration alongside a resolved cfg_attr(path) candidate is still an ambiguity

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod child;`, `weird.rs` exists on disk, and BOTH `src/child.rs` and `src/child/mod.rs` also exist
- **THEN** the system still reports the ambiguity constitution error (exit 2) — a resolved conditional remap candidate makes an *absent* conventional file tolerable, never an ambiguity between two present ones resolvable

#### Scenario: An unconditionally missing conventional file is a constitution error

- **WHEN** a crate declares a plain `mod child;` with no `#[cfg]` gate and neither `src/child.rs` nor `src/child/mod.rs` exists
- **THEN** the system reports a constitution error (exit 2) naming both expected paths, rather than silently dropping `crate::child` from the reachable set

#### Scenario: A walk refusal names its compilation unit

- **WHEN** a library root `lib.rs` declares `mod ghost;` and no file backs it
- **THEN** the refusal opens `cannot walk crate 'x' in compilation unit 'lib.rs':` before naming `crate::ghost`
- **PINNED-BY** `a_walk_refusal_names_its_compilation_unit`

#### Scenario: A walk refusal names where each declaration is written

- **WHEN** `src/a.rs` declares `mod ghost;` and no file backs it; or a block inline module carries `#[cfg_attr(unix, path = "d")]` and `#[cfg_attr(windows, path = "e")]`, neither directory exists, and its body declares `pub mod m;`
- **THEN** the refusal names `src/a.rs` as where `crate::a::ghost` is declared; and names both `d/m.rs` and `e/m.rs` among the files it expected
- **PINNED-BY** `a_walk_refusal_names_where_each_declaration_is_written`

#### Scenario: A bare cfg-gated missing file is tolerated

- **WHEN** a crate declares `#[cfg(feature = "extra")] mod child;` and neither conventional file exists
- **THEN** the system skips the declaration rather than erroring, since an off predicate legitimately leaves the module with no file in this configuration

#### Scenario: An inner attribute is no attribute of the mod after it
- **WHEN** a crate root writes `#![cfg(unix)]` and then `mod c;`, and no file backs `crate::c`; or the attribute before `mod c;` is `#![path = "x.rs"]`, or `mod tests { #![cfg(test)] mod c; }`
- **THEN** the system reports a scan error for the missing file, and the attribute reader reads neither the `cfg` as making `c` conditional nor the `path` as remapping it: an inner attribute applies to the item it is written in, not to the `mod` after it. rustc refuses the crate-root rows (`E0583`) in every build, and the `mod tests` row wherever `tests` is compiled, since there its `cfg(test)` removes `tests` itself
- **PINNED-BY** `an_inner_attribute_is_no_attribute_of_the_mod_after_it`
- **PINNED-BY** `both_readers_take_the_attribute_name_from_one_position`

#### Scenario: A missing file for a module declared inside a cfg_if arm is tolerated

- **WHEN** a crate declares `cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }`, `src/unix_impl.rs` exists, and `src/windows_impl.rs` does not
- **THEN** the system skips the fileless arm declaration rather than erroring, and judges the crate — matching the identical shape written as two bare-`#[cfg]`-gated declarations, which it already tolerates; refusing one spelling and accepting the other would make a working build's verdict depend on which form its author chose

#### Scenario: An arm module whose file exists is still reached and governed

- **WHEN** a crate declares `cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }`, only `src/unix_impl.rs` exists, it contains a forbidden import, and a boundary governs `crate::unix_impl`
- **THEN** the system reports the forbidden import violation (exit 1) — tolerating the sibling arm's absent file does not stop the present arm's module from being observed

#### Scenario: A cfg_attr-wrapped missing file with no path meta is not tolerated

- **WHEN** a crate declares `#[cfg_attr(unix, allow(dead_code))] mod child;` and neither conventional file exists
- **THEN** the system reports the missing-file constitution error (exit 2), because this `cfg_attr` carries no `path` remap and applies no `cfg` — such a `cfg_attr` never removes the item, so with no candidate to back it the absent file is a genuine compile error in every configuration, and tolerating it would be a silent pass over source that cannot build

#### Scenario: A single resolved cfg_attr(path) candidate tolerates a missing conventional file

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod child;`, `weird.rs` exists on disk, and neither `src/child.rs` nor `src/child/mod.rs` exists
- **THEN** the system skips the plain-file requirement rather than erroring — the resolved candidate is treated exactly like a bare `#[cfg]`-tolerated absence — and `weird.rs` is still governed under `crate::child` through the union-scan the sibling requirement below already performs

#### Scenario: Stacked resolved cfg_attr(path) candidates tolerate a missing conventional file

- **WHEN** a crate declares `#[cfg_attr(unix, path = "unix_child.rs")] #[cfg_attr(not(unix), path = "other_child.rs")] mod child;`, BOTH `unix_child.rs` and `other_child.rs` exist on disk, and neither `src/child.rs` nor `src/child/mod.rs` exists
- **THEN** the system skips the plain-file requirement rather than erroring, and both `unix_child.rs` and `other_child.rs` are governed under `crate::child` — the shape every real rustc build compiles through exactly one of the two mutually-exhaustive targets, never through a conventional file this declaration never needs

#### Scenario: A qualified applied path names no module target

- **WHEN** a crate declares `#[cfg_attr(any(), foo::path = "bogus.rs", path = "real.rs")] mod plat;` and `bogus.rs` exists on disk
- **THEN** no dimension reads `bogus.rs` as a target of `crate::plat`, because the built-in remap is the **single-segment** `path` and a segment reached through `::` is somebody else's attribute — measured under rustc 1.96.0, edition 2021, `--crate-type lib`, the declaration compiles, since a false predicate expands no applied attribute and never resolves `foo::path`
- **AND** the same narrowing governs the applied target as much as the `cfg_attr` wrapper: reading a qualified target reports a violation against source the governed tree does not compile, and counts a probe inside it as coverage for a seam nothing probes on any real build
- **PINNED-BY** `a_qualified_applied_path_is_not_a_module_target`

#### Scenario: A raw-identifier attribute name is the built-in it spells

- **WHEN** a crate declares `#[r#path = "imp_unix.rs"] mod imp;`, with `imp_unix.rs` on disk and a conventional `src/imp.rs` present as well
- **THEN** every dimension reads `imp_unix.rs`, because the attribute's own name position takes a raw identifier exactly as a `cfg_attr`'s argument list does — measured under rustc 1.96.0, edition 2021, `--crate-type lib`, the declaration compiles with only `imp_unix.rs` present, and with both present it is the remapped file that is compiled
- **PINNED-BY** `all_three_dimensions_read_an_unconditional_raw_identifier_path_remap`

#### Scenario: A raw-identifier bare cfg grants the absent-file tolerance

- **WHEN** a crate declares `#[r#cfg(target_os = "none")] mod gone;` with no `gone.rs` anywhere
- **THEN** no dimension reports the missing-file constitution error, because `r#cfg` is the built-in `cfg` and a false predicate removes the `mod` item outright — measured under rustc 1.96.0, edition 2021, `--crate-type lib`, the declaration compiles with no backing file, exactly as the plain spelling does
- **PINNED-BY** `all_three_dimensions_tolerate_an_absent_file_behind_a_raw_identifier_cfg`

#### Scenario: A raw-identifier spelling of a path remap is the same remap

- **WHEN** a crate declares `#[cfg_attr(unix, r#path = "imp_unix.rs")] mod imp;`, or the nested `#[cfg_attr(unix, r#cfg_attr(not(target_os = "none"), path = "imp_unix.rs"))] mod imp;`, and `imp_unix.rs` exists on disk
- **THEN** every dimension reads `imp_unix.rs` as the remapped target, because `r#` changes an identifier's lexical spelling and not the name it spells — measured under rustc 1.96.0, edition 2021, `--crate-type lib`, both declarations compile with only `imp_unix.rs` present, and with a conventional `imp.rs` present as well it is the remapped file rustc compiles
- **PINNED-BY** `all_three_dimensions_read_a_raw_identifier_spelling_of_a_cfg_attr_path`

#### Scenario: A conventional file beside a raw-identifier remap does not hide the remapped one

- **WHEN** a crate declares `#[cfg_attr(unix, r#path = "imp_unix.rs")] mod imp;` with a clean `src/imp.rs` present and every violation in `imp_unix.rs`
- **THEN** the violation in `imp_unix.rs` is reported, because governing the conventional file alone reports clean over the file the build actually contains — a false negative produced by a spelling of the governed code, which the Core Contract's non-bypassability forbids
- **PINNED-BY** `a_conventional_file_beside_a_raw_identifier_remap_does_not_hide_the_remapped_one`

#### Scenario: An unresolved cfg_attr(path) candidate alone does not tolerate a missing conventional file

- **WHEN** a crate declares `#[cfg_attr(windows, path = "windows_only.rs")] mod child;`, `windows_only.rs` does NOT exist on disk, and neither `src/child.rs` nor `src/child/mod.rs` exists either
- **THEN** the system reports the missing-file constitution error (exit 2) — every candidate this declaration could compile through is absent, and no bare `#[cfg]`/`cfg_if!` arm applies, so the module is genuinely unbacked on every configuration, matching the runtime dimension's identical boundary for this shape (三儀 ⊥ 三儀)

#### Scenario: An absent module file beneath what a cfg removes is tolerated

- **WHEN** a `mod` has no file and a `cfg` removes something enclosing it: `#[cfg(any())] fn f() { #[path = "x.rs"] mod m; }`, `fn f() { #[cfg(any())] { #[path = "x.rs"] mod m; } }`, the same with `#[cfg_attr(all(), cfg(any()))]`, `fn f() { cfg_if::cfg_if! { if #[cfg(any())] { #[path = "x.rs"] mod m; } } }`, a `fn` holding that `mod` inside a `cfg_if!` arm, a generic `fn`, a match arm and a field each carrying `#[cfg(any())]` over that `mod`, `#[cfg(any())] mod o { mod i; }`, or `#[cfg(any())] mod o;` over an `o.rs` declaring `mod i;`; and, as controls, the same `fn` with no `cfg`, with `#[cfg_attr(unix, allow(dead_code))]`, a `cfg` on the match arm or field before the one holding the `mod`, and an `o.rs` declaring `mod i;` under a plain `mod o;`
- **THEN** the system skips each fileless declaration and judges the crate (exit 0), refuses each control (exit 2), and still reads an `x.rs` that exists beneath a compiled-out `fn`, reporting the call in it: rustc 1.96.0, edition 2021, builds each skipped row with no file behind the inner `mod`, and refuses each control for that file
- **PINNED-BY** `an_absent_module_file_beneath_what_a_cfg_removes_is_tolerated`

#### Scenario: A block module's file and a path through it carry one path
- **WHEN** a crate root writes `pub fn g() { mod k { #[path = "y.rs"] pub mod m; } k::m::s(); }` and `src/k/y.rs` writes `pub use crate::secret::go as s;`; or `k` also holds `pub mod sx { pub use crate::secret::go; }` and `y.rs` calls `super::sx::go()`; under a prefix `crate::secret`
- **THEN** the system reports `crate::secret::go in crate` for the first and `crate::secret::go in crate::{block}::k::m` for the second: the walk naming `y.rs`'s module and the scope a path through `k` names read one path, and rustc 1.96.0, edition 2021, builds both
- **PINNED-BY** `a_block_modules_file_and_a_path_through_it_carry_one_path`

#### Scenario: A path attribute gives a block inline module a directory of its own
- **WHEN** a crate root writes `pub fn f() { #[path = "d"] mod k { pub mod m; } }`, the same with `#[cfg_attr(all(), path = "d")]`, with a nested `pub mod j { pub mod m; }`, or `pub fn f() { mod k { #[path = "d"] pub mod j { pub mod m; } } }`, and the `m.rs` rustc reads calls `std::process::id()`
- **THEN** the system reports the call in `src/d/m.rs`, `src/d/m.rs`, `src/d/j/m.rs` and `src/k/d/m.rs` respectively: rustc 1.96.0, edition 2021, builds each, reading the plain `mod m;` from the directory the path attribute names
- **PINNED-BY** `a_path_attribute_gives_a_block_inline_module_a_directory_of_its_own`

#### Scenario: A cfg_attr path is a block inline module's only base
- **WHEN** a crate root writes `pub mod k;` over `src/k/mod.rs` and `pub fn f() { #[cfg_attr(unix, path = "d")] mod k { pub mod m; } k::m::s(); }`, `src/d/m.rs` calls `std::process::id()`, and `src/k/m.rs` is absent, or exists and calls `std::process::abort()`
- **THEN** the system reports the call in `src/d/m.rs` alone for each, neither refusing the absent `src/k/m.rs` nor reading the present one: rustc 1.96.0, edition 2021, builds each, and refuses the `mod m;` where the `cfg_attr` does not apply
- **PINNED-BY** `a_cfg_attr_path_is_a_block_inline_modules_only_base`

#### Scenario: A cfg before a separator its construct holds is not read — a stated bound
- **WHEN** a `#[path = "x.rs"] mod m;` with no `x.rs` stands in a block of an item, statement, parameter or field carrying `#[cfg(any())]` whose own tokens hold a `,`, a brace group or an attribute before that block: `let _v: HashMap<u8, u8> = { … }`, `let _c = |_a: u8, _b: u8| { … }`, `if a {} else { … }`, a match arm `Foo { a } => { … }`, `let _s = S { a: 1 }.a + { … }`, `fn h<A, B>(_: [u8; { … }])`, `fn h<T>() where T: Copy, [u8; { … }]: Sized`, a field or a parameter typed `HashMap<u8, [u8; { … }]>`, or `struct T<A, B>(A, B, [u8; { … }]);`
- **THEN** the system refuses the file (exit 2), where rustc 1.96.0, edition 2021, builds each: the owner of the block is read back to that separator, so the construct's `cfg` is not read — a refusal declared, never a silent pass
- **PINNED-BY** `a_cfg_before_a_separator_its_construct_holds_is_not_read`

#### Scenario: An absent path remap target inside a cfg_if arm is tolerated

- **WHEN** a `#[path = "windows_impl.rs"] mod imp;` is declared inside a `cfg_if!` arm and that target file does not exist
- **THEN** the system skips the declaration rather than reporting the remap-target-missing constitution error, the same cfg-conditional test the plain-absence outcome uses

### Requirement: An unconditional path-remapped module is followed to its target

The system SHALL follow a file-form module declared with an **unconditional, direct** `#[path = "…"]` attribute (`mod foo;`) to its author-chosen target: the target's imports SHALL be observed under the declared module's logical path, and the module SHALL be a governable target at that path — matching 渾儀 (semantic) and 漏刻 (runtime), which follow the same relocation for a declaration carrying one direct path attribute. The target is resolved relative to `path_base`: the declaring file's own directory, with each enclosing inline `mod` name accumulated onto it (rustc's rule) — the crate root's own directory for a `#[path]` written there, or `<accumulated dir>/<name>` for one written inside an inline `mod name { … }`. A `#[path]`-loaded file is itself mod-rs-like, so a `#[path]` or conventional child written inside it resolves from ITS OWN directory in turn. A same-named conventional file beside the remapped declaration (e.g. `foo.rs` beside a `#[path = "weird.rs"] mod foo;`) remains an orphan Rust never compiles as that module — it SHALL NOT be governed in the remap's target place, the same "not compiled ⇒ not governed" rule as an undeclared orphan and an inline-only shadow, applied to the remap case: the orphan file alone is excluded, never the whole logical path.

An unconditional target that does not exist on disk is a genuine broken reference (rustc itself errors on it) and SHALL be a scan error (exit 2), never a silent skip. A `#[path]` chain that resolves back to a source file already open on the path from the crate root (only possible through `#[path]`, since ordinary conventional/inline nesting is bounded by the crate's finite file list) SHALL likewise be a scan error, never an unbounded walk — tracked by the set of files open on the *current descent path*, not a monotonic whole-crate visited set, so two sibling or cousin declarations legitimately sharing one `#[path]` target (rustc compiles the same file twice, as two distinct modules) is never misreported as a cycle.

The scanner SHALL recognize both a direct `#[path = "…"]` and a `path = "…"` meta recursively wrapped in one or more `#[cfg_attr(predicate, …)]` attributes. rustc compiles the first path attribute written on a declaration and reports every later one as unused, so which of them the scanner reads SHALL be decided by position, leaving only a `cfg_attr` predicate to the build, which this cfg-blind scanner does not evaluate. The first direct `#[path = "…"]` SHALL be the direct remap. A `cfg_attr(..., path = "...")` target written before it, or on a declaration with no direct one, SHALL be a candidate, since it is the first attribute wherever its predicate holds, and the scanner SHALL perform a union-scan across the direct target and every candidate target file that physically exists on disk (`path.exists()`). A path attribute written after the first direct one, direct or `cfg_attr`, is never compiled, and SHALL NOT be read: neither followed nor refused. A direct `#[path]` written after a `cfg_attr` candidate is the first attribute only where that candidate's predicate is false, so its target SHALL be tolerated absent where some candidate's file exists — the test a lone `cfg_attr` path already meets — and SHALL be the missing-file scan error where none does, since no configuration then builds. A `cfg_attr` that applies a `cfg` — `#[cfg_attr(pred, cfg(…))]` — can remove the item, and SHALL make the declaration cfg-conditional as a bare `#[cfg]` does. Candidate files that do not exist on disk SHALL be safely skipped without triggering a scan error (treating them as absent under the active compilation target). An unconditional `#[path]` attribute on an inline module (`mod foo { … }`) does not relocate the module's own content (rustc treats the attribute as a no-op for that purpose — the body already IS the module) and SHALL NOT make that inline module disappear from reachability, but it DOES relocate the base directory the inline body's OWN file-form children resolve from, exactly as it would for a file-form declaration — the system SHALL follow it there too, resolved from the declaring source's own `path_base`.

#### Scenario: A path-remapped module's imports are observed at its real target

- **WHEN** a crate declares `#[path = "weird/place.rs"] mod foo;` and `weird/place.rs` contains `use crate::other::Thing;`
- **THEN** the system observes an import of `crate::other` for `crate::foo`, attributed to `weird/place.rs`

#### Scenario: A path-remapped module is a governable target at its real file

- **WHEN** a boundary targets `crate::foo`, declared via `#[path = "weird/place.rs"] mod foo;`, and `weird/place.rs` contains a forbidden import
- **THEN** the system reports the violation naming `crate::foo` and the file `weird/place.rs` — never a constitution error, and never governing a same-named conventional orphan in its place

#### Scenario: A declaration a predicate may remove or redirect tolerates its absent file

- **WHEN** a crate root declares `#[cfg_attr(unix, path = "b.rs")] #[path = "a.rs"] mod m;` with only `b.rs` present, or `#[cfg_attr(all(), cfg(any()))] mod m;` with no `m.rs`
- **THEN** the system governs `b.rs` as `crate::m` in the first and nothing in the second, and reports no scan error in either: rustc 1.96.0 builds both on unix. With neither `a.rs` nor `b.rs`, the first is the scan error naming `a.rs`, since no configuration builds it
- **PINNED-BY** `a_declaration_a_predicate_may_remove_or_redirect_tolerates_its_absent_file`
- **PINNED-BY** `a_direct_path_after_a_candidate_no_file_backs_is_a_scan_error`

#### Scenario: A path attribute is read by its position

- **WHEN** a crate root declares `mod m;` after `#[path = "a.rs"] #[path = "b.rs"]`, after `#[path = "a.rs"] #[cfg_attr(unix, path = "b.rs")]`, or after `#[cfg_attr(unix, path = "b.rs")] #[path = "a.rs"]`, with `a.rs` present and `b.rs` present or absent
- **THEN** the system governs `a.rs` as `crate::m` in every row, governs `b.rs` as `crate::m` only in the `cfg_attr`-first row and only where it exists, and reports no scan error in any row: a path after the first direct one is never compiled, which rustc 1.96.0 confirms by building each direct-first row with `b.rs` absent and reporting the later attribute unused
- **PINNED-BY** `a_path_attribute_is_read_by_its_position`

#### Scenario: A conventional orphan beside a path-remapped declaration is not governed

- **WHEN** a crate declares `#[path = "weird.rs"] mod foo;`, `weird.rs` is clean, and a conventional `foo.rs` also exists containing a forbidden import
- **THEN** the system reports no violation for `foo.rs` — the orphan is not compiled as `crate::foo` and is never governed in the remap's place

#### Scenario: A plain child of a path-remapped module is governed under its logical path

- **WHEN** a crate declares `#[path = "other/weird.rs"] mod kernel;`, `other/weird.rs` declares a plain `mod child;`, and `other/child.rs` contains a forbidden import
- **THEN** the system observes the import under `crate::kernel::child`, attributed to `other/child.rs` — even though `other/child.rs`'s own on-disk location does not structurally match its logical path

#### Scenario: A missing unconditional path target is a scan error

- **WHEN** a crate declares `#[path = "absent.rs"] mod foo;` and `absent.rs` does not exist
- **THEN** the system reports a scan error (exit 2), because an unconditional `#[path]` target absent from disk is a genuine broken reference rustc itself rejects — never a silent skip

#### Scenario: Two declarations sharing one path target is not a cycle

- **WHEN** a crate declares `#[path = "s.rs"] mod a;` and `#[path = "s.rs"] mod b;`, both resolving to the same real file `s.rs`
- **THEN** the system reports both `crate::a` and `crate::b` as reachable and governable, because rustc compiles `s.rs` twice as two distinct modules — never a reported cycle

#### Scenario: A path chain cycling back to an already-open file is a scan error

- **WHEN** a crate declares `mod a { #[path = "../lib.rs"] mod b; }` at the crate root, and `a`'s own accumulated directory makes `../lib.rs` resolve back to the crate root file itself
- **THEN** the system reports a scan error (exit 2) rather than looping or overflowing the stack — the cycle is on the descent path from the crate root, not merely a repeated file elsewhere in the crate

#### Scenario: A nested path crossing into a mutually-exclusive cfg sibling's own target is not a cycle

- **WHEN** a crate declares `#[cfg(feature = "a")] #[path = "variant_a.rs"] mod imp;` and `#[cfg(feature = "b")] #[path = "variant_b.rs"] mod imp;`, and `variant_a.rs` itself declares `#[path = "variant_b.rs"] mod also_b;`
- **THEN** the system follows `crate::imp::also_b` to `variant_b.rs` without reporting a cycle, because the two `#[cfg]` arms' targets are never simultaneously open in any real single build — the ancestor tracking that would otherwise misreport this is scoped per physical source file, never merged across mutually-exclusive `#[cfg]` sibling arms of one logical module path

#### Scenario: A nested path inside a plain child of an inline cfg arm is not a cycle through a sibling arm

- **WHEN** a crate declares `#[cfg(feature = "u")] mod x { mod y; }` (inline) and `#[cfg(feature = "w")] #[path = "windows_x.rs"] mod x;` (file-form), where only the inline arm declares the plain child `y`, and `y`'s own file declares `#[path = "../windows_x.rs"] mod cross;`
- **THEN** the system follows `crate::x::y::cross` to `windows_x.rs` without reporting a cycle — a plain child's ancestor set is scoped to only the source that actually declared it, never unioned across `x`'s other mutually-exclusive `#[cfg]` sources

#### Scenario: A grandchild of a plain child of a path-remapped module is governed under its logical path

- **WHEN** a crate declares `#[path = "other/weird.rs"] mod kernel;`, `other/weird.rs` declares a plain `mod child;` (resolved to `other/child.rs`), and `other/child.rs` itself declares a further plain `mod grandchild;`
- **THEN** the system observes a forbidden import in `other/child/grandchild.rs` under `crate::kernel::child::grandchild` — the ordinary stem-subdirectory convention relative to `child.rs`'s own location, since a plain child reached through a remap is not itself mod-rs-like for its own further children

#### Scenario: A stray file at a remapped module's naive structural location is not phantom-governed

- **WHEN** a crate declares `#[path = "other/weird.rs"] mod kernel;`, `other/weird.rs` declares a plain `mod child;` (resolved to the real `other/child.rs`), and an unrelated, wholly undeclared file also happens to physically sit at `kernel/child.rs` (the location a plain `mod child;` inside a NON-remapped `kernel` would occupy)
- **THEN** the system governs only `other/child.rs` under `crate::kernel::child` — the stray file at `kernel/child.rs` is never compiled by rustc (`kernel` is wholly remapped) and must never be phantom-governed alongside the real one merely for coincidentally sharing its naive structural path

#### Scenario: A cfg_attr-wrapped path attribute undergoes union-scan when target file exists

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod foo;`, `weird.rs` exists on disk containing `use crate::forbidden::Y;`, and a boundary governs `crate::foo` forbidding `crate::forbidden`
- **THEN** the system observes the import in `weird.rs` for `crate::foo` and reports the violation, performing union-scan over all physically existing candidate files rather than marking the module out of scope

#### Scenario: A missing cfg_attr-wrapped path target is safely skipped

- **WHEN** a crate declares `#[cfg_attr(windows, path = "win_only.rs")] mod foo;` and `win_only.rs` does not exist on disk
- **THEN** the system skips `win_only.rs` without raising a scan error, treating absent conditional targets as inactive under the current source checkout

#### Scenario: A nested cfg_attr-wrapped path attribute is recognized as a candidate remap

- **WHEN** a crate declares `#[cfg_attr(a, cfg_attr(b, path = "weird.rs"))] mod foo;` and `weird.rs` exists on disk
- **THEN** the system recursively recognizes the applied `path` target and includes `weird.rs` in the union-scan for `crate::foo`

#### Scenario: A cfg_attr without an applied path remains governable

- **WHEN** a crate declares `#[cfg_attr(path, allow(dead_code))] mod foo;` with a conventional `foo.rs`
- **THEN** the system does not mistake the predicate named `path` for a remap and governs the conventional module normally

#### Scenario: An unconditional path attribute wins regardless of attribute order

- **WHEN** a crate declares `#[cfg_attr(some_platform, path = "b.rs")] #[path = "a.rs"] mod foo;` — a cfg-conditional remap textually BEFORE the unconditional one on the same declaration
- **THEN** the system follows the unconditional `#[path = "a.rs"]` target exactly as it would if the two attributes were written in the opposite order, since rustc compiles `a.rs` whenever `some_platform` does not hold regardless of which attribute is written first

#### Scenario: An unconditional path on an inline module relocates its own file-form children

- **WHEN** a crate declares `#[path = "thread_files"] pub mod thread { pub mod local_data; }`, `thread_files/local_data.rs` contains a forbidden import, and no `thread/` directory exists at all
- **THEN** the system observes the forbidden import under `crate::thread::local_data`, attributed to `thread_files/local_data.rs` — the `#[path]` attribute is not treated as a no-op merely because the module it precedes is inline; it relocates where the inline body's own file-form children resolve from, exactly as it would for a file-form declaration

### Requirement: A module may restrict who imports it to a closed allowlist

A module boundary SHALL support an inbound **closed-allowlist** rule: `ModuleBoundary::in_crate(p).module(m).must_only_be_imported_by([x, …]).because(...)` declares that the protected module `m` may be imported only by a listed importer `x` (or anything beneath it) or by `m`'s own subtree; any **other** module that imports `m` (or anything beneath `m`) SHALL be a violation. This is the inbound dual of `restrict_imports_to` (the outbound closed allowlist), exactly as `must_not_be_imported_by` is the inbound dual of `must_not_import`. An **empty** allowlist permits only `m`'s own subtree (every outside importer reacts).

The system SHALL observe this from the crate's source `use` declarations across all reachable files (the same crate-wide inbound scan `must_not_be_imported_by` uses): a file whose enclosing module imports `m` or anything beneath `m`, and whose enclosing module is neither within `m`'s own subtree nor within any allowlisted importer's subtree, SHALL be a violation — naming `m` as the target and the offending importing module as the finding. The "or beneath" test SHALL be `::`-delimited on both sides (an exact match OR an `x::` prefix), so an allowlisted importer `crate::facade` does not admit a sibling `crate::facadex`, and a protected `crate::internal` is not matched by a sibling `crate::internal_util`. Allowlist entries SHALL be canonicalized (raw-identifier `r#name` → `name`) like the governed path. A file whose enclosing module is `m` or beneath `m` SHALL NOT be treated as an importer — a module importing its own subtree is not an inbound edge. External imports SHALL remain out of scope.

The finding SHALL be the importing module path, deduplicated so one offending importer yields one violation per protected target; the rule SHALL carry severity (default enforce, `warn` available), an `AllowlistGap` repair polarity (an importer outside the allowed set — repair by removing the import or widening the allowlist), and flow through the baseline like the other module rules. The protected module `m` SHALL be a reachable file-based module, with the same inline/unknown constitution-error handling as other module targets. Declaring the rule with `m` = `crate` (the crate root) SHALL be a constitution error (exit 2), self-describing and distinct from a violation, because every internal import is then "m or beneath" and the rule could never react as an inbound rule.

#### Scenario: An import from outside the allowlist violates

- **WHEN** `crate::internal` is protected by `must_only_be_imported_by(["crate::facade"])` and a file in `crate::consumer` declares `use crate::internal::Secret;`
- **THEN** the system emits a violation naming `crate::internal` as the target and the offending importer `crate::consumer`, and exits 1 at enforce severity

#### Scenario: An import from an allowlisted importer is clean

- **WHEN** the same boundary holds and a file in `crate::facade` declares `use crate::internal::Secret;`
- **THEN** the system reports no violation, because `crate::facade` is an allowlisted importer

#### Scenario: The allowlist admits the importer's subtree

- **WHEN** a file in `crate::facade::v1` declares `use crate::internal::Secret;` under `must_only_be_imported_by(["crate::facade"])`
- **THEN** the system reports no violation, because `crate::facade::v1` is beneath the allowlisted importer `crate::facade`

#### Scenario: A prefix-colliding importer sibling is not admitted

- **WHEN** a file in `crate::facadex` declares `use crate::internal::Secret;` under `must_only_be_imported_by(["crate::facade"])`
- **THEN** the system emits a violation, because `crate::facadex` is neither `crate::facade` nor beneath `crate::facade::` — a sibling is not admitted by the allowlist

#### Scenario: The protected module's own subtree is always allowed

- **WHEN** `crate::internal` is protected by `must_only_be_imported_by(["crate::facade"])` and a file in `crate::internal::deep` declares `use crate::internal::Secret;`
- **THEN** the system reports no violation, because a module within the protected module's own subtree is never an inbound importer

#### Scenario: An empty allowlist forbids every outside importer

- **WHEN** `crate::internal` is protected by `must_only_be_imported_by([])` and any module outside `crate::internal`'s own subtree imports it
- **THEN** the system emits a violation for that importer, because an empty allowlist permits only the protected module's own subtree

#### Scenario: An external import is ignored

- **WHEN** a file in `crate::consumer` declares `use serde::Deserialize;` under `must_only_be_imported_by(["crate::facade"])` protecting `crate::internal`
- **THEN** the system reports no violation, because external imports are out of scope

#### Scenario: Multiple allowlisted importers are all admitted

- **WHEN** `crate::internal` is protected by `must_only_be_imported_by(["crate::facade", "crate::api"])` and files in both `crate::facade` and `crate::api` import it, while `crate::consumer` also imports it
- **THEN** the system reports no violation for `crate::facade` or `crate::api`, and one violation naming `crate::consumer`

#### Scenario: Restricting importers of the crate root is a constitution error

- **WHEN** a boundary declares `must_only_be_imported_by([x])` on `crate` (the crate root)
- **THEN** the system emits a self-describing constitution error and exits 2 — distinct from a boundary violation, never a silent pass — because every internal import would be within the protected subtree and the rule could never react as an inbound rule

### Requirement: Module violations de-duplicate by semantic identity

Module-boundary findings SHALL de-duplicate by governed target, structured rule key, and structured
observed fact. Source file, rendered import text, and traversal order SHALL NOT affect identity.

#### Scenario: Repeated imports remain one fact
- **WHEN** the same governed module observes the same violating import in multiple files or lines
- **THEN** it emits one identity while a structurally different import remains distinct

### Requirement: Mixed direct and conditional path remaps remain observable

When one file-module declaration carries both direct `#[path = "…"]` and one or more `cfg_attr(..., path = "…")` remaps, 圭表 SHALL conservatively resolve every physically existing written candidate and SHALL scan their union. Attribute order SHALL NOT silently remove a candidate, and canonically identical candidates SHALL be evaluated once.

#### Scenario: Conditional remap after a direct remap is observed

- **WHEN** a module declares a direct path followed by a conditional path whose physical target contains a forbidden import
- **THEN** the forbidden import reacts even though current rustc configurations may select only one candidate

#### Scenario: Conditional remap before a direct remap is observed

- **WHEN** the same two remaps are written in the opposite order and either physical target contains a forbidden import
- **THEN** each physical candidate remains in the governed source union

### Requirement: Module projection preserves legacy depth shape

A legacy module boundary whose evaluation depth is `Subtree` SHALL omit `scan_depth` from its
projection so its JSON and derived Markdown remain byte-compatible. A boundary configured with the
non-legacy `Shallow` depth SHALL emit `scan_depth: "shallow"`.

#### Scenario: Legacy subtree projection is unchanged

- **WHEN** a module boundary is constructed without an explicit depth modifier
- **THEN** its projection contains no `scan_depth` field

#### Scenario: Shallow projection is explicit

- **WHEN** a module boundary is configured with `ScanDepth::Shallow`
- **THEN** its projection contains `scan_depth: "shallow"`

### Requirement: Lexical hygiene never panics on malformed source

The system SHALL react 0/1/2 on any governed source file, including one whose lexical structure is
malformed in a way rustc itself would reject (an unterminated block comment, or any other unclosed
construct reaching end-of-file) — never panicking or otherwise aborting the process. An unterminated
block comment SHALL be treated as extending through end-of-file: every byte within it, including a
trailing byte that would otherwise be the orphaned tail of a multi-byte character, is consumed as
part of the comment rather than re-scanned as code.

#### Scenario: An unterminated block comment swallowing a multi-byte character does not panic

- **WHEN** a governed source file ends in an unterminated block comment (no closing `*/`) whose
  dropped content includes a multi-byte UTF-8 character with no trailing newline after it
- **THEN** the system reacts with a normal violation or clean outcome instead of panicking

#### Scenario: An unterminated block comment at end of file does not panic

- **WHEN** a governed source file ends in an unterminated block comment with no trailing newline,
  regardless of what precedes it
- **THEN** the system reacts 0/1/2 instead of panicking, and every module declared before the
  comment remains observable

### Requirement: A pathologically nested use tree is a scan error, never a silent drop

The system SHALL react 0/1/2 on a `use` declaration whose brace-group nesting depth is bounded by
a measured stack-safety cap, and past that cap SHALL fail loud (a constitution error, exit 2)
rather than silently dropping the sub-tree from observation. A real, compilable `use` nested past
the cap would otherwise vanish entirely from the imported-path set with no report —
`Outcome::Clean` when a real violation exists, the false negative the core contract forbids.
Nesting comfortably under the cap SHALL be observed exactly as a shallower tree would be. A tree is
refused only when it is nested past the cap, so the refusal's statement of the cap is true of the
tree it names: a tree nested as deep as the cap is read. The refusal SHALL name the file the tree
is written in. The bindings such a tree makes are missing from its file's scopes, so an import whose
head is read through any scope of that file SHALL be refused as well, naming the file the import is
written in and the module it reads through, rather than read without them; an import whose
resolution never reads that file is judged.

#### Scenario: A use tree nested past the depth cap is a scan error

- **WHEN** a crate declares a `use` tree whose brace-group nesting exceeds the depth cap this
  scanner supports
- **THEN** the system reports a constitution error (exit 2) naming the depth bound it could not
  judge past, rather than silently omitting the tree's imports from observation

#### Scenario: A use tree as deep as the cap is read, and one level past it is refused
- **WHEN** `crate::b` imports `crate::a::X` through `use crate::{…{a::X}…};` nested 128 braces deep, or 129, under `must_not_import("crate::a")`
- **THEN** 128 braces reports `crate::a::X`, and 129 is a constitution error (exit 2) naming the cap of 128 brace levels and `src/b.rs`, the file it was met in
- **PINNED-BY** `an_import_rule_reads_a_use_tree_128_braces_deep_and_refuses_129`

#### Scenario: An import read through another file's unreadable use tree is refused
- **WHEN** in an edition-2015 package `crate::a` writes `use hub::X;`, and the crate root binds `hub` only in `pub use crate::{…forbidden as hub…};` nested 130 braces deep, under `must_not_import("crate::forbidden")` on `crate::a`
- **THEN** the system reports a constitution error (exit 2) naming `src/a.rs`, the module `crate` it reads through, and the cap of 128 brace levels, where the import went unreported; with `pub use forbidden as hub;` instead it reports `crate::forbidden::X`
- **PINNED-BY** `an_import_read_through_another_files_unreadable_use_tree_is_refused`

#### Scenario: A use tree nested just under the depth cap is still observed

- **WHEN** a crate declares a `use` tree nested well under the depth cap
- **THEN** the system judges it normally, observing every imported path exactly as a shallower
  tree would be

### Requirement: A conditional path remap on an inline module relocates its children's base

The scanner SHALL treat a `cfg_attr(…, path = "…")` remap on an **inline** `mod name { … }` as naming
the **base directory** that body's own file-form children resolve from, exactly as it already does for
an unconditional `#[path]` on the same shape. The first direct `#[path]` SHALL relocate that base, and every
`cfg_attr` target written before it, or with no direct one, SHALL be a **candidate** base whose directory, where
it exists, is read as well, because rustc takes the first path attribute written and the scanner does not
evaluate `cfg`, so it cannot know which a given build compiles — preferring one would silently drop every child
beneath the other, the false negative the core contract forbids. A path written after the first direct one is
never compiled and SHALL NOT be read. A candidate directory that exists without a child the body declares is the
missing-file scan error it is beside two `cfg_attr` bases, since the configuration selecting it does not build. A
direct `#[path]` written after a candidate applies only where that candidate's predicate is false, so its base SHALL
be descended as a candidate's is.

Each candidate base — every `cfg_attr` target, a direct one written after a candidate, **and** the conventional
directory — SHALL be descended
only when it exists as a directory. Descending an absent one would spuriously fail loud on the body's
other, unrelated nested items solely because one platform's directory is missing, even when another
candidate already backs them. When **no** candidate exists as a directory, the conventional base SHALL
be descended anyway — the direct base where there is one — so a nested reference genuinely broken on every
platform still fails loud.

Resolving such a body's children from the conventional base alone would report a missing-module constitution
error (exit 2) on source that compiles cleanly under real rustc — refusing to judge a crate rather than judging
it. 漏刻 states a rule for the same shape and implements it independently (三儀 ⊥ 三儀: the same rule, not the
same function); where the two readings of path attributes still differ is tracked in `BACKLOG.md`.

#### Scenario: A conditional remap on an inline module is followed to its child base

- **WHEN** a crate declares `#[cfg_attr(unix, path = "unix_dir")] pub mod x { pub mod y; }` with
  `src/unix_dir/y.rs` present, no conventional `src/x/` directory, and `y.rs` importing a forbidden
  module
- **THEN** the system observes that import and reacts (exit 1) attributed to `unix_dir/y.rs`, rather
  than reporting a missing-module constitution error for `src/x/y.rs` on a crate that builds

#### Scenario: An inline module's path bases are read by their position
- **WHEN** a crate root declares `mod m { pub mod k; }` after `#[cfg_attr(unix, path = "c")] #[path = "d"]`, after `#[path = "d"] #[cfg_attr(unix, path = "c")]`, or after `#[path = "d"] #[path = "c"]`, with `src/d/k.rs` present and `src/c/k.rs` present or absent beside an existing `src/c/`
- **THEN** in the `cfg_attr`-first order the system governs both `c/k.rs` and `d/k.rs` as `crate::m::k` where both exist, and reports the missing-file scan error naming `c/k.rs` where only `c/` does, as rustc 1.96.0 refuses it with `E0583` on unix; in the direct-first orders it governs `d/k.rs` alone and reports no scan error, which rustc 1.96.0 confirms by building them with no `c/k.rs`
- **PINNED-BY** `a_path_attribute_is_read_by_its_position`

#### Scenario: An inline module's direct path after a candidate is descended as a candidate
- **WHEN** a crate root declares `#[cfg_attr(unix, path = "c")] #[path = "d"] mod m { pub mod k; }` with only `src/c/k.rs` on disk, or with neither `src/c/` nor `src/d/`
- **THEN** the system governs `c/k.rs` as `crate::m::k` and reports no scan error in the first, as rustc 1.96.0 builds it on unix, and in the second reports the missing-file scan error naming `d/k.rs`, since no configuration builds it
- **PINNED-BY** `an_inline_modules_direct_path_after_a_candidate_is_descended_as_a_candidate`

#### Scenario: Every present conditional base of an inline module is descended

- **WHEN** one inline `mod x { pub mod y; }` carries two `cfg_attr` path remaps naming two directories
  that both exist, whose `y.rs` files import two different forbidden members
- **THEN** the system reacts to both — the union is real, cfg-blind, and neither base is silently
  preferred over the other

#### Scenario: An inline module whose every conditional base is absent still fails loud

- **WHEN** an inline `mod x { pub mod y; }` carries a `cfg_attr` path remap whose directory is absent
  and no conventional `src/x/` directory exists either
- **THEN** the system reports the missing-module constitution error (exit 2) for the child, because the
  reference is broken on every configuration — the absent-base tolerance never becomes a silent pass

### Requirement: Every compiled root of a package is governed

The governed corpus of a package SHALL be **every** compiled crate root Cargo reports for it — each
library-kind target and each `bin` target, wherever its source path lies — together with the modules
reachable from each root through `mod` declarations. A violation written in any of them SHALL react. The
static and semantic dimensions SHALL agree on this scope, and the runtime dimension already observes
every root, so the three no longer disagree about which of a package's source Cargo actually compiles.

Each root SHALL be resolved as its own module graph: two roots of one package both denote the module path
`crate`, and neither's declarations, inline-module shadowing, nor `#[path]` remaps SHALL leak into the
other's resolution. A root's corpus SHALL begin at its own root file: a file whose path alone would denote
`crate` without being this root — a top-level `mod.rs`, or a top-level `lib.rs` or `main.rs` beside a
conventional root — is another compiled root or a file no target compiles, and SHALL NOT enter this root's
graph as a second source of `crate` by virtue of its path. Such a file reached through an explicit `mod` or
`#[path]` declaration is that declared module's source, like any other file. An observation SHALL
carry the compilation unit it came from as an identity role, per `structured-violation-identity`.

A governed module SHALL be looked for in **every** root's graph, and an unknown-module constitution error
SHALL be reported only when **no** root has it. A module legitimately exists in one root's graph and not
another's — a library's internals are not the binary's — so erroring per root would make a boundary on a
library-only module exit 2 for the package's `bin` root, refusing to judge source that compiles.

What a root without the governed module contributes depends on the rule's perimeter. For a rule whose
perimeter is the governed module — every outbound and inbound rule, and `must_not_call_inline` —
that root holds nothing the rule governs, and nothing is observed there. External-crate confinement and
`confine_inline_call` have the whole root as their perimeter, with the governed module only the region where
the confined import or inline call is permitted, so such a root is judged with an empty permitted region, per
`external-crate-confinement` and `inline-symbol-path-confinement`.

A package whose metadata reports no target at all SHALL fall back to its conventional source directory,
which is what synthetic metadata in a caller's own tests carries; that fallback is load-bearing and SHALL
NOT be dropped when the corpus becomes per-root. A package whose metadata reports targets none of which is a
library kind or a binary SHALL be a constitution error naming it, rather than falling back: no compiled root
reads its `src/`, so a finding there would be about source nothing builds, and the governed corpus is the
library-kind and `bin` roots, not an example, test, bench or build-script root.

A target root whose path does not lie under the package's own directory SHALL be a constitution error
naming it, because the compilation-unit identity role is that path relative to the package directory and
no checkout-independent label exists for a root outside it. This SHALL NOT be resolved by using the path
as given: that path is the checkout's own location, so the identity would differ between two clones of one
commit. Refusing to judge is the Core Contract's ordering over a silently checkout-dependent identity.
This bound is narrow by construction — a target rooted anywhere INSIDE the package, including outside its
source directory, is governed normally.

While evaluating each root, only "this root does not have the governed module" SHALL be deferred to the
other roots. Every other failure — an unreadable source, a resolution ambiguity, a root outside the
package directory, a governed module that root declares inline — SHALL propagate immediately, because
deferring it until a sibling root happened to be governable would silently pass over source the system
could not read. An inline target is present in its root rather than absent from it: deferred, a sibling
root backing the same path with a file would be governed in its place and the inline body left unobserved.

An outbound rule's finding SHALL carry the **importing module** — the module that lexically declares the
`use`, so an import inside an inline `mod inner { … }` is attributed to that module rather than the
containing file's. Without it, two different modules of the governed subtree importing the same forbidden
path collapse to one finding, so accepting one in a baseline masks the other. The inbound rules already
qualify by importer; this makes the two families symmetric, and the dedup key becomes the (importing
module, import path) pair rather than the path alone.

#### Scenario: A violation in a package's binary root reacts

- **WHEN** a package builds both `src/lib.rs` and `src/main.rs`, a forbidden construct is written only in
  `main.rs`, and a boundary governs `crate`
- **THEN** the system reacts, naming `main.rs` as the offending file, rather than reporting the package
  clean

#### Scenario: Every binary target's root is governed wherever it lives

- **WHEN** a package with a library root also builds `src/bin/tool.rs`, a `[[bin]]` whose `path` is inside
  the source directory, and a `[[bin]]` whose `path` is outside it, each containing a forbidden construct
- **THEN** each reacts — a conventional `src/bin` target and a custom `path` are treated identically, and
  a root outside the source directory is not skipped for lying elsewhere

#### Scenario: A module present in only one root is not an unknown-module error

- **WHEN** a boundary governs a module declared only in the library root, and the package also builds a
  `bin` root whose graph has no such module
- **THEN** the system governs it in the library root and reports no constitution error, rather than
  refusing to judge because one root lacks it

#### Scenario: A target declared inline in one root is refused though another root backs it with a file

- **WHEN** one root of a package declares the governed module inline and another root backs the same path
  with a file, whichever of the library and the binary holds the inline form, under an outbound rule or
  an external-crate confinement
- **THEN** the system reports the inline-module constitution error (exit 2), rather than governing the
  file-backed root in its place and reporting clean

#### Scenario: A source file no target compiles is in no root's corpus

- **WHEN** a package with `autobins = false` builds only its library, and an uncompiled `src/main.rs`
  either declares `pub mod shared;` beside a library that declares `shared` inline, or imports a module a
  boundary on `crate` forbids — or an undeclared top-level `src/mod.rs` imports it
- **THEN** the library's inline `shared` is refused as an inline target (exit 2) rather than governed
  through `src/shared.rs`, and the uncompiled file's import does not react, because none of those files'
  content is source the package compiles

#### Scenario: A package whose targets compile no root is refused

- **WHEN** a boundary governs a package every target of which Cargo reports as an example, a test, a bench or
  a build script
- **THEN** the system reports a constitution error (exit 2) naming the package, rather than judging its `src/`
  directory: no compiled root reads it, so a finding there would be about source nothing builds
- **PINNED-BY** `a_package_whose_targets_compile_no_root_is_refused`
- **PINNED-BY** `guibiao_and_hunyi_agree_on_the_parallel_no_compiled_root_wording`

#### Scenario: An example, test, bench or build-script root is not governed — a stated bound

- **WHEN** a package with a library root has an example, test, bench or build-script target whose source
  imports what a boundary on the package forbids
- **THEN** the system reports no violation for it: the governed corpus is the library-kind and `bin` roots,
  the code the package ships, and those targets are compiled beside it rather than into it
- **PINNED-BY** `an_example_root_is_not_governed`

#### Scenario: A conventional root filename reached through a declaration is that module's source

- **WHEN** a package with `autobins = false` builds only its library, whose `lib.rs` declares
  `pub mod main;`, and `src/main.rs` imports a module a boundary on `crate::main` forbids
- **THEN** the system reports the violation with `crate::main` as its importer, because the declaration makes `main.rs`
  that module's source, rather than refusing the file as a cycle back to the crate root or excluding it
  for its filename

#### Scenario: One root's declarations do not leak into another's graph
- **WHEN** two roots of one package each declare a same-named submodule backed by different files
- **THEN** each root's graph resolves its own, so neither root's module is observed in place of the
  other's

#### Scenario: A target root outside the package directory is refused

- **WHEN** a package declares a target whose `path` reaches outside the package's own directory, and a
  boundary governs that package
- **THEN** the system reports a constitution error naming that root, rather than labeling the observation
  by a path that varies with where the repository was cloned

#### Scenario: A scan error in one root is not deferred away by a governable sibling

- **WHEN** one of a package's roots cannot be judged for a reason other than the governed module being
  absent from it, while another root hosts that module
- **THEN** the system reports that failure, rather than reporting the sibling's violations and swallowing
  it

#### Scenario: Two modules importing one forbidden path stay two findings

- **WHEN** two different modules of one governed subtree each import the same forbidden path
- **THEN** the system emits two findings distinguished by their importing module, so accepting one in a
  baseline does not suppress the other
