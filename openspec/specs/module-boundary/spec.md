# module-boundary Specification

## Purpose

Govern the intra-crate module import graph that Cargo cannot see — the
differentiated value over `cargo tree` / `cargo-deny`. A module boundary forbids
one module from importing another ("the kernel must not import a projection"),
observed from the target crate's source `use` declarations (use-only, file-based;
see the scanner decision in `PROJECT.md`). Module violations flow through severity
and the baseline exactly like crate violations.
## Requirements
### Requirement: Module boundary declared in Rust

A module boundary SHALL be declared in Rust, targeting a crate and a module path within it and forbidding an import of another module path. It SHALL be declared as `ModuleBoundary::in_crate("app").module("crate::kernel").must_not_import("crate::projection").because("…")`, and SHALL carry a severity (default enforce, `warn` available) like a crate boundary. The umbrella `Boundary` SHALL accept both crate and module boundaries.

#### Scenario: Module boundary holds its target, module, and forbidden import

- **WHEN** a developer declares `ModuleBoundary::in_crate("app").module("crate::kernel").must_not_import("crate::projection").because("…")`
- **THEN** the constitution holds a module boundary on crate `app`, governing module `crate::kernel`, forbidding imports of `crate::projection`, with a non-empty reason

### Requirement: Module imports observed from source use declarations

The system SHALL observe module imports by scanning the target crate's source `use` declarations. It SHALL resolve `crate`, `self`, and `super` paths to absolute `crate::…` module paths, expand grouped (`{a, b}`) and glob (`::*`) forms, and ignore paths whose first segment is an external crate. A first segment that names a crate-root module SHALL be resolved to `crate::…` **only when the importing file is the crate root**: there a sibling `mod` is in scope and shadows the extern prelude, so a bare `use foo::…` is the local module. In a submodule a bare first segment reaches only the extern prelude — it is an external crate, or a compile error — and SHALL be treated as external, even when a crate-root module of that name exists. The crate-root module names used for this resolution SHALL be observed from the crate's own source as **declared modules** — a `mod name;` or `mod name { … }` declaration in the crate-root file(s) — not from the mere existence of a like-named source file: an undeclared orphan source file (e.g. a stray `src/foo.rs` that no `mod foo;` declares) does NOT make its name a crate-root module, because Rust does not bring an undeclared file into scope and a bare `use foo::…` then resolves through the extern prelude. A path written with a leading `::` (`use ::name::…`) is the explicit external/global form and SHALL be treated as external even when its first segment matches a crate-root module. Text inside comments and string literals SHALL NOT be treated as a `use` (or `mod`) declaration: it is removed before scanning, so neither a `//` inside a string nor a `use …;` written inside a string affects the result. Bare path expressions and macro-generated imports SHALL be out of scope (see the scanner decision in `PROJECT.md`); the rule enforces only what real `use` declarations observe. In particular, a `use` written inside a macro body — a `macro_rules!` definition OR a macro invocation (`ident! {…}` / `(…)` / `[…]`) — is a macro-generated import: the `macro_rules!` definition (its name and balanced body) and any macro invocation's balanced `{}`/`()`/`[]` body are removed before scanning, so such a `use` SHALL NOT be observed. A `use` token that is **not an import statement** — specifically a **precise-capturing bound** (`-> impl Trait + use<'a, T>`, stable Rust), where the `use` token is immediately followed (after optional whitespace) by `<` — SHALL NOT be treated as an import and SHALL NOT consume a following real `use` declaration; a `use` *statement* is always followed by a path (an identifier, `{`, `*`, `::`, or `crate`/`self`/`super`), never `<`, so the following-token `<` is the discriminator, and skipping the bound keeps the next real `use` observable (never a silent drop). Comments and string literals — normal, byte, and raw — SHALL be removed before scanning. Modules SHALL be file-based **and reachable from the crate root via `mod` declarations**: a source file that no `mod` declaration brings into scope — an undeclared orphan, at the crate root or anywhere in a subtree — is not a module of the crate, is not governed, and its imports SHALL NOT be observed, matching the compiler (which never compiles it). A governed module path that matches no reachable source file SHALL be a constitution error (exit 2), never a silent pass. A governed source file that exists but cannot be read SHALL likewise be a scan error (exit 2), never silently skipped — an unreadable file is "cannot judge", not "nothing to judge", and skipping it could hide a real violation. A governed source directory that cannot be traversed SHALL likewise be a scan error (exit 2), naming the directory, never silently skipped — the same "cannot judge, not nothing to judge" rule, because a skipped subtree could hide a real violation.

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

#### Scenario: An unknown governed module is a constitution error

- **WHEN** a module boundary governs a module path that matches no reachable source file in the crate
- **THEN** the system reports a constitution error and exits 2

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

A module boundary SHALL report each distinct violation at most once: its violations SHALL be deduplicated by identity `(target, rule, finding_key)`. When the governed module's subtree spans multiple source files that produce the same finding — a parent and a child file importing the same path, or a module backed by both `lib.rs` and `main.rs` (which both resolve to `crate`) — the system SHALL emit a single violation, not one per file. Deduplication SHALL be performed per boundary at the point findings are produced, so a duplicate arising from any other source is not silently suppressed.

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

### Requirement: Imports are attributed to their enclosing inline module

The system SHALL attribute each `use` declaration to the module that lexically encloses it, including an inline `mod name { … }` submodule, rather than to the containing file's module. A `self`/`super` path SHALL be resolved against that enclosing module, and a bare first segment inside an inline submodule SHALL be treated as external even when the file itself is the crate root, matching how the compiler resolves it. A `mod name;` declaration with no inline body does not enclose any `use` and SHALL NOT change attribution.

#### Scenario: A self import inside an inline submodule resolves against that submodule

- **WHEN** the crate-root file declares `mod inner { use self::leaf::Thing; }`
- **THEN** the import is observed as `crate::inner::leaf::Thing`, not `crate::leaf::Thing`, because it is attributed to the enclosing inline module `crate::inner`

### Requirement: Module declarations inside macro bodies are not observed

The system SHALL NOT observe a `mod` declaration written inside a macro body — a `macro_rules!` definition or a macro invocation (`ident! {…}` / `(…)` / `[…]`) — as a real module of the crate, the same out-of-scope rule already applied to a `use` inside a macro body. The macro body SHALL be removed before scanning for `mod` declarations, so a `mod` token inside a `()`/`[]`-delimited macro invocation is not mistaken for a crate-root module declaration.

#### Scenario: A mod inside a macro invocation is not a declared module

- **WHEN** the crate-root file declares `some_macro!( mod ghost; );` and no real `mod ghost;` outside the macro
- **THEN** the system does not treat `crate::ghost` as a declared, reachable module, so a bare `use ghost::…` elsewhere stays external

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

### Requirement: An unconditional path-remapped module is followed to its target

The system SHALL follow a file-form module declared with an **unconditional, direct** `#[path = "…"]` attribute (`mod foo;`) to its author-chosen target: the target's imports SHALL be observed under the declared module's logical path, and the module SHALL be a governable target at that path — matching 渾儀 (semantic) and 漏刻 (runtime), which already follow this same relocation, so all three observation dimensions agree on what rustc actually compiles. The target is resolved relative to `path_base`: the declaring file's own directory, with each enclosing inline `mod` name accumulated onto it (rustc's rule) — the crate root's own directory for a `#[path]` written there, or `<accumulated dir>/<name>` for one written inside an inline `mod name { … }`. A `#[path]`-loaded file is itself mod-rs-like, so a `#[path]` or conventional child written inside it resolves from ITS OWN directory in turn. A same-named conventional file beside the remapped declaration (e.g. `foo.rs` beside a `#[path = "weird.rs"] mod foo;`) remains an orphan Rust never compiles as that module — it SHALL NOT be governed in the remap's target place, the same "not compiled ⇒ not governed" rule as an undeclared orphan and an inline-only shadow, now applied to the remap case instead of excluding the whole logical path.

An unconditional target that does not exist on disk is a genuine broken reference (rustc itself errors on it) and SHALL be a scan error (exit 2), never a silent skip. A `#[path]` chain that resolves back to a source file already open on the path from the crate root (only possible through `#[path]`, since ordinary conventional/inline nesting is bounded by the crate's finite file list) SHALL likewise be a scan error, never an unbounded walk — tracked by the set of files open on the *current descent path*, not a monotonic whole-crate visited set, so two sibling or cousin declarations legitimately sharing one `#[path]` target (rustc compiles the same file twice, as two distinct modules) is never misreported as a cycle.

The scanner SHALL recognize both a direct `#[path = "…"]` and a `path = "…"` meta recursively wrapped in one or more `#[cfg_attr(predicate, …)]` attributes, but only the **direct, unconditional** form is followed. A `cfg_attr`-wrapped `path` remains a stated, cfg-conditional coverage bound (the same family as inline and macro-generated items — see the scanner decision in `PROJECT.md`): it is cfg-conditional, so following it cfg-blind could read a file rustc does not compile in the active configuration; a `cfg_attr` whose applied metas contain no genuine `path` name-value SHALL remain an ordinary module. This cfg-blind exclusion prevents a conditional remap from silently governing a same-named conventional orphan. A `path` attribute on an inline module (`mod foo { … }`) does not relocate it (rustc treats it as a no-op there) and SHALL NOT make that inline module disappear from reachability.

#### Scenario: A path-remapped module's imports are observed at its real target

- **WHEN** a crate declares `#[path = "weird/place.rs"] mod foo;` and `weird/place.rs` contains `use crate::other::Thing;`
- **THEN** the system observes an import of `crate::other` for `crate::foo`, attributed to `weird/place.rs`

#### Scenario: A path-remapped module is a governable target at its real file

- **WHEN** a boundary targets `crate::foo`, declared via `#[path = "weird/place.rs"] mod foo;`, and `weird/place.rs` contains a forbidden import
- **THEN** the system reports the violation naming `crate::foo` and the file `weird/place.rs` — never a constitution error, and never governing a same-named conventional orphan in its place

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

#### Scenario: A cfg_attr-wrapped path attribute is recognized as a remap

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod foo;`, a conventional `foo.rs` exists containing `use crate::forbidden::Y;`, and a boundary governs `crate::foo` forbidding `crate::forbidden`
- **THEN** the system treats `crate::foo` as remapped and reports it as out of scope (exit 2), rather than governing the conventional orphan or silently passing; the scanner does not evaluate whether `unix` is active

#### Scenario: A nested cfg_attr-wrapped path attribute is recognized as a remap

- **WHEN** a crate declares `#[cfg_attr(a, cfg_attr(b, path = "weird.rs"))] mod foo;`
- **THEN** the system recursively recognizes the applied `path` name-value and treats `crate::foo` as remapped and out of scope

#### Scenario: A cfg_attr without an applied path remains governable

- **WHEN** a crate declares `#[cfg_attr(path, allow(dead_code))] mod foo;` with a conventional `foo.rs`
- **THEN** the system does not mistake the predicate named `path` for a remap and governs the conventional module normally

#### Scenario: An unconditional path attribute wins regardless of attribute order

- **WHEN** a crate declares `#[cfg_attr(some_platform, path = "b.rs")] #[path = "a.rs"] mod foo;` — a cfg-conditional remap textually BEFORE the unconditional one on the same declaration
- **THEN** the system follows the unconditional `#[path = "a.rs"]` target exactly as it would if the two attributes were written in the opposite order, since rustc compiles `a.rs` whenever `some_platform` does not hold regardless of which attribute is written first

### Requirement: A multi-target package's cross-root same-named submodule is a stated bound

The system SHALL treat, as a **documented out-of-scope bound** (never a silent claim of cleanliness), the imports written in the **inline** body of a submodule whose name is declared **inline in one crate root and file-backed in the other** of a package that builds both a lib and a bin. Because the system observes a package's source under one conventional-path tree, both crate roots (`lib.rs` and `main.rs`) resolve to `crate` and it maintains no per-target module graphs; so when `lib.rs` declares `mod shared { … }` (inline) and `main.rs` declares `mod shared;` (backed by `shared.rs`), the file-backed `shared.rs` is the governed module and the inline body's imports are NOT observed — the conventional-path model cannot distinguish the lib crate's `crate::shared` from the bin crate's. This is the submodule corollary of the same lib+bin conventional-path conflation the dedup requirement already names; closing it would require per-target module graphs, an amendment beyond the conventional-path scanner.

#### Scenario: A submodule declared inline in the lib root and file-backed in the bin root is a documented bound

- **WHEN** a package's `lib.rs` declares `mod shared { use crate::forbidden::X; }` (inline), its `main.rs` declares `mod shared;` (backed by a clean `shared.rs`), and a boundary governs `crate::shared` forbidding `crate::forbidden`
- **THEN** the system governs `shared.rs` and does not observe the inline body's `use crate::forbidden::X` — a documented lib+bin conventional-path bound, recorded rather than silently claimed clean

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
