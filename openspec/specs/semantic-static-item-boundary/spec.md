# semantic-static-item-boundary Specification

## Purpose

The 渾儀 (semantic) dimension's static-item capability: declare in Rust that a module, and every
module beneath it, declares no `static` item and no `thread_local!`. It governs *declarations*,
observed from the local crate's AST
(`syn`): a `static` or `static mut` wherever it is written in the subtree, a foreign `static` in an
`extern` block, and each static a `thread_local!` invocation declares. Anchored at a module, it is
architectural intent rather than a style check: statics outside the anchored subtree are not governed.

What a call does to process-global state — `std::env::set_var`, a registry a dependency keeps — is
not a declaration and is outside this capability; confining such calls is `must_not_call_inline`'s
(`inline-symbol-path-confinement`). A boundary's reason is therefore written as what it observes —
*declares no `static` item or `thread_local!`* — and never as *has no global state*. The capability is
not part of any composed profile.

## Subject

- `crates/hunyi/src/static_item.rs`
- `crates/hunyi/src/scan/static_sites.rs`
- `crates/hunyi/src/dsl/static_item.rs`
- `crates/hunyi/src/tests/static_item.rs`

## Requirements
### Requirement: Static-item boundary declared in Rust

A static-item boundary SHALL be expressed as Rust code and is part of the single source of truth:
`StaticBoundary::in_crate(package).module(module).must_not_declare_static().because(reason)`, with the
shared `.warn()` and `.with_anchor(..)`, composed through `Constitution::static_boundary(..)` and
exported in the prelude. Its rule key SHALL be `tianheng.rule/hunyi/static-item` with no parameter,
its rule label `must not declare static items`, its polarity `DenyBreach`, and its violation target the
boundary's module in its canonical spelling. The system MUST NOT require TOML, YAML, Markdown, or any
generated policy file.

#### Scenario: A module-level static and static mut react with their kind

- **WHEN** a boundary on `crate::kernel` governs a module declaring `pub static COUNTER: u8 = 0;` and `static mut LEGACY: u8 = 0;`
- **THEN** each is one violation targeting `crate::kernel` under the rule label `must not declare static items` with `DenyBreach` polarity, whose fact is `tianheng.fact/hunyi/static-item` of kind `static` and `static_mut` respectively
- **PINNED-BY** `a_module_level_static_and_static_mut_react_with_their_kind`

### Requirement: The whole anchored subtree is governed, and nothing outside it

A boundary SHALL govern every module at or beneath its anchor, always: there is no seam-only depth and
no knob selecting one. The 渾儀 exposure families default to the anchored module alone because they
govern a seam — what a module exposes to its callers — and a descendant's seam is its own. A static is
state the anchored layer holds wherever beneath the anchor it is declared, so a descendant's static is
the anchor's concern; this is the same default the 圭表 module rules and `must_not_call_inline` take for
the same reason. Anchoring at `crate` SHALL govern the whole crate. The boundary SHALL report its depth as
`ScanDepth::Subtree`, and its `list` projections SHALL read the depth from the boundary and carry it in
the vocabulary every depth-carrying 渾儀 module boundary shares — `including_submodules: true` and
`scan_depth: "subtree"` in JSON, a ` (including submodules)` rule-line suffix in text. Each fact SHALL be attributed to the
module that declares it — an inline child, a `#[path]` child, a module reached through an item-position
`cfg_if!` arm — exactly once.

#### Scenario: Each static is attributed once to its declaring module

- **WHEN** a boundary on `crate::kernel` governs a subtree with statics in `crate::kernel` (inside an item-position `cfg_if!` arm), an inline child, a `#[path]` child and a file child, and `crate::other` declares a static too
- **THEN** each subtree static is one finding carrying its own declaring module, every finding targets `crate::kernel`, and the static in `crate::other` does not react
- **PINNED-BY** `each_static_is_attributed_once_to_its_declaring_module`

#### Scenario: The projected depth is the boundary's own

- **WHEN** a static-item boundary is projected by `list` as JSON and as text
- **THEN** the JSON entry's `including_submodules` and `scan_depth` and the text rule line's suffix are those the boundary's `scan_depth()` — `Subtree` — gives every depth-carrying semantic boundary
- **PINNED-BY** `every_semantic_depth_projects_in_one_vocabulary_read_from_the_boundary`

#### Scenario: A boundary anchored at the crate root governs the whole crate

- **WHEN** a boundary is anchored at `crate` over a crate whose `crate::kernel` and `crate::other` each declare a static
- **THEN** both react
- **PINNED-BY** `a_boundary_anchored_at_crate_governs_the_whole_crate`

### Requirement: Every declared static is observed wherever the subtree writes it

The system SHALL observe a `static` or `static mut` item at module level and in any body the subtree
holds — a free function, an inherent method, a trait's default method, a trait-impl method, a closure,
and a `const` or `static` initializer. It SHALL observe a foreign `static` and `static mut` in an
`extern` block, including a `safe`- or `unsafe`-qualified one in an `unsafe extern` block, read through the one
foreign-item decoder the visibility and signature-coupling capabilities share. It SHALL observe each
static a `thread_local!` invocation declares, at item and at statement position: the invocation's body
is read by the macro's own grammar — attributed, visibility-qualified `static NAME: T = init`
declarations separated by `;`, the last one's `;` optional, as std's `$init:expr $(;)?` makes it — and
every `static` in it is its own finding. A `'static` lifetime, a `let` binding
and a `const` item declare no static and SHALL NOT react.

#### Scenario: A body static is owned by its named value items

- **WHEN** statics are declared in a free fn, a fn nested in it, an inherent method, a trait default method, a trait-impl method, a closure, a `const` initializer and a `static` initializer
- **THEN** each reacts, owned respectively by `f`, `f::inner`, `Foo::m`, `Tr::d`, `<Tr for Foo>::m`, the closure's enclosing fn, `const C` and `static S`
- **PINNED-BY** `a_body_static_is_owned_by_its_named_value_items`

#### Scenario: Every static one thread_local! declares reacts

- **WHEN** one `thread_local!` invocation declares `SCRATCH` and then `DEPTH`
- **THEN** both are findings of kind `thread_local` — the second is the one a reader taking the body's first item would drop
- **PINNED-BY** `every_static_a_thread_local_declares_reacts`

#### Scenario: A thread_local! whose last static has no `;` reacts

- **WHEN** the governed module writes `thread_local!(static FOO: Cell<u32> = Cell::new(1));`, the form std's own documentation uses, and `thread_local! { static A: Cell<u8> = Cell::new(0); pub static B: Cell<u8> = const { Cell::new(0) } }`
- **THEN** `FOO`, `A` and `B` each react with kind `thread_local`, never a constitution error
- **PINNED-BY** `a_thread_local_whose_last_static_has_no_semicolon_reacts`

#### Scenario: A statement-position thread_local! is owned by its fn

- **WHEN** `pub fn bump() { std::thread_local! { static LOCAL: u8 = 0; } }` is declared in the governed module
- **THEN** `LOCAL` reacts with kind `thread_local`, owned by `bump`
- **PINNED-BY** `a_statement_position_thread_local_is_owned_by_its_fn`

#### Scenario: Foreign statics react with their mutability

- **WHEN** an `extern "C"` block declares `static A` and `static mut B`, and an `unsafe extern "C"` block declares `pub safe static C`, `pub unsafe static mut D`, `static E` and a `safe fn`
- **THEN** `A`, `C` and `E` react as `foreign_static`, `B` and `D` as `foreign_static_mut`, and the `fn` does not react
- **PINNED-BY** `foreign_statics_react_with_their_mutability`

#### Scenario: A static lifetime, a let and a const are clean

- **WHEN** the governed module writes `&'static str`, a `T: 'static` bound, a `let` binding and `pub const LIMIT: u8 = 3;`
- **THEN** no violation is reported
- **PINNED-BY** `a_static_lifetime_a_let_and_a_const_are_clean`

### Requirement: A static's identity is its kind, declaring module, name and owner

A static-item fact SHALL be `tianheng.fact/hunyi/static-item` with shape equal to its kind — one of
`static`, `static_mut`, `foreign_static`, `foreign_static_mut`, `thread_local` — and identity fields
`module`, `name`, `owner`, `unit` and `governing_package`. `module` is the declaring module, never the
anchor. `owner` is the chain of named value items enclosing the static, joined by `::`: empty at module
level; a fn by its name; a method by its impl's self type or trait, `Foo::m`, `Tr::m` or
`<Tr for Foo>::m`, the self type named by the owner reader unsafe confinement uses; an initializer as
`const NAME` or `static NAME`. A closure and a block add nothing, so a static in a closure is owned by
the fn holding it. Mutability SHALL be part of the kind, so a baseline accepting `static X` does not
accept a later `static mut X`; a `safe` or `unsafe` qualifier SHALL NOT be. Identity SHALL never carry
scan position, so reordering declarations keeps every identity.

#### Scenario: Every spelling of thread_local! shares the bare identity

- **WHEN** the same static is declared through `thread_local!`, `std::thread_local!`, `::std::thread_local!` and `r#thread_local!` in turn
- **THEN** each spelling yields the identity the bare one does
- **PINNED-BY** `every_spelling_of_thread_local_shares_the_bare_identity`

#### Scenario: Reordering statics keeps their identities

- **WHEN** a module-level static and a fn-body static swap places in the source
- **THEN** the set of violation identities is unchanged
- **PINNED-BY** `reordering_statics_keeps_their_identities`

#### Scenario: Same-named statics under one owner share one identity — a stated bound

- **WHEN** one fn declares a static `M` in each of two nested blocks, two `const _` initializers each declare a static `H`, or two closures of one fn each declare a static `C`
- **THEN** each pair is one finding, because a block, a `const _` and a closure add no name to the owner chain and scan position is not identity; a baseline accepting one also accepts the other
- **PINNED-BY** `nested_block_statics_share_one_identity`
- **PINNED-BY** `anonymous_const_statics_share_one_identity`
- **PINNED-BY** `closure_statics_share_one_identity`

### Requirement: The anchor is a canonical module anchor naming a module that exists

The boundary's module SHALL be read by the module-anchor rule every module-anchored 渾儀 capability
shares (`semantic-signature-coupling`): `crate`, or `crate::` followed by `::`-separated identifiers,
with `r#x` read as `x`; any other spelling is a constitution error (exit 2) suggesting the canonical
one. The anchor SHALL name a module some compilation unit of the package declares; one no unit declares
is the shared unknown-module constitution error, and one absent from a single unit is governed where it
is declared.

#### Scenario: The anchor is held to the module-anchor rule

- **WHEN** a boundary is anchored at `kernel` over a crate declaring `crate::kernel`, and another at `crate::nope`
- **THEN** the first is a constitution error quoting `kernel` and suggesting `crate::kernel`, and the second is the shared unknown-module error
- **PINNED-BY** `the_anchor_is_held_to_the_module_anchor_rule`
- **PINNED-BY** `every_anchored_capability_refuses_a_non_canonical_spelling`

### Requirement: thread_local! is recognized by its name, and a rename is refused

A macro invocation SHALL be read as `thread_local!` when its path's last segment, with a raw prefix
removed, is `thread_local` — the name gate this dimension applies to `cfg_if!`, since reading a
macro's body is sound only for a macro known by name. A crate governed by a static-item boundary that
renames it — `use … thread_local as X` with `X` neither `thread_local` nor `_`, anywhere in the crate, a
function body included — SHALL be a constitution error (exit 2) naming the new name and asking for the
macro to be written by its name, because an invocation under the new name would escape the name gate.
Only the governed crate's own source is read for a rename, and all of it is: every file of a
compilation unit that holds the anchor is parsed, so a file outside the anchored subtree that cannot be
read or parsed SHALL make the whole boundary a constitution error (exit 2), since the rename it might
hold is undecided.

#### Scenario: A crate renaming thread_local! refuses to judge — a stated bound

- **WHEN** the governed crate writes `use std::thread_local as tls;` at module level, or inside a function body outside the anchored module, or re-exports `pub(crate) use std::thread_local as tlq;` from a module another one globs in
- **THEN** the system emits a constitution error (exit 2) saying the crate renames `thread_local` and asking for `thread_local!` to be written directly
- **PINNED-BY** `a_renamed_thread_local_refuses_to_judge`

#### Scenario: An unparseable file outside the anchor refuses to judge

- **WHEN** a boundary on `crate::kernel` governs a crate whose `crate::other` file does not parse
- **THEN** the system emits a constitution error (exit 2) naming that file, rather than judging `crate::kernel` alone
- **PINNED-BY** `an_unparseable_file_outside_the_anchor_refuses_to_judge`

#### Scenario: A rename bringing no new name is not refused

- **WHEN** the governed module writes `use std::thread_local as thread_local;` and `use std::thread_local as _;` beside a `thread_local!`
- **THEN** the boundary is judged, and the `thread_local!` static reacts
- **PINNED-BY** `a_rename_to_thread_local_or_underscore_is_not_refused`

#### Scenario: A local macro sharing the thread_local! name over-reacts — a stated bound

- **WHEN** the governed module defines `macro_rules! thread_local` and invokes `thread_local! { static NOT_REAL: u32 = 0; }`, which that local macro expands to nothing
- **THEN** `NOT_REAL` reacts as a `thread_local` finding, because the invocation is recognized by name
- **PINNED-BY** `a_local_thread_local_macro_over_reacts_is_a_bound`

#### Scenario: A foreign-crate rename of thread_local! is a documented bound

- **WHEN** the governed module writes `use dep::tls;` and `tls! { static FROM_DEP: u8 = 0; }`, where another crate re-exports `thread_local` as `tls`
- **THEN** no violation is reported and the boundary is not refused, because another crate's source is not read
- **PINNED-BY** `a_foreign_crate_rename_of_thread_local_is_a_documented_bound`

### Requirement: A declaration that cannot be named is refused, never passed

The system SHALL refuse to judge (exit 2), naming the module and cause, rather than pass a declaration
at or beneath the anchor that it cannot name: a `thread_local!` whose body does not read as `static`
items; a static whose enclosing impl's self type or trait has no supported rendering, since no
positional label is invented for it; a foreign item the shared foreign-item decoder cannot read. A
declaration of those kinds outside the anchor is ungoverned and SHALL NOT refuse the boundary. A rename
of `thread_local` SHALL still be read across the whole unit, and a declaration that cannot be named SHALL
NOT stop that read, because the module holding it is exactly where a rename beside it would otherwise be
dropped. An unreadable or unparseable source anywhere in a compilation unit that holds the anchor SHALL
be a constitution error too, because the whole unit is read for a rename.

#### Scenario: A thread_local! body that is not static declarations refuses to judge — a stated bound

- **WHEN** the governed module writes `thread_local! { not a static }` or `thread_local! { fn not_a_static() {} }`
- **THEN** the system emits a constitution error (exit 2) naming the module, never a clean pass
- **PINNED-BY** `an_unparseable_thread_local_body_refuses_to_judge`

#### Scenario: An unnameable owner refuses to judge

- **WHEN** a static is declared in a method of `impl Tr<{ N + 1 }> for Foo`
- **THEN** the system emits a constitution error (exit 2) naming the static and that its enclosing impl trait cannot be named
- **PINNED-BY** `an_unnameable_owner_refuses_to_judge`

#### Scenario: An undecodable foreign item refuses to judge

- **WHEN** an `unsafe extern "C"` block in the governed module holds `#[cfg(any())] pub fn with_body() {}`
- **THEN** the system emits the shared undecodable-foreign-item constitution error (exit 2) naming it, the module `crate::kernel`, and deletion or expanded declaration as the repair
- **PINNED-BY** `an_undecodable_foreign_item_refuses_to_judge`

#### Scenario: A declaration outside the anchor that cannot be named does not refuse the boundary

- **WHEN** a boundary on `crate::kernel` governs a crate whose `crate::other` writes `thread_local! { not a static }`
- **THEN** the boundary is judged and `crate::kernel`'s `IN_KERNEL` reacts
- **PINNED-BY** `semantic_error_in_ungoverned_module_does_not_fail_governed_static_scan`

#### Scenario: A rename beside an unnameable declaration outside the anchor is still refused

- **WHEN** a boundary on `crate::kernel` governs a crate whose `crate::other` writes `thread_local! { not a static }` and then `use std::thread_local as tls;`
- **THEN** the system emits the rename constitution error (exit 2) naming `tls` and asking for `thread_local!` by its name, never a judgement over `crate::kernel`
- **PINNED-BY** `a_rename_beside_an_unnameable_declaration_outside_the_anchor_is_still_refused`

### Requirement: Observation bounds are stated, not silent

The capability SHALL declare, each as a bound with its pinning test, the shapes it does not decide the
way a reader might assume: a static appearing only in a macro's expansion is not observed; a `static`
whose `#[cfg]` predicate is false on the host is observed as written, the policy every 渾儀 capability
shares; and a `const` with interior mutability is a value inlined at each use rather than one shared
location, so it declares no static.

#### Scenario: A macro-generated static is a documented bound

- **WHEN** the governed module declares a static only through `macro_rules! make_static`, or wraps a `thread_local!` in another macro
- **THEN** no violation is reported, because macros other than `thread_local!` are not expanded
- **PINNED-BY** `a_macro_generated_static_is_a_documented_bound`

#### Scenario: Cfg-gated statics are observed as written — a stated bound

- **WHEN** the governed module declares `#[cfg(test)] static TEST_ONLY` and `#[cfg(any())] static NEVER`
- **THEN** both react, because this AST observation does not evaluate cfg predicates
- **PINNED-BY** `static_cfg_is_observed_as_written`

#### Scenario: An interior-mutable const is not a static — a stated bound

- **WHEN** the governed module declares `pub const FRESH: Cell<u8> = Cell::new(0);`
- **THEN** no violation is reported
- **PINNED-BY** `an_interior_mutable_const_is_not_a_static`

### Requirement: CI reaction, severity, and baseline parity

The system SHALL fold static-item findings into the same exit-code contract as the other dimensions (0
clean / 1 enforce violation / 2 constitution or scan error), aggregated with the other boundaries. A
boundary SHALL carry a severity (`enforce` default, or `warn`, which reports without failing), and its
violations SHALL be gated against the same `Baseline` by `(target, rule_key, fact)`.

#### Scenario: A warn boundary reports at warn severity

- **WHEN** a `.warn()` boundary governs a module declaring a static
- **THEN** the violation is reported at `warn` severity
- **PINNED-BY** `a_warn_boundary_reports_at_warn_severity`
