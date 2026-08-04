## REMOVED Requirements

### Requirement: Only the resolved crate root and its reachable modules are governed

**Reason**: It recorded the single-root corpus as a documented out-of-scope bound and named per-target
module graphs as the design work that would close it. That work is done here, so the bound no longer
exists and a requirement may not keep describing one. Replaced by `Every compiled root of a package is
governed` below.

**Migration**: A violation in a package's non-first compiled root — `main.rs` beside a `lib.rs`, any
`src/bin/*.rs`, any `[[bin]] path` — now reacts where it was previously unobserved, so a first run after
this change can report violations that were never reported rather than relabeled ones. Every module and
semantic baseline entry re-keys, absorbed by the regeneration this release already requires.

## ADDED Requirements

### Requirement: Every compiled root of a package is governed

The governed corpus of a package SHALL be **every** compiled crate root Cargo reports for it — each
library-kind target and each `bin` target, wherever its source path lies — together with the modules
reachable from each root through `mod` declarations. A violation written in any of them SHALL react. The
static and semantic dimensions SHALL agree on this scope, and the runtime dimension already observes
every root, so the three no longer disagree about which of a package's source Cargo actually compiles.

Each root SHALL be resolved as its own module graph: two roots of one package both denote the module path
`crate`, and neither's declarations, inline-module shadowing, nor `#[path]` remaps SHALL leak into the
other's resolution. An observation SHALL carry the compilation unit it came from as an identity role, per
`structured-violation-identity`.

A governed module SHALL be looked for in **every** root's graph, and an unknown-module constitution error
SHALL be reported only when **no** root has it. A module legitimately exists in one root's graph and not
another's — a library's internals are not the binary's — so erroring per root would make a boundary on a
library-only module exit 2 for the package's `bin` root, refusing to judge source that compiles.

A package whose metadata reports no target at all SHALL fall back to its conventional source directory,
which is what synthetic metadata in a caller's own tests carries; that fallback is load-bearing and SHALL
NOT be dropped when the corpus becomes per-root.

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

#### Scenario: One root's declarations do not leak into another's graph
- **WHEN** two roots of one package each declare a same-named submodule backed by different files
- **THEN** each root's graph resolves its own, so neither root's module is observed in place of the
  other's
