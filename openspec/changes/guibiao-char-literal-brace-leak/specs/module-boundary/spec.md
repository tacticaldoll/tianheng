## MODIFIED Requirements

### Requirement: Module imports observed from source use declarations

The system SHALL observe module imports by scanning the target crate's source `use` declarations. It SHALL resolve `crate`, `self`, and `super` paths to absolute `crate::…` module paths, expand grouped (`{a, b}`) and glob (`::*`) forms, and ignore paths whose first segment is an external crate. A first segment that names a crate-root module SHALL be resolved to `crate::…` **only when the importing file is the crate root**: there a sibling `mod` is in scope and shadows the extern prelude, so a bare `use foo::…` is the local module. In a submodule a bare first segment reaches only the extern prelude — it is an external crate, or a compile error — and SHALL be treated as external, even when a crate-root module of that name exists. The crate-root module names used for this resolution SHALL be observed from the crate's own source as **declared modules** — a `mod name;` or `mod name { … }` declaration in the crate-root file(s) — not from the mere existence of a like-named source file: an undeclared orphan source file (e.g. a stray `src/foo.rs` that no `mod foo;` declares) does NOT make its name a crate-root module, because Rust does not bring an undeclared file into scope and a bare `use foo::…` then resolves through the extern prelude. A path written with a leading `::` (`use ::name::…`) is the explicit external/global form and SHALL be treated as external even when its first segment matches a crate-root module. Text inside comments and string literals SHALL NOT be treated as a `use` (or `mod`) declaration: it is removed before scanning, so neither a `//` inside a string nor a `use …;` written inside a string affects the result. Bare path expressions and macro-generated imports SHALL be out of scope (see the scanner decision in `PROJECT.md`); the rule enforces only what real `use` declarations observe. In particular, a `use` written inside a macro body — a `macro_rules!` definition OR a macro invocation (`ident! {…}` / `(…)` / `[…]`) — is a macro-generated import: the `macro_rules!` definition (its name and balanced body) and any macro invocation's balanced `{}`/`()`/`[]` body are removed before scanning, so such a `use` SHALL NOT be observed. A `use` token that is **not an import statement** — specifically a **precise-capturing bound** (`-> impl Trait + use<'a, T>`, stable Rust), where the `use` token is immediately followed (after optional whitespace) by `<` — SHALL NOT be treated as an import and SHALL NOT consume a following real `use` declaration; a `use` *statement* is always followed by a path (an identifier, `{`, `*`, `::`, or `crate`/`self`/`super`), never `<`, so the following-token `<` is the discriminator, and skipping the bound keeps the next real `use` observable (never a silent drop). Comments, string literals, and char literals — normal, byte, and raw string forms, and a char literal's full scalar value regardless of its UTF-8 byte length — SHALL be removed before scanning, so that a character a char literal contains (including `{` or `}`) is never mistaken for a real structural brace by the reachability walk. Modules SHALL be file-based **and reachable from the crate root via `mod` declarations**: a source file that no `mod` declaration brings into scope — an undeclared orphan, at the crate root or anywhere in a subtree — is not a module of the crate, is not governed, and its imports SHALL NOT be observed, matching the compiler (which never compiles it). A governed module path that matches no reachable source file SHALL be a constitution error (exit 2), never a silent pass. A governed source file that exists but cannot be read SHALL likewise be a scan error (exit 2), never silently skipped — an unreadable file is "cannot judge", not "nothing to judge", and skipping it could hide a real violation. A governed source directory that cannot be traversed SHALL likewise be a scan error (exit 2), naming the directory, never silently skipped — the same "cannot judge, not nothing to judge" rule, because a skipped subtree could hide a real violation.

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
- **THEN** the system treats the import as external and does not observe `crate::serde::Deserialize`, because a submodule's bare first segment reaches only the extern prelude

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
- **THEN** the system does not observe an import of `crate::projection`, because the `macro_rules!` body is a macro-generated import and is removed before scanning

#### Scenario: A use written inside a macro invocation body is not observed

- **WHEN** a file declares `some_macro! { use crate::projection::Thing; }` and no real `use` of that path outside the macro
- **THEN** the system does not observe an import of `crate::projection`, because a macro invocation body is a macro-generated import and is removed before scanning

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
