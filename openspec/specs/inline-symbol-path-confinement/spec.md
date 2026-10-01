# inline-symbol-path-confinement Specification

## Purpose

The 圭表 (static) inline-symbol-path confinement — the layer-(b) sibling of
`external-crate-confinement`, observing **calls** rather than `use` imports. Within a governed
module subtree it forbids inline symbol-path calls resolving under a declared module-path prefix
(the "core reads no ambient clock; time is injected" pattern). A call-vs-mention default keeps 圭表
free of a read-verb heuristic (type annotations and constants pass); `.ending_with([verbs])`
narrows to adopter-declared read verbs, `.strict_prefix_only()` escalates to any mention. Path
heads resolve from a typed lexical scope table — module and block scopes, `use` bindings, glob
edges followed to a fixed point, and the local `type`-alias and `pub use` re-export closure; a glob
that can bring a prefix-resolving name into scope reacts fail-closed (stated by hazard, not shape). Source-observed on the hand-rolled 圭表 token scanner
(no `syn`: `serde_json` and Unicode's identifier and normalization tables); not `cargo-deny`'s resolved/whole-graph lane.

## Subject

- `crates/guibiao/src/**/*.rs`
- `crates/guibiao/src/tests/symbol_confinement.rs`

## Requirements
### Requirement: Inline-symbol-path confinement declared in Rust

A module boundary SHALL support forbidding, within a governed module subtree, inline symbol
paths resolving under a declared **module-path prefix**, declared as
`ModuleBoundary::in_crate(p).module(s).must_not_call_inline(prefix).because("…")`, where `s`
is the governed subtree and `prefix` is a module-path prefix (e.g. `std::time`). The target is
a **prefix**, never a hand-copied leaf/function list (a leaf list drifts — the `freeze_methods`
anti-pattern). The optional modifiers (`.ending_with`, `.strict_prefix_only`) SHALL hang off a
**dedicated inline-confinement draft stage**, distinct from the shared module-rule draft, so
they cannot be applied to `must_not_import` / `confine_external_crate` (no modifier pollution of
other module rules). It SHALL carry a severity (default `enforce`, `warn` available) like every
other module rule and be accepted by the umbrella `Boundary`. The system MUST NOT require any
generated policy file.

#### Scenario: A confinement holds its subtree, prefix, and reason
- **WHEN** a developer declares `ModuleBoundary::in_crate("app").module("crate::core").must_not_call_inline("std::time").because("time is injected")`
- **THEN** the constitution holds a module boundary on crate `app`, governing subtree `crate::core`, forbidding inline calls under prefix `std::time`, with a non-empty reason and default `enforce` severity

### Requirement: Call-vs-mention default

By default (no narrowing modifier) the system SHALL react on an inline path resolving under the
prefix **only when it is applied as a call** (`path(...)` or `path::<...>(...)`). A file's bytes SHALL be read by the token
tree alone; the tree takes whitespace the Reference's *Whitespace* names, Unicode's
`Pattern_White_Space`, as separating tokens, comments dropped, every literal one token, a multi-byte punctuation
token read whole by maximal munch — the set the Rust Reference's *Tokens → Punctuation* table names, held both ways to
a copy of that table the test carries — and `(`, `[` and `{` paired with their closers before any judgement is made; a raw identifier, a
lifetime, `..`, `..=` and `->` are each one token, so the path after a range operator is a head and never a method's
receiver: `0..std::process::id()` is a call. A path occurrence SHALL be a token run — `::`? *head* ( `::` *segment* |
`::` `<…>` )* — and its role SHALL be read from the tokens beside it alone: the name a `fn` item, a tuple struct or a
tuple variant declares is its definition; a run followed by a parenthesized group is a call; anything else is a
mention. No expression or pattern grammar is read, so where an expression ends decides nothing, and a tuple-struct or
tuple-variant pattern, written as a call is written, is read as a call — a declared over-reaction (bound:
inline-symbol-path-confinement/a-path-in-a-pattern-position-is-read-as-a-call-a-stated-bound). A `<…>` group SHALL be
counted in two places only, by one reading: after `::`, where the `<` is a turbofish, and at a `<` that opens a
qualified path; it closes at a `>` token, or at each `>` a `>>` or `>>=` holds as rustc splits them in a generic list,
never at `->` or `=>`, and a group inside `(…)`, `[…]` or `{…}` is passed over whole, so `size_of::<fn() -> u8>()` is
one path applied as a call. Counting a turbofish says where a path ends; its contents — types and const arguments,
after a path or after a method — SHALL be read as any other tokens are. Everywhere else a `<` is punctuation. A `<` SHALL open a qualified path exactly where the
token before it cannot end an operand — a comparison needs a left operand — where its `<…>` closes at its own level and
a `::` follows; an operand ends with an identifier, a literal, a closing delimiter, `?`, `self`, `Self`, `super`,
`crate`, `true`, `false` or the `await` of a postfix `.await`. A `}` ends a block-like operand as well as a statement,
and is read as an operand's end, so a comparison after a block never opens a qualified path; a qualified path opening
a statement right after a `}` is then read as a comparison, a declared over-reaction (bound:
inline-symbol-path-confinement/a-qualified-path-after-a-closing-brace-is-read-as-a-rooted-path-a-stated-bound). A `<<`
after a token that ends an operand is a shift, whatever `<…>` groups its tokens could close: a qualified path its
second `<` opens in a generic list, `Vec<<u8 as Tr>::md5x>`, is then read as a comparison's right operand, a declared
over-reaction (bound:
inline-symbol-path-confinement/a-qualified-path-a-shift-opens-in-a-generic-list-is-read-as-a-rooted-path-a-stated-bound),
since reading it as a list would drop the rooted call `1 << n > ::std::process::id() && n > 0` compares. A
qualified path's tail is left to the receiver-method bound; the paths inside its `<…>` are read as any others are. Such a
mention reacts under `.strict_prefix_only()` alone. A
path used as a type annotation, a bare constant reference, or any non-call position SHALL NOT react. This
distinction is structural — the engine keys on the presence of a call application, never on a
built-in notion of which verb is a "read". A forbidden path taken as a **value** rather than
called (`let f = std::time::SystemTime::now; f()`) is a mention under the default and is covered
only by `.strict_prefix_only()` — a stated bound (bound: inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default) (see "Observation bounds"), not a claim.

#### Scenario: An associated-function call under the prefix reacts
- **WHEN** `crate::core` contains `std::time::SystemTime::now()` and a boundary forbids inline calls under `std::time` on `crate::core`
- **THEN** the system emits a violation naming the offending call and module

#### Scenario: A turbofish holding a fn arrow is one path
- **WHEN** code calls `std::mem::size_of::<fn() -> u8>()` under a boundary forbidding inline calls under `std::mem`
- **THEN** the system reports `std::mem::size_of in crate` in either mode
- **PINNED-BY** `a_turbofish_holding_a_fn_arrow_is_one_group`

#### Scenario: A path through an enum glob names the variant
- **WHEN** `crate::s` declares `pub use crate::a::E::*;` for `pub enum E { V(u8) }` in `crate::a`, and the crate root calls `crate::s::V(1)`; or the crate root declares `use crate::a::E::*;` and calls a bare `V(1)` beside `Some(1)`, under a prefix `crate::a`
- **THEN** the system reports `crate::a::E::V in crate` beside `glob crate::a::E in crate` for the path, and only the glob's finding for the bare call, with and without `.strict_external()`: an enum has no scope the resolver reads, so its glob is read as one of a crate whose contents are not read
- **PINNED-BY** `a_path_through_an_enum_glob_names_the_variant`

#### Scenario: An angle group holding a brace or a negative is one group
- **WHEN** a crate root writes `impl Tr for S<{ 1 }> { fn exit() {} fn run() { exit(0) } }` beside `use std::process::exit;`, or `pub enum E<const N: usize = { 1 }> { exit(i32) }` beside it, under a prefix `std::process`; or `a::f::<-1>()` under a prefix `crate::a`
- **THEN** the system reports `std::process::exit in crate` for the `impl`, nothing for the `enum`, and `crate::a::f in crate` for the turbofish, with and without `.strict_external()`: a brace group inside `<…>` is a const argument or default and ends no item header, and `<-` opens a group as a `<` before a `-`
- **PINNED-BY** `an_angle_group_holding_a_brace_or_a_negative_is_one_group`

#### Scenario: A construction is a call wherever it stands
- **WHEN** a file module `m` declaring `pub struct P(pub u8)` constructs `P(1)` in a block arm's body, in a closure's body and after a bitwise `|`, governed from `crate` under `crate::m::P`
- **THEN** the system reports `crate::m::P in crate::m` in either mode, once
- **PINNED-BY** `a_construction_beside_a_pattern_position_is_a_call`

#### Scenario: A range operator leaves the path after it its head
- **WHEN** code writes `for _ in 0..std::process::id() {}`, `(0..=std::process::id()).is_empty()` or `for _ in 0..::std::process::id() {}` under a boundary forbidding inline calls under `std::process`
- **THEN** the system reports `std::process::id in crate` in either mode
- **PINNED-BY** `a_range_before_a_path_leaves_the_path_its_head`

#### Scenario: A call reports wherever an expression stands
- **WHEN** `std::process::id()` is called inside a match scrutinee that is `unsafe { … }`, `{ … }`, `loop { break … }`, `if c { … } else { 0 }`, `async { … }` or a `match` nested sixty-six deep; in a match arm's body after `convert::<u32, u64>(…)`, a `collect::<HashMap<_, _>>()`, a bare, boxed or called closure; or after `a().await |`
- **THEN** each reports `std::process::id in crate` in either mode
- **PINNED-BY** `a_call_reports_wherever_an_expression_stands`

#### Scenario: A qualified path's type is read as one group
- **WHEN** a match arm's body begins with `<HashMap<u8, u8> as Default>::default()`, `<HashMap<String, u32>>::with_capacity(…)` or `<Result<u8, u8> as Clone>::clone(…)`, and then calls `std::process::id()`
- **THEN** each reports `std::process::id in crate` in either mode
- **PINNED-BY** `a_qualified_paths_type_is_read_as_one_group`

#### Scenario: A call inside a turbofish reports
- **WHEN** code calls `v.m::<{ crate::k::n() }>()` or `f::<{ crate::k::n() }>()`, `k` declaring `pub const fn n()`, under a boundary forbidding inline calls under `crate::k`
- **THEN** each reports `crate::k::n in crate` in either mode
- **PINNED-BY** `a_call_inside_a_turbofish_reports`

#### Scenario: A float literal ending in a dot is one token
- **WHEN** code writes `1. < x && x > ::std::process::id() as f64` under a boundary forbidding inline calls under `std::process`
- **THEN** the system reports `std::process::id in crate` in either mode: `1.` is one literal, as rustc reads it
- **PINNED-BY** `a_float_literal_ending_in_a_dot_is_one_token`

#### Scenario: A dot after a suffix or a tuple index calls a method
- **WHEN** a file module `m` declaring `pub fn max(_: u8)` and `pub fn clone(_: u8)` writes `t.0. clone()` and `1u8. max(2)`, governed from `crate` under `crate::m`
- **THEN** the system reports nothing in either mode: a literal takes a `.` only straight after its digits, and the word after a tuple index's `.` is a method
- **PINNED-BY** `a_dot_after_a_suffix_or_a_tuple_index_calls_a_method`

#### Scenario: A turbofish nested in a turbofish is read as a type
- **WHEN** code writes `std::mem::size_of::<crate::k::T::<u8>>()` under a boundary forbidding inline calls under `crate::k`
- **THEN** the system reports nothing in either mode: the inner run ends inside the outer group, and the `(` after that group applies to `size_of`
- **PINNED-BY** `a_turbofish_nested_in_a_turbofish_is_read_as_a_type`

#### Scenario: A qualified path opened by a shift token keeps its tail
- **WHEN** a package depending on `md5x` writes `a::<<u8 as Tr>::md5x>()`, `Tr` declaring an associated type `md5x`, under a confinement on `md5x` with `.strict_prefix_only()` and `.strict_external()`
- **THEN** the system reports nothing: the inner `<` of `<<` opens a qualified path whose tail is left to the receiver-method bound
- **PINNED-BY** `a_qualified_path_opened_by_a_shift_token_keeps_its_tail`

#### Scenario: A comparison beside a cast or after a block opens no type position
- **WHEN** code writes `(n as u32) < m && m > ::std::process::id()`, `x as u32 > ::std::process::id()`, `0 < n && n > ::std::process::id()`, `"x" < y && 1 > ::std::process::id()`, `let b = match x { _ => 1 } < n && n > ::std::process::id();` or a scrutinee `{ 1 } < 2` before arms calling `std::process::id()`
- **THEN** each reports `std::process::id in crate` in either mode
- **PINNED-BY** `a_comparison_beside_a_cast_or_after_a_block_opens_no_type_position`

#### Scenario: A comparison after a block opens no qualified path
- **WHEN** `< n && n > ::std::process::id()` follows an `if … else`, a `loop`, a `match`, a bare block or an `unsafe` block standing in a `let` initializer or a call's argument; or `<W>::md5x()` and a call of `std::process::id()` follow twenty thousand consecutive `for` loops
- **THEN** each reports `std::process::id in crate` in either mode, and the loops are read without recursing through them
- **PINNED-BY** `a_comparison_after_a_block_opens_no_qualified_path`
- **PINNED-BY** `a_long_run_of_statements_is_read_without_recursing_through_it`

#### Scenario: An operand keyword ends an operand
- **WHEN** `true`, `false`, `self`, `Self` or a postfix `.await` is followed by `|`, `||` or `<` and then a call of `std::process::id()` — `Self` naming no `bool`, it has no `||` row — and, as controls, `self.0 |`, `self.t() ||` and `true && n < 3 && n > ::std::process::id()`; and `a().await < b && c > ::std::process::id()`
- **THEN** each reports `std::process::id in crate` in either mode
- **PINNED-BY** `an_operand_keyword_ends_an_operand`
- **PINNED-BY** `a_less_than_after_await_opens_no_qualified_path`

#### Scenario: A type annotation under the prefix passes
- **WHEN** `crate::core` contains `fn handle(ev: Event, now: std::time::Instant)` (a type annotation, no call)
- **THEN** the system reports no violation (a mention is not a call — the core may receive injected time)

#### Scenario: A deterministic constant under the prefix passes
- **WHEN** `crate::core` reads `std::time::SystemTime::UNIX_EPOCH` (a constant, no call)
- **THEN** the system reports no violation (a constant is not a call and is not an ambient read)

#### Scenario: Every Pattern_White_Space character separates tokens
- **WHEN** a `use` group writes `{c Thing c}` for each `c` of `Pattern_White_Space` — the ASCII six a vertical tab among them, and U+0085, U+200E, U+200F, U+2028, U+2029 — under a boundary forbidding the importing module to import `crate::forbidden`
- **THEN** the system reports `crate::forbidden::Thing` for each, the name read without the character: rustc 1.96.0, edition 2021, compiles each row
- **PINNED-BY** `every_pattern_white_space_character_separates_tokens`

### Requirement: Prefix resolution follows imports, type aliases and local re-exports to a fixpoint

The system SHALL resolve a written path's head to a canonical path before prefix-matching, through
the scope table's bindings — the `use` imports of the scope a path stands in (**aliases included**),
the crate's local `type` aliases, and the crate's local `pub use` re-exports — chased to a fixpoint. Each of the following SHALL
resolve under the prefix and react: a fully-qualified path (`std::time::SystemTime::now()`); a
rename (`use std::time::SystemTime as SysT; SysT::now()`); a bare path (`use std::time;
time::SystemTime::now()`); a local `type` alias (`type Clock = std::time::SystemTime; Clock::now()`);
and a local re-export used from the governed subtree, **same- or cross-module**
(`pub use std::time::SystemTime;` in one local module, then `SystemTime::now()`). An unresolved
head SHALL NOT be matched by leaf alone (that would be a false positive on a same-named local
type). Within this resolvable scope there SHALL be no false negative.

#### Scenario: A renamed alias resolves and reacts
- **WHEN** `crate::core` declares `use std::time::SystemTime as SysT;` then calls `SysT::now()`
- **THEN** the system resolves `SysT` through its `use` binding to `std::time::SystemTime` and reacts

#### Scenario: A self-prefixed use-group member resolves and reacts
- **WHEN** `crate::clock` declares a module `self_utc` and a struct `Duration`, and `crate::core` declares `use crate::clock::{self_utc as clk, Duration};` then calls `clk::now()` under a prefix `crate::clock` — a group member whose name merely *starts with* the substring `self`, not the `self` leaf
- **THEN** the system resolves `clk` to `crate::clock::self_utc` and reacts; only the exact `self` group leaf is read as the prefix module, never a legal `self`-prefixed identifier (dropping it would be a false negative — the confined call would pass unresolved)
- **PINNED-BY** `inline_resolves_a_self_prefixed_group_alias`

#### Scenario: A `{self}` group leaf binds its module
- **WHEN** `crate::core` declares `use std::io::{self, Write};` then calls `io::stdout()`, `use crate::a::{self};` then calls `a::X::f()`, or `use std::time::{self as t};` then calls `t::Instant::now()`
- **THEN** the system binds the prefix module under its last segment, or under the ` as ` alias, and reports each call under `std::io`, `crate::a` and `std::time` respectively, in either mode
- **PINNED-BY** `a_self_leaf_binds_its_module_under_its_last_segment_or_alias`

#### Scenario: A bare path resolves and reacts
- **WHEN** `crate::core` declares `use std::time;` then calls `time::Instant::now()`
- **THEN** the system resolves `time` to `std::time` and reacts

#### Scenario: A local type alias resolves and reacts
- **WHEN** `crate::core` declares `type Clock = std::time::SystemTime;` then calls `Clock::now()`
- **THEN** the system resolves `Clock` to `std::time::SystemTime` and reacts (a `type` alias is followed, not treated as an unresolved local)

#### Scenario: A cross-module local re-export resolves and reacts
- **WHEN** `crate::support` declares `pub use std::time::SystemTime;`, `crate::core` declares `use crate::support::SystemTime;` then calls `SystemTime::now()`
- **THEN** the system chases the local re-export closure to `std::time::SystemTime` and reacts

#### Scenario: A generic type alias's parameter list closes where rustc splits its last token
- **WHEN** the crate root declares `type A<T>= crate::x::Y<T>;` or `type A<T: Into<u8>>= crate::x::Y<T>;`, with no space before the `=`, or the spaced `type A<T: Into<u8>> = crate::x::Y<T>;`, and calls `A::<u8>::f()`, under a prefix `crate::x`
- **THEN** the system reports `crate::x::Y::f in crate` for each, with and without `.strict_external()`: the list closes at the `>` a `>=` or `>>=` holds, and the `=` that token also holds is the alias's
- **PINNED-BY** `a_generic_type_alias_closing_at_a_compound_token_binds_its_target`

#### Scenario: A type alias is read past a reference or pointer
- **WHEN** the crate root declares `type A = &'static crate::x::Y;`, `&'static mut crate::x::Y`, `*const crate::x::Y` or `*mut crate::x::Y`, a trait `T` implemented for that type, and calls `A::f()`, under a prefix `crate::x`
- **THEN** the system reports `crate::x::Y::f in crate`, with and without `.strict_external()`
- **PINNED-BY** `a_type_alias_is_read_past_a_reference_or_pointer`

#### Scenario: A multi-hop type alias resolves to a fixpoint
- **WHEN** `crate::core` declares `type A = std::time::SystemTime; type B = A;` then calls `B::now()`
- **THEN** the system chases `B → A → std::time::SystemTime` to a fixpoint and reacts (resolution is not single-hop)

#### Scenario: A multi-hop local re-export resolves to a fixpoint
- **WHEN** `crate::a` declares `pub use std::time::SystemTime;`, `crate::b` declares `pub use crate::a::SystemTime;`, `crate::core` uses `crate::b::SystemTime` then calls `SystemTime::now()`
- **THEN** the system chases the two-hop re-export closure to `std::time::SystemTime` and reacts

#### Scenario: An unresolved same-named local is not matched by leaf
- **WHEN** `crate::core` defines a local type `Instant` (not `std::time::Instant`) and calls `Instant::now()`, with no `use` / `type` / re-export bringing `std::time::Instant` into scope
- **THEN** the system does NOT react (leaf-only matching is rejected — it would be a false positive)

### Requirement: Inline path heads resolve from one typed lexical scope

For each recognized inline path occurrence, the system SHALL resolve the first identifier from the
occurrence's lexical scope chain — each enclosing block, then its module — using the package
edition, the namespace the head is looked up in (a head followed by `::` names a module or type, a
bare call names a value), item declarations, named imports and aliases — a `{self}` group leaf
binding its module under the last segment or its alias — the local `type`-alias and
`pub use` closure, and glob-import edges followed to a fixed point. Every binding — a `use` leaf, a
`pub use`, a glob and a `type` alias — SHALL be recorded as written, with the scope it is written in,
and SHALL be resolved from that scope by the one resolver when a lookup reaches it, never when it is
recorded. In edition 2018 and later a `use` path's first segment SHALL be looked up as a uniform path:
in the scope chain of the `use` — its block, then its module — among child modules, items, imports and
the names its globs bring, and only where nothing there binds it SHALL it name a crate, a sysroot crate
or a dependency. A `type` alias's target SHALL be read the same way, so a bare head no scope binds
names the extern-prelude crate of that name in an alias as in a `use`. Where a local name and an external crate share the head, rustc refuses the path as
ambiguous (E0659); the scanner takes the local one. A glob's finding SHALL name the path the glob
resolves to, not the path as written. Only a module body and a block
open a scope: an `impl`, `trait`, `enum`, `struct` or `union` body binds none of its members to a
bare head, and the items of an `extern` block or a `cfg_if!` arm belong to the enclosing scope. A
`use` or item written directly in a block SHALL bind for that whole block and SHALL NOT bind
outside it. A block whose only binding of a name is an import of what is not read SHALL NOT end the lookup there:
since such an import may hold the name in the other namespace alone, the name SHALL also be read from the scope
around the block, and the answers joined. A glob SHALL carry the names visible where it is written, and each binding of a name
only as far as that binding's own visibility — `pub`, `pub(crate)`, `pub(super)`, `pub(in …)` or
private — reaches the glob's module, a private import reaching a descendant through `use super::*`. In an edition-2015 package, a `use` path and a path
beginning with `::` SHALL look their head up in the crate root's scope, and name a crate where nothing
there binds it. The system SHALL distinguish a local binding from an external-prelude binding and SHALL NOT
select among distinct candidate bindings: where a scope binds a name more than once, as
cfg-exclusive imports do, the occurrence SHALL react under each candidate that resolves under the
prefix, and a candidate naming a block-local item SHALL NOT remove the others. A module scope's
candidates for a name SHALL be every binding of it together with the item it declares under it: neither
SHALL remove the other, since under exclusive cfgs either can be the live one. A crate-rooted path
SHALL name itself and every path each binding on it names: where a segment names something a module binds
rather than declares — an import, a `type` alias read in that module's own scope, or a name one of its
globs brings — every path those bindings name is read on from it, to a fixed point, so a module's
cfg-exclusive `pub use`, or a `type` alias over its cfg-exclusive imports, carries each candidate to a
call through it. A call SHALL report under every path its resolution passes: the path as written, each path a
binding or a glob rewrites it to, and — for a name a glob brings to a bare head — the glob's module followed by the
name, a path reported and not read again, since what it names is already among the candidates read where the glob
stands. A finding's identity is its path, so a call whose resolution passes two paths under one prefix is two
findings. Each segment of a crate-rooted path SHALL be looked up in the module the segments before it
reach by the same module-level lookup a head written in that module gets, glob visibility judged from the
module the glob is written in: a declared module, or the crate an `extern crate self` names, is walked into; a
declared item names itself followed by the rest of the path; a name one of the module's globs brings is a
binding of that module; and a segment only a glob of a crate whose contents are not read can bring names that
glob's path followed by the segment. A bare head reads no such foreign-glob candidate, since a prelude name or
a local binding may answer it instead, and the glob's own finding is what reacts. A scope SHALL record every
declaration of a name — under exclusive cfgs a name is declared more than once, and a type and a value may
share one — each with its namespaces, its own visibility and its kind: a module, an `extern crate`, or another
item. A lookup SHALL read the declarations and the bindings of the namespace it is made in, a `use` leaf and a path
mentioned without being called being read in each namespace and joined, so a value `fn X` beside a glob bringing a
struct `X` leaves the struct to a type lookup. A binding SHALL hold its name only in the namespaces its target
provides — a `type` alias names a type, and an import holds each namespace its target names something in, and both
where its target is known in neither, as an item a macro generates is, or where the target's module holds a macro
invocation standing where an item can — one inside an item, as a `const`'s initializer, generates none — that may
generate the namespace its target is not found in — so a scope's globs are read for a namespace
no binding and no declaration there holds; and a glob SHALL bring each declaration only where that declaration's own
visibility reaches the glob's module. An `extern crate` SHALL bind its name, or its `as` alias, to the crate it
names — `crate` for `self` — as an item of its module, and one at the crate root SHALL bind it in every module of
the unit after that module's own scopes. A module declared in a block SHALL be resolved through as a module: a
path through it names what its bindings name, and an item it declares names nothing a prefix can reach, since no
path outside the block names it — a block is named by its file as well as its place in it, so two cfg-exclusive
files of one module hold two blocks — and a `super` written in it SHALL name the module the block stands in — the block
is not a module, as rustc resolves it. Only a block's item that is neither a module nor an `extern crate` SHALL be read
as block-local. A `use` a block inside a macro's group holds SHALL bind that block's paths, as the expansion that
emits the block does; a `use` written directly in the group binds in no scope, a declared bound, since where the
macro expands it is not read. An item SHALL start after a `;`, a brace group, a braced macro invocation or an attribute, so an item
written right after `thread_local! { … }` is declared, while a macro invocation that does not stand as an item — the
self type of `impl Tr for t!() { … }` or of `impl Tr for t!{} { … }` — continues the header it stands in and is
never its body. An item stands only at a file's top level or directly inside a brace group, and a macro invocation
stands as an item only there, after a `;`, a brace or an attribute, so one in an initializer, an array's elements
or a call's arguments generates no item. A `const` SHALL be an item where a name
and `:` follow it, whatever the name, so `pub const safe: u8` is declared, and otherwise it qualifies what follows. A glob of a module declared in a block names no path a finding can carry, so it reacts through
the calls it brings and through the globs written inside that module, never as a hazard of its own. Every written path's first segment — of a `use` leaf, a glob, a `type` alias's target
or an occurrence — SHALL be classified by one dispatch into a crate-rooted path, an external crate's
root, a head looked up in the crate root's scope, or a head looked up in the path's own scope, so a
block-local alias whose target begins with `::` names what the same target names anywhere else; a
`std`, `core` or `alloc` head no scope binds names the sysroot crate, and so does a `proc_macro` head in a proc-macro crate, whose extern prelude holds it without an `extern crate`. The tail of a qualified path is not resolved, and which `<` opens one is the one
judgement the "Call-vs-mention default" requirement states; such a tail is covered by the receiver-method
observation bound. A head no scope binds SHALL name no item of the module it stands in, since the scope table
records every item a module declares: such a head is a prelude name, a local binding, an attribute's name
or an item a macro generates, and names nothing a prefix can reach — under `.strict_external()` it names
a crate where it matches a declared dependency. An attribute's own path (`#[…]` or `#![…]`, each meta a
`cfg_attr` applies included) and a `cfg` or `cfg_attr` predicate SHALL hold no occurrence, and what else an attribute
holds SHALL be read as any other tokens are — an attribute macro reads its arguments as its own input, and a
derive's path is a mention of the crate it names. A lint tool's path, `#[allow(clippy::all)]`, is read the same way and
reports nothing by construction rather than by a rule of attributes: its head names no crate unless a dependency
carries the tool's name. A keyword of the target's edition written without `r#` SHALL
begin no path, except the path keywords `crate`, `self`, `super` and `Self`, which begin one. A `fn` item's own name is its definition and SHALL NOT be read as a call, and nor SHALL the
name a tuple struct or a tuple variant declares; constructing one is a call. Resolution SHALL answer
with a typed result, and SHALL NOT itself decide an exit code: a chain it walks past the nesting cap is
one of its answers, which the scan refuses as the "pathologically nested" requirement states.

The confined prefix SHALL be normalized once into its canonical form — segments without a leading
`::` and without `r#` — and that one value SHALL be what the rule key, a violation's target and the
call matcher read, so `std::time` and `::std::time` are one identity. Default and strict-external
observation policy SHALL be applied after resolution: an un-`use`d external dependency call remains
outside the default policy and is observed under strict-external, whether it is written
`dep::item()` or `::dep::item()`.

#### Scenario: A parenthesized alias of a generic path is read through it
- **WHEN** the crate root writes `pub mod secret { pub struct G<T>(pub T); impl<T> G<T> { pub fn make() {} } }` and `pub fn g() { type C = (crate::secret::G<u8>); C::make(); }`, under a prefix `crate::secret`
- **THEN** the system reports `crate::secret::G::make in crate`, with and without `.strict_external()`: a parenthesized target is read with the generic arguments its path takes, and rustc 1.96.0, edition 2021, calls `crate::secret::G::make`
- **PINNED-BY** `a_parenthesized_alias_of_a_generic_path_is_read_through_it`

#### Scenario: A self leaf imports its module and no value of that name
- **WHEN** the crate root writes `pub mod local { pub mod both {} pub fn both() {} }`, `use crate::secret::both;` beside `pub mod secret { pub fn both() {} }`, and `pub fn g() { use crate::local::both::{self}; both(); }`, under a prefix `crate::secret`
- **THEN** the system reports `crate::secret::both in crate`, with and without `.strict_external()`: a `{self}` leaf imports the module its group names and holds no value, so rustc 1.96.0, edition 2021, calls the module's import
- **PINNED-BY** `a_self_leaf_imports_its_module_and_no_value_of_that_name`

#### Scenario: A block's alias of no path is a block-local item
- **WHEN** the crate root writes `use std::process::Command;` and `pub fn g() { type Command = (u8, u8); let _ = Command::default(); }`, or the same with `[u8; 2]`, or with `type Command = std::process::Command;`, or `type Command = (std::process::Command);`, and `Command::new("true")`, under a prefix `std::process`
- **THEN** the system reports nothing for the tuple and the array, which name no path and are items of the block, and `std::process::Command::new in crate` for the path, parenthesized or not: rustc 1.96.0, edition 2021, calls the alias's `default` in the first two
- **PINNED-BY** `a_block_alias_of_no_path_is_a_block_local_item`

#### Scenario: A block's import of what is not read leaves the name to the scope around it
- **WHEN** the crate root writes `pub mod forbidden { pub fn fmt() {} }`, `use crate::forbidden::fmt;` and `pub fn g() { use std::fmt; fmt(); }`, or the same with `use std::fmt::{self};`, under a prefix `crate::forbidden`
- **THEN** the system reports `crate::forbidden::fmt in crate` for each, with and without `.strict_external()`: `std::fmt` names a module and no value, and rustc 1.96.0, edition 2021, calls the module's import
- **PINNED-BY** `a_block_import_of_what_is_not_read_leaves_the_name_to_the_scope_around_it`

#### Scenario: Every binding shape reads alike through a lexical head and a path segment
- **WHEN** `X` is reached through a named `use`, a `pub use` re-export, a glob, a `type` alias, cfg-exclusive re-exports, a `{self}` leaf, a module declared in a block, `extern crate self as me;` at the crate root, or a glob beside a same-named value `fn X`; or `X` is a type from a binding and a value from a glob, or a value from a binding and a type from a glob; and one file module calls `X` through a name that shape binds while another writes the path through it — `crate::…::X`, or for the block's module `m::X`
- **THEN** each cell reports what its namespace names — `crate::a::X::f` under `crate::a`, the cfg-exclusive shape under `crate::b` as well, and `crate::v::X` under `crate::v` for the call `X()` of the value a glob brings — in its own module, in either mode
- **PINNED-BY** `every_binding_shape_resolves_a_lexical_head_and_a_path_segment_alike`

#### Scenario: A path through a glob re-export names the item the glob brings
- **WHEN** `support` is `pub use crate::a::*;` and `crate::core2`, in its own file, writes `use crate::support::X; X::f()`, or the crate root writes that or `crate::support::X::f()`
- **THEN** the system reports `crate::a::X::f` in the calling module under `crate::a` in either mode, beside `glob crate::a in crate` where the glob is governed
- **PINNED-BY** `a_path_through_a_glob_reexport_names_the_item_the_glob_brings`

#### Scenario: A call reports under every path its resolution passes
- **WHEN** `support` holds `pub use crate::a::X;`, `pub use crate::a::*;` or `pub type X = crate::a::X;`, and the crate root writes `use crate::support::X; X::f()` or `crate::support::X::f()`; or `support` re-exports `crate::a::X` and the crate root writes `use crate::support::*; X::f()`
- **THEN** the call reports `crate::support::X::f in crate` under `crate::support` and `crate::a::X::f in crate` under `crate::a`, beside the governed glob's finding under each prefix its glob reaches, in either mode
- **PINNED-BY** `a_call_reports_under_every_path_its_resolution_passes`
- **PINNED-BY** `a_name_a_glob_brings_reports_under_the_globs_module_too`

#### Scenario: One call through two paths under one prefix is two findings
- **WHEN** `crate::p::support` re-exports its sibling `crate::p::a`'s `X`, and the crate root calls `crate::p::support::X::f()` under a prefix `crate::p`
- **THEN** the system reports `crate::p::a::X::f in crate` and `crate::p::support::X::f in crate`, in either mode
- **PINNED-BY** `one_call_through_two_paths_under_one_prefix_is_two_findings`

#### Scenario: A binding holds only the namespaces its target provides
- **WHEN** `support` holds `pub type X = crate::a::X;` beside `pub use crate::v::*;`, whose `v` declares `fn X`, and `crate::core2` writes `use crate::support::X; X();`; or `support` holds `pub use crate::v::X;` beside `pub use crate::a::*;` and `crate::core2` writes `use crate::support::X; X::f();`
- **THEN** the system reports `crate::v::X in crate::core2` under `crate::v`, and `crate::a::X::f in crate::core2` under `crate::a`, in either mode; and where `support` imports `X` from `crate::v`, a function, under one cfg and from `crate::w`, a struct, under the other, `X::f()` reports `crate::w::X::f in crate::core2` under `crate::w`
- **PINNED-BY** `a_binding_holds_only_the_namespaces_its_target_provides`
- **PINNED-BY** `each_import_of_a_name_holds_the_namespaces_of_its_own_target`

#### Scenario: An import holds a namespace its target's macro may generate
- **WHEN** `crate::c` declares `pub fn X()` and invokes a macro — as `gen!();`, `self::gen!();` or `crate::c::gen! {}` — that generates a braced `struct X` with an associated `f`, and `crate::core2` writes `use crate::c::X; X::f();`
- **THEN** the system reports `crate::c::X::f in crate::core2` under `crate::c` in either mode
- **PINNED-BY** `an_import_holds_a_namespace_its_targets_macro_may_generate`

#### Scenario: A macro inside an item generates no namespace
- **WHEN** `crate::c` declares `pub fn X()` and a macro invocation inside an item — `pub const S: usize = concat!("ab").len();`, in an array's elements, an array type's length or a call's argument — `support` holds `pub use crate::c::X;` beside `pub use crate::p::*;` whose `p` declares `struct X` with an associated `f`, and `crate::core2` writes `use crate::support::X; X::f();`
- **THEN** the system reports `crate::p::X::f in crate::core2` under `crate::p` in either mode
- **PINNED-BY** `a_macro_inside_an_item_generates_no_namespace`
- **PINNED-BY** `a_macro_inside_any_item_generates_no_namespace`

#### Scenario: A path through a foreign glob names the foreign path
- **WHEN** `support` is `pub use std::process::*;` and the crate root writes `use crate::support::Command; Command::new("x")`
- **THEN** the system reports `std::process::Command::new in crate` beside `glob std::process in crate` in either mode
- **PINNED-BY** `a_path_through_a_foreign_glob_names_the_foreign_path`

#### Scenario: A declaration binds its name in its own namespace only
- **WHEN** `support` declares `pub fn X() {}` beside `pub use crate::a::*;`, and `crate::core2` writes `use crate::support::X; X::f()`
- **THEN** the system reports `crate::a::X::f in crate::core2` in either mode
- **PINNED-BY** `a_value_item_does_not_hide_a_type_a_glob_brings`

#### Scenario: Each declaration keeps its own visibility
- **WHEN** `crate::a` declares `#[cfg(unix)] pub struct X;` and `#[cfg(not(unix))] struct X;`, in either order, `crate::r` is `pub use crate::a::*;`, and `crate::user` writes `use crate::r::X; X::f()`
- **THEN** the system reports `crate::a::X::f in crate::user` in either mode
- **PINNED-BY** `cfg_exclusive_declarations_keep_each_visibility`

#### Scenario: An edition-2015 root glob brings the crate root's names
- **WHEN** an edition-2015 crate root declares `pub use std::time;` and `mod m { use ::*; … time::Instant::now() }`, or the same with `use *;` or `use {*};`, under a prefix `std::time` over `crate`
- **THEN** the system reports `std::time::Instant::now in crate` beside `glob crate in crate` for each, with and without `.strict_external()`: a `use` path starts at the crate root in edition 2015, so each spelling globs it
- **PINNED-BY** `an_edition_2015_root_glob_brings_the_crate_roots_names`

#### Scenario: An extern crate binds its name or its alias
- **WHEN** the crate root declares `extern crate self as me;` and calls `me::clock::now()`; or `crate::core`, or the crate root, declares `extern crate md5x as chr;` and `crate::core` calls `chr::compute()`; or the crate root declares `extern crate md5x;` and `crate::core` calls `md5x::compute()`, or in edition 2015 also `::md5x::compute()`
- **THEN** the system reports `crate::clock::now in crate` under `crate::clock`, and `md5x::compute in crate::core` under `md5x`, in either mode
- **PINNED-BY** `an_extern_crate_self_alias_names_the_crate_root`
- **PINNED-BY** `an_extern_crate_names_its_crate_under_its_name_or_alias`

#### Scenario: A module declared in a block is resolved through
- **WHEN** a function body declares `mod m { pub use std::fs::read; }` and calls `m::read("x")`, or `use m::*; read("x")`; and, as the control, a function body declares `struct L; impl L { fn f() {} }` and calls `L::f()` under a prefix covering its module
- **THEN** the system reports `std::fs::read in crate` alone under `std::fs` in either mode, and nothing for `L::f()`
- **PINNED-BY** `a_block_local_module_is_resolved_through`

#### Scenario: Blocks of two files of one module are two blocks
- **WHEN** `crate::a` is backed by cfg-exclusive `#[path]` files, `a_unix.rs` writing `pub fn g() { mod m { pub use std::fs::read; } let _ = m::read("x"); }` and `a_other.rs` a function body declaring no module
- **THEN** the system reports `std::fs::read in crate::a` under `std::fs` in either mode
- **PINNED-BY** `blocks_of_two_files_of_one_module_are_two_blocks`

#### Scenario: A super path in a module declared in a block names the enclosing module
- **WHEN** `crate::core` writes `use crate::a::X;` and a function body declaring `mod m { pub fn h() -> u8 { super::X::f() } }`
- **THEN** the system reports `crate::a::X::f in crate::core` under `crate::a` in either mode
- **PINNED-BY** `a_super_path_in_a_module_declared_in_a_block_names_the_enclosing_module`

#### Scenario: An item after a macro invocation is an item
- **WHEN** `crate::core` writes `thread_local! { static T: u8 = 0; }` and then `mod inner { pub use crate::a::X; }` and calls `inner::X::f()`; and, as the control, the same with `const T: u8 = 0;` in the macro's place
- **THEN** each reports `crate::a::X::f in crate::core` under `crate::a` in either mode
- **PINNED-BY** `an_item_after_a_macro_invocation_is_an_item`

#### Scenario: An impl for a macro type binds none of its members
- **WHEN** a crate writes `use std::process::exit;` and `impl Tr for t!() { fn exit() {} fn g() { exit(3) } }`, `t!()` expanding to a struct; or `use crate::k::g;` and `impl crate::Tr2 for t!{} { fn g() {} fn h() { g() } }`
- **THEN** the system reports `std::process::exit in crate` under `std::process`, and `crate::k::g in crate` under `crate::k`, in either mode
- **PINNED-BY** `an_impl_for_a_macro_type_binds_none_of_its_members`
- **PINNED-BY** `an_impl_for_a_braced_macro_type_binds_none_of_its_members`

#### Scenario: A const named like a qualifier is declared
- **WHEN** a crate declares `pub const safe: u8 = 1;` and `pub const default: u8 = 2;`, and a boundary names `crate::safe` or `crate::default` as its prefix
- **THEN** the prefix is accepted and the system reports nothing
- **PINNED-BY** `a_const_named_like_a_qualifier_is_declared`

#### Scenario: A private import inherited through a parent glob resolves in an inline test
- **WHEN** `crate::clock` privately imports `std::time::SystemTime`, and its inline test module, through `use super::*` or two levels down through `use super::super::*`, calls `SystemTime::now()`
- **THEN** the system resolves the head to `std::time::SystemTime` through the parent scope and reports `std::time::SystemTime::now in crate::clock`
- **PINNED-BY** `inline_private_use_inherited_through_super_glob_resolves`

#### Scenario: An inline module use resolves from its enclosing module
- **WHEN** a nested inline module imports `crate::clock::now` from its actual ancestor scope and calls `now()` under a boundary on `crate::core`
- **THEN** the system resolves the alias to `crate::clock::now` and reports the call
- **PINNED-BY** `an_inline_module_use_resolves_from_its_enclosing_module`

#### Scenario: An inline use does not leak to a sibling
- **WHEN** one inline child imports `crate::clock::now` and a sibling defines and calls its own `now`
- **THEN** the system keeps the two lexical scopes distinct and reports no violation for the sibling
- **PINNED-BY** `an_inline_module_use_does_not_leak_to_sibling_modules`

#### Scenario: A block-local use binds for its whole block and only inside it
- **WHEN** `crate::core` imports `crate::a::X` at module level, one function calls `X::fa()`, and another declares `use crate::b::X;` and calls `X::fb()`, in any textual order; or a block calls `Command::new("x")` before its own `use std::process::Command`; or a char literal `'{'`, a raw string `r#"}"#`, a byte literal `b'}'` or a lifetime stands in the block
- **THEN** under a prefix `crate::a` the system reports `crate::a::X::fa` only and under `crate::b` `crate::b::X::fb` only, the call before the block's `use` is reported through it, and no literal's brace opens or closes a scope
- **PINNED-BY** `inline_block_local_use_binds_only_inside_its_block`
- **PINNED-BY** `a_block_local_use_resolves_inside_its_block`
- **PINNED-BY** `a_fn_local_use_shadows_a_module_use_of_the_same_name`
- **PINNED-BY** `a_block_local_use_covers_text_before_it`
- **PINNED-BY** `a_brace_in_a_literal_or_beside_a_lifetime_opens_no_scope`

#### Scenario: A block-local item or alias binds only inside its block
- **WHEN** `crate::core` imports `std::process::Command` and a function body declares its own `struct Command` and calls `Command::new("x")`; or a function body declares `type Clock = std::time::SystemTime;` beside a module-level `struct Clock` called elsewhere as `Clock::tick()`
- **THEN** the system resolves the block's head to the block's item and reports no violation under `std::process`, and reports only the block's `std::time::SystemTime::now` under `std::time`, never the module's `Clock::tick()`
- **PINNED-BY** `inline_block_local_item_shadows_a_module_import`
- **PINNED-BY** `a_block_local_type_alias_does_not_bind_outside_its_block`

#### Scenario: A member of a body that opens no scope binds no bare head
- **WHEN** `crate::core` imports `std::process::Command` and calls `Command::new("x")` inside a method of `impl S { const Command: u8 = 0; … }`, beside `pub enum E { Command }`, or inside a trait impl declaring `type Command = u8;`; or a trait impl declares `type Item = std::time::SystemTime;` beside a module-level `Item` called as `Item::tick()`
- **THEN** the system reports `std::process::Command::new in crate::core` in the first three, and nothing under `std::time` in the last
- **PINNED-BY** `inline_associated_item_or_variant_does_not_shadow_an_import`
- **PINNED-BY** `an_associated_type_does_not_bind_a_bare_head`

#### Scenario: An extern block's and a cfg_if arm's items belong to the enclosing scope
- **WHEN** a function body declares `extern "C" { fn Command(); }` and calls `Command()`, and a `cfg_if!` holds `use crate::u::Y;` in one arm and `use crate::w::Y;` in the other before `Y::b()`
- **THEN** `Command()` names the block's foreign item, and `Y` names both `crate::u::Y` and `crate::w::Y` at module scope
- **PINNED-BY** `member_bodies_bind_nothing_and_pass_through_bodies_bind_outward`

#### Scenario: An inline type alias resolves at its inline path
- **WHEN** an inline child defines a type alias to `std::process::Command` and an occurrence calls the alias's `new`
- **THEN** the system resolves the alias to `std::process::Command` and reports the call
- **PINNED-BY** `an_inline_module_type_alias_resolves_under_its_inline_path`

#### Scenario: A local public re-export resolves across modules
- **WHEN** `crate::support` declares `pub use std::time::SystemTime`, `crate::core` imports that name, and calls `SystemTime::now()`
- **THEN** the system resolves the public re-export closure to `std::time::SystemTime` and reports the call
- **PINNED-BY** `inline_resolves_a_cross_module_local_reexport`

#### Scenario: A glob brings only the names visible where it is written
- **WHEN** `crate::core` globs `crate::a::support`, which holds a private `Hidden`, a `pub(super)` `Near`, a `pub(in crate::a)` `Far`, a public `helper`, a `pub(in crate)` `wide` and a `pub(crate)` `crate_wide`, and globs `crate::clocks`, which holds public `Hidden`, `Near` and `Far`, then calls `helper()`, `wide()`, `crate_wide()`, `Hidden::now()`, `Near::now()` and `Far::now()`
- **THEN** under a prefix `crate::a::support` the system reports only `helper`, `wide` and `crate_wide` beside the glob hazard, and under `crate::clocks` the three `now` calls
- **PINNED-BY** `a_glob_brings_only_the_names_visible_where_it_is_written`

#### Scenario: Every binding of a name is a candidate
- **WHEN** `crate::core` writes `#[cfg(unix)] use crate::a::X;` and `#[cfg(not(unix))] use crate::b::X;` and calls `X::f()`; or a block holds `#[cfg(unix)] type X = L;` for a block-local `L` beside `#[cfg(not(unix))] use crate::a::X;`
- **THEN** the system reports the call under a prefix `crate::a` and under a prefix `crate::b` in the first, and under `crate::a` in either order in the second, since cfg-gated source is observed as written and a candidate naming a local item removes no other
- **PINNED-BY** `inline_cfg_alternative_uses_are_both_observed`
- **PINNED-BY** `a_block_candidate_naming_a_local_item_keeps_the_other_candidates`

#### Scenario: A `use` path's head is looked up in the scope of the `use`
- **WHEN** a module writes `use exec::Command;`, `use exec::{Command, Stdio};`, `pub use exec::Command;` or `use std2::Command;` beside a child module `exec` or `std2` that re-exports `std::process::Command`, whether the child is inline or `mod exec;` in its own file, or writes `use inner::X;` beside `pub mod inner`, `use m::inner; use inner::X;`, or `use std::process; use process::Command;`; or writes any of these `use`s in a function body
- **THEN** the system resolves the head from the scope of the `use` and reports the call through the name it binds — `std::process::Command::new`, `std::process::Stdio::null` or `crate::m::inner::X::f` — in either mode
- **PINNED-BY** `a_uniform_use_path_through_a_child_module_in_another_file_resolves`
- **PINNED-BY** `a_uniform_use_path_through_a_child_module_resolves`
- **PINNED-BY** `a_uniform_use_path_names_a_child_modules_item`
- **PINNED-BY** `a_block_uniform_use_path_names_a_child_modules_item`
- **PINNED-BY** `a_uniform_use_path_through_an_imported_module_resolves`
- **PINNED-BY** `a_uniform_pub_use_reexport_resolves_through_a_private_child`
- **PINNED-BY** `a_uniform_use_path_through_an_imported_std_module_resolves`
- **PINNED-BY** `a_uniform_use_group_resolves_each_leaf`
- **PINNED-BY** `a_block_uniform_use_path_through_a_child_module_resolves`
- **PINNED-BY** `a_uniform_use_path_head_named_like_a_crate_resolves_locally`
- **PINNED-BY** `uniform_path_controls_keep_their_answers`

#### Scenario: A `type` alias's target names a dependency as a `use` path does
- **WHEN** the package depends on `md5x`, nothing local is named `md5x`, and the crate root writes `type T = md5x::W;` and calls `T::f()`
- **THEN** the system reports `md5x::W::f in crate` under a prefix `md5x` in either mode, as it does for `use md5x::W as T;`
- **PINNED-BY** `a_type_alias_to_an_unused_dependency_names_the_dependency`

#### Scenario: A glob's finding names the path it resolves to
- **WHEN** `crate::m` writes `use inner::*;` beside `pub mod inner` holding `X`, or `use exec::*;` beside `pub mod exec { pub use crate::a::X; }`, and calls `X::f()`; or an inline test writes `use super::*;` or `use super::super::*;`
- **THEN** the glob is reported as `glob crate::m::inner in crate` or `glob crate::m::exec in crate` beside the call `crate::m::inner::X::f in crate` or `crate::a::X::f in crate`, and a `super` glob as the module it names
- **PINNED-BY** `a_uniform_glob_path_resolves_and_its_call_reports`
- **PINNED-BY** `a_uniform_glob_of_a_reexporting_child_reports_the_call`
- **PINNED-BY** `a_nested_inline_glob_resolves_to_its_true_ancestor`
- **PINNED-BY** `file_module_self_and_super_globs_keep_their_resolution`

#### Scenario: A cfg-exclusive re-export or alias one module away carries every candidate
- **WHEN** `crate::support` writes `#[cfg(unix)] pub use crate::a::X;` beside `#[cfg(not(unix))] pub use crate::b::X;`, or imports `X` from each of them under those cfgs and publishes `pub type Y = X;`, and the crate root imports `crate::support::X` or `crate::support::Y` and calls `X::f()` or `Y::f()`, in either order of the two
- **THEN** the system reports the call under a prefix `crate::a` and under a prefix `crate::b`, in either mode
- **PINNED-BY** `a_cfg_exclusive_reexport_reports_under_each_candidate`
- **PINNED-BY** `a_module_alias_of_cfg_exclusive_imports_reports_under_each_candidate`

#### Scenario: Edition-2015 root paths resolve from the crate root
- **WHEN** an edition-2015 package declares `mod clock` and `fn now` at its root, and its modules call `::clock::now()`, write `use clock::now;` and call `now()`, write `use now;` and call `now()`, or call `::now()`; or a function body writes `type K = ::clock::C;` and calls `K::now()`
- **THEN** the system reports the calls under a prefix `crate::clock` or `crate::now` respectively
- **PINNED-BY** `inline_edition_2015_root_paths_resolve_from_the_crate_root`
- **PINNED-BY** `inline_edition_2015_root_paths_reach_a_crate_root_item`
- **PINNED-BY** `an_edition_2015_root_qualified_block_alias_resolves_from_the_crate_root`

#### Scenario: A block-local alias whose target begins with `::` names its root
- **WHEN** a function body writes `type Clock = ::std::time::SystemTime;` and calls `Clock::now()`, or `type D = ::core::time::Duration;` and calls `D::from_secs(1)`
- **THEN** the system reports `std::time::SystemTime::now in crate` under `std::time` and `core::time::Duration::from_secs in crate` under `core::time`, in either mode, as it does for a module-level alias with that target and a direct `::std::time::SystemTime::now()`
- **PINNED-BY** `a_root_qualified_block_alias_resolves_to_std`
- **PINNED-BY** `a_root_qualified_block_alias_resolves_to_core`
- **PINNED-BY** `a_root_qualified_path_outside_a_block_alias_reacts`

#### Scenario: A local same-name module shadows an external dependency
- **WHEN** the crate root defines `mod md5x` returning a local `md5x::Local`, the package also depends on an external `md5x` returning a different type, and code calls bare `md5x::compute()`
- **THEN** the system resolves the bare head to the local module and does not match an external-only `::md5x` prefix in either mode
- **PINNED-BY** `inline_bare_local_module_shadows_same_named_dependency`

#### Scenario: An un-used dependency has the same answer under both root spellings
- **WHEN** the package depends on `md5x` and writes `md5x::compute()` or `::md5x::compute()` without a `use`, under a prefix `md5x` or `::md5x`, or writes `::md5x::compute()` beside a crate-root `mod md5x`
- **THEN** both paths resolve to the same external binding; default mode reports neither, and strict-external reports each as `md5x::compute`, the leading `::` naming the dependency rather than the local module
- **PINNED-BY** `inline_external_dependency_root_spelling_is_mode_invariant`
- **PINNED-BY** `a_leading_colon_names_the_dependency_not_the_same_named_module`

#### Scenario: Both root spellings of a prefix share one baseline identity
- **WHEN** a baseline is recorded for a call under a prefix `::std::time` and the boundary is then declared as `std::time`, or the reverse
- **THEN** the baseline suppresses the same finding under the other spelling
- **PINNED-BY** `a_root_qualified_prefix_shares_its_baseline_identity`

#### Scenario: A proc-macro crate names proc_macro without an extern crate
- **WHEN** a `[lib] proc-macro = true` crate calls `proc_macro::TokenStream::new()` in a module outside `crate::allowed`, with and without `extern crate proc_macro;`, under `confine_inline_call("proc_macro::TokenStream")` over `crate::allowed`
- **THEN** the system reports `proc_macro::TokenStream::new in crate` for each: rustc 1.96.0, edition 2021, puts `proc_macro` in a proc-macro crate's extern prelude, and reads the call as the crate's either way
- **PINNED-BY** `a_proc_macro_crate_names_proc_macro_without_an_extern_crate`

#### Scenario: A sysroot call reacts under either root spelling of its prefix
- **WHEN** `crate::core` calls `std::time::SystemTime::now()`, written in full, under a boundary declaring its prefix as `std::time` or as `::std::time`
- **THEN** the system reports the call under either spelling of the prefix, in either mode
- **PINNED-BY** `inline_relative_and_root_sysroot_prefixes_react_on_a_qualified_call`

#### Scenario: A qualified associated path is not treated as an extern root
- **WHEN** code declares `impl W { fn md5x() {} }` and calls `<W>::md5x()`, or writes `return <W>::md5x();`, under a `md5x` external-prefix boundary
- **THEN** the system does not treat the `::` after `>` as a leading root and reports no violation in either mode
- **PINNED-BY** `inline_associated_path_after_angle_close_is_not_global_root`
- **PINNED-BY** `a_qualified_path_after_a_keyword_is_not_a_global_root`

#### Scenario: A literal is an operand
- **WHEN** code writes a string, raw, byte, raw byte, C-string, char, escaped char or byte literal before `< y && n > ::std::process::id()`
- **THEN** the system reads the literal as an operand, the `<` as a comparison and the `::` after the `>` as a root, and reports `std::process::id in crate` in either mode; a suffixed number, a method call on a byte string, a `?` and a lifetime before a byte literal do the same, and a `<W>` after a statement ending in a literal still opens a qualified path
- **PINNED-BY** `a_literal_before_a_comparison_is_an_operand`
- **PINNED-BY** `operands_and_qualified_paths_beside_literals_keep_their_answers`

#### Scenario: A multi-byte operator is one token
- **WHEN** code writes any multi-byte operator of the Reference's punctuation table that stands between two operands — `&&`, `||`, `<<`, `>>`, `==`, `!=`, `>=`, `<=`, `..`, `..=` — before a rooted call or between a `<` and a `>` a rooted call follows, or any compound assignment of a call; or writes `a || check(1)` in a `let` initializer or in a closure's body, `b || n < 3 && n > ::std::process::id()`, or `1 << n > ::std::process::id()` — alone, before `&& n > 0`, as a tuple's first element or with an `as` cast, each of which puts a later `>` at its level
- **THEN** the system reads each operator as one token and reports the call in either mode; a bitwise `|`, `&&`, an empty closure `|| …`, `|=`, `>>` and `<=` keep their answers
- **PINNED-BY** `a_multi_byte_operator_is_one_token`
- **PINNED-BY** `a_logical_or_before_a_call_reports`
- **PINNED-BY** `a_logical_or_before_a_rooted_call_reports`
- **PINNED-BY** `a_comparison_then_a_logical_or_before_a_rooted_call_reports`
- **PINNED-BY** `a_logical_or_then_a_comparison_pair_reports`
- **PINNED-BY** `a_shift_before_a_comparison_with_a_rooted_call_reports`
- **PINNED-BY** `a_logical_or_in_a_let_initializer_reports`
- **PINNED-BY** `a_logical_or_in_a_closure_body_reports`
- **PINNED-BY** `a_bitwise_or_before_a_rooted_call_reports`
- **PINNED-BY** `a_logical_and_before_a_rooted_call_reports`
- **PINNED-BY** `a_call_in_a_parameterless_closure_body_reports`
- **PINNED-BY** `a_compound_bitwise_or_assignment_of_a_call_reports`
- **PINNED-BY** `a_shift_right_before_a_rooted_call_reports`
- **PINNED-BY** `a_less_or_equal_then_a_comparison_pair_reports`

#### Scenario: A comparison opens no angle group
- **WHEN** code writes `0 < n && n > ::std::process::id()`, or `f::<u32>() > ::std::process::id()`, under a boundary forbidding inline calls under `std::process`
- **THEN** the system reads the `::` after the comparison's `>` as the root of a path and reports `std::process::id in crate` in either mode
- **PINNED-BY** `a_comparison_before_a_root_qualified_call_opens_no_angle_group`
- **PINNED-BY** `a_root_qualified_path_outside_a_block_alias_reacts`

#### Scenario: A fn item's name is its definition, not a call
- **WHEN** a nested `fn md5x()`, an associated `fn md5x()` or a trait's `fn md5x();` is declared under a single-segment `md5x` prefix with strict-external on
- **THEN** the system reports nothing
- **PINNED-BY** `a_fn_name_is_its_definition_not_a_call`

#### Scenario: A head no scope binds names no item of its module
- **WHEN** a file module `m`, governed from `crate` under a prefix `crate::m`, holds `#[cfg(any(unix, not(windows)))]`, `#[derive(Clone, Debug)]`, a `Fn(u8) -> u8` bound, a parameter `f` called as `f(1)`, a `fn(u8) -> u8` type, and bare calls of `Ok`, `Some` and `drop`; or the same bare calls stand in another module `crate::api`, governed under a prefix `crate::api`
- **THEN** the system reports nothing in either mode
- **PINNED-BY** `an_unbound_bare_head_names_no_item_of_its_module`
- **PINNED-BY** `a_prelude_name_called_bare_names_no_item_of_its_module`
- **PINNED-BY** `prelude_names_outside_the_prefixs_module_report_nothing`

#### Scenario: An attribute's name and predicate, a bare keyword and a tuple declaration hold no call
- **WHEN** a module declares `pub fn cfg() {}` beside `#[cfg(unix)]` under a prefix naming that function; a package depending on `serde` writes `#[cfg_attr(any(), serde(default))]` under a strict-external prefix `serde`; a module declares `pub fn r#match()` and writes `match (x) { … }`; or a module declares `pub struct P(pub u8);` and `pub enum E { A(u8) }` beside `pub use self::E::A;`
- **THEN** the system reports nothing in either mode; in edition 2015, where `try` is an identifier, `try()` beside `pub fn try()` reports the call; and constructing `P(1)` or `E::A(1)` reports it
- **PINNED-BY** `an_attributes_name_and_predicate_hold_no_call`
- **PINNED-BY** `a_keyword_is_never_a_path_head`
- **PINNED-BY** `a_tuple_struct_or_variant_declaration_is_not_a_call`
- **PINNED-BY** `a_tuple_struct_or_variant_construction_is_a_call`

#### Scenario: A call in an attribute's arguments reports
- **WHEN** a crate writes `#[attr::instrument(fields(t = std::process::id()))]` on a function, `attr` a proc-macro dependency, beside `#[cfg_attr(all(), derive(Debug))]`, under a boundary forbidding inline calls under `std::process`
- **THEN** the system reports `std::process::id in crate` in either mode
- **PINNED-BY** `a_call_in_an_attributes_arguments_reports`

#### Scenario: A derive path is a mention
- **WHEN** a package depending on the proc-macro crate `serx` writes `#[derive(serx::Ser)]` on a struct
- **THEN** a prefix `serx` reports `serx::Ser in crate` under `.strict_prefix_only().strict_external()`, and nothing under `.strict_prefix_only()` alone, `.strict_external()` alone, or neither
- **PINNED-BY** `a_derive_path_in_an_attribute_is_a_mention`

#### Scenario: A declared item and a binding of one name are both candidates
- **WHEN** `crate::m` writes `#[cfg(unix)] pub use crate::a::X;` beside `#[cfg(not(unix))] pub struct X;`, in either order, and the crate root calls `m::X::f()`
- **THEN** the system reports `crate::m::X::f in crate` under `crate::m::X` and `crate::a::X::f in crate` under `crate::a`, in either mode
- **PINNED-BY** `a_declared_item_and_a_binding_of_one_name_are_both_candidates`

#### Scenario: A glob brings each binding only where it is visible
- **WHEN** `crate::s` writes `#[cfg(unix)] pub use crate::a::X;` beside `#[cfg(not(unix))] use crate::b::X;`, and the crate root writes `use s::*;` and calls `X::f()`
- **THEN** the system reports nothing under `crate::b`, and `crate::a::X::f in crate` beside `glob crate::s in crate` under `crate::a`
- **PINNED-BY** `a_glob_brings_each_binding_only_where_it_is_visible`

#### Scenario: A local name shadows a crate of the same name
- **WHEN** the crate root declares `pub mod std { pub mod process { pub fn id() -> u32 { 0 } } }` and calls `std::process::id()` under a prefix `std::process`; or a package depending on `md5x` writes `use local::*;` bringing a local module `md5x` and calls `md5x::compute()` under a strict-external prefix `md5x`
- **THEN** the system reports nothing in either mode
- **PINNED-BY** `a_local_std_module_shadows_the_sysroot`
- **PINNED-BY** `a_glob_brought_name_shadows_a_same_named_dependency`

### Requirement: A glob that can bring a prefix-resolving name into scope reacts (fail-closed)

The rule SHALL be stated by the **hazard**, not a single glob shape (an enumerated shape list
would itself drift): the system SHALL react (fail-closed) on a glob import within the governed
subtree whenever the glob can bring into scope a name that resolves under the confined prefix but
that the scanner cannot enumerate, naming the glob import as the finding. After resolving the
glob's own path from its scope, through the same imports, `type` aliases and re-exports every path is resolved through, the system SHALL react
when the resolved glob path is: (a) the confined prefix or **beneath** it (`use std::time::*`,
`use std::time::ext::*`); (b) an **ancestor** of the prefix — the glob brings the prefix's next
segment below the ancestor into scope (`use std::*` brings module `time`, the segment below `std`,
into scope); or (c) a **local module whose own re-export closure reaches under the prefix** —
where "reaches" applies this same hazard test recursively (chased to a fixpoint / visited set,
cycle-safe), over that module's re-exports and `type` aliases, each resolved from its own scope: a concrete `pub use std::time::…` (or `pub use std::time;`, or a `type … = std::time::…;` alias of any visibility),
OR a glob/ancestor re-export in that module that itself reaches the prefix (`pub use std::time::*;`
/ `pub use std::*;` inside `crate::support`, then `use crate::support::*;` in the subtree), OR a private glob in the
module the glob names — the one module whose private globs it carries — that is visible from the module the
reacting glob is written in and itself reaches the prefix (a private `use std::process::*;` in the crate root, then
`use super::*;` in `crate::agent`). Grouped or mixed glob forms (`use std::time::{*}`, `use std::time::{self, *}`)
SHALL be treated as globs. A glob finding SHALL NOT be suppressed by `.ending_with` narrowing (a
glob has no call terminal segment; narrowing applies to calls only). The scanner cannot prove such
a glob introduces no forbidden read, so the glob itself is the violation — one finding, never an FP
flood, never a silent pass. The hazard test is wider than what a glob can bring into scope, and that width SHALL
be declared by the over-reaction scenario below rather than claimed as precision: it asks whether any alias or
re-export **beneath** the glob's resolved module resolves under the prefix, not whether the glob brings that name
into scope. Where the glob's module is local, a call through a name the glob brings SHALL also resolve through
it and report as a call, beside the glob's own finding. The hazard SHALL cover every name the resolver
cannot enumerate, which are those a glob of an external path brings, and is wider than that only as the bound below
declares: a concrete binding a glob carries — a
private `use` or `extern crate` of an ancestor reaching a descendant through `use super::*` included — is
enumerated, so a call through it SHALL resolve and report as a call, and the binding SHALL NOT make the glob react.

#### Scenario: A glob of the confined prefix reacts
- **WHEN** `crate::core` declares `use std::time::*;` under a boundary confining `std::time`
- **THEN** the system reacts, naming the glob import as the finding

#### Scenario: A glob above the prefix reacts
- **WHEN** `crate::core` declares `use std::*;` (bringing module `time` into scope) then `time::Instant::now()`, under a boundary confining `std::time`
- **THEN** the system reacts on the glob `use std::*;` (an ancestor glob that brings the prefix's next segment below the ancestor, `time`, into scope), rather than silently passing the unresolvable bare `time::…`

#### Scenario: A glob of a local re-exporting module reacts
- **WHEN** `crate::support` declares `pub use std::time::SystemTime;`, `crate::core` declares `use crate::support::*;` then `SystemTime::now()`, under a boundary confining `std::time`
- **THEN** the system reacts on the glob `use crate::support::*;` (a local module whose observable re-exports reach under the prefix)

#### Scenario: A glob of a local module that itself globs the prefix reacts (recursive hazard)
- **WHEN** `crate::support` declares `pub use std::time::*;`, `crate::core` declares `use crate::support::*;` then `Instant::now()`, under a boundary confining `std::time`
- **THEN** the system reacts on the glob `use crate::support::*;` — the hazard test applied recursively to `support`'s re-export closure finds a glob reaching under the prefix (the "family not shape" rule does not stop at one level)

#### Scenario: A glob carries an ancestor's private glob, and enumerates its private imports
- **WHEN** the crate root privately declares `use std::process::*;` and `crate::agent` declares `use super::*;` then calls `id()`; or `crate::agent` globs a sibling `crate::other` that privately declares `use std::process::*;`; or the crate root privately declares `use std::process::id;` or `extern crate std as sp;` and `crate::agent`'s `use super::*;` holds no call, an `id()` call, or an `sp::process::id()` call, under a boundary confining `std::process` over `crate::agent`
- **THEN** the system reports `glob crate in crate::agent` for the ancestor's private glob; nothing for the sibling's, which `crate::agent` cannot see; nothing for the private import with no call; and `std::process::id in crate::agent` for each call — with and without `.strict_external()`
- **PINNED-BY** `a_glob_carries_an_ancestors_private_glob_and_enumerates_its_private_imports`

#### Scenario: A glob carries no private glob of a module beneath its target
- **WHEN** `crate::a::b` privately declares `use std::process::*;` and `crate::a::b::c` declares `use crate::a::*;`, under a boundary confining `std::process` over `crate::a::b::c`
- **THEN** the system reports nothing, with and without `.strict_external()`: `crate::a::*` brings `crate::a`'s names and none of `crate::a::b`'s
- **PINNED-BY** `a_glob_carries_no_private_glob_of_a_module_beneath_its_target`

#### Scenario: An aliased-prefix glob reacts (glob path resolved first)
- **WHEN** `crate::core` declares `use std::time as t; use t::*;` under a boundary confining `std::time`
- **THEN** the system resolves the glob's own path `t → std::time` through `t`'s `use` binding, then reacts (case (a)) under the resolved `glob std::time in crate::core`, rather than missing it because the glob was written through an alias

#### Scenario: A glob finding is not suppressed by narrowing
- **WHEN** a boundary declares `.must_not_call_inline("std::time").ending_with(["now"])` and `crate::core` declares `use std::time::*;`
- **THEN** the system still reacts on the glob (narrowing filters call terminal segments, not globs)

#### Scenario: A call through a local glob reports beside the glob
- **WHEN** `crate::core` globs `crate::a` and `crate::b`, each holding a struct `X`, and calls `X::f()` — both globs written plainly, cfg-exclusive, or `crate::b` re-exporting `crate::a::X`
- **THEN** the system reports each glob that reaches the prefix and the call under each candidate it reaches (`crate::a::X::f in crate::core` under `crate::a`)
- **PINNED-BY** `a_glob_that_can_bring_the_prefix_reacts_however_the_name_is_used`
- **PINNED-BY** `an_explicit_import_beside_a_glob_reacts_through_each`

#### Scenario: A glob reacts to any alias or re-export beneath its resolved module — a stated bound
- **WHEN** `crate::agent` declares `mod hidden { pub type Spawner = std::process::Command; }` and holds only `mod tests { use super::*; }`, under a boundary permitting `std::process::Command` only within `crate::exec`
- **THEN** the system reacts on `glob crate::agent in crate::agent`: the glob only brings the `hidden` module into scope, not `Spawner`, but the alias beneath the glob's resolved module is still treated as a possible prefix-resolving name — an over-reaction declared, not a precision claim
- **PINNED-BY** `a_sibling_test_glob_reacts_to_an_alias_in_its_resolved_module`

### Requirement: A prefix may be permitted only within the governed subtree

`ModuleBoundary::in_crate(p).module(m).confine_inline_call(prefix)` SHALL permit inline calls resolving under
`prefix` **only** within `m`'s subtree: a call anywhere else in any compiled root of the package SHALL be a
violation, with the confined prefix as the target, the call's resolved path and module as the finding, and
`allowlist_gap` polarity. It is the permitting dual of `must_not_call_inline`, as `confine_external_crate` is for
imports, and it SHALL observe exactly what that rule observes: the same call-versus-mention default, the same
alias, type-alias, re-export and glob resolution, and the same `.ending_with`, `.strict_prefix_only` and
`.strict_external` modifiers. Its perimeter SHALL be the whole package: a compiled root whose graph has no `m`
SHALL be judged with an empty permitted region, and a package where no root declares `m` SHALL be a
constitution error. Permitting the prefix within `crate` SHALL be a constitution error, since the root's subtree
is the whole crate and the rule could never react. A declaration at `ScanDepth::Shallow` SHALL be a constitution
error: the permitted region is compared at the grain of a file's module, and a shallow region is the anchored module
alone, so a call in an inline child of the permitted file could not be told apart from a permitted one. The
misdeclarations `must_not_call_inline` refuses — an empty prefix, an empty verb set, narrowing combined with
strict — SHALL be refused here too. Its rule identity SHALL be its own
(`tianheng.rule/guibiao/confine-inline-call`), so declaring it leaves every `must_not_call_inline` finding
byte-identical.

#### Scenario: An inline call in a sibling of the permitted module reacts
- **WHEN** `crate::exec` is the permitted module for `std::process::Command` and a sibling `crate::other` calls `std::process::Command::new(…)`
- **THEN** the system exits 1 with `std::process::Command::new in crate::other`, `allowlist_gap`, under the library root's unit
- **PINNED-BY** `an_inline_call_in_a_sibling_of_the_permitted_module_reacts`

#### Scenario: An inline call in a root without the permitted module reacts
- **WHEN** the library root declares `crate::exec` and the binary root, which declares no `exec`, calls `std::process::Command::new(…)`
- **THEN** the system exits 1 with `std::process::Command::new in crate` under the binary root's unit
- **PINNED-BY** `an_inline_call_in_a_root_without_the_permitted_module_reacts`

#### Scenario: An aliased or glob-reached call outside the permitted module reacts
- **WHEN** a sibling writes `use std::process::Command as Spawn;` and calls `Spawn::new(…)`, or writes `use std::process::*;`
- **THEN** the system reacts on the resolved call, or on the glob, as `must_not_call_inline` does
- **PINNED-BY** `an_aliased_inline_call_outside_the_permitted_module_reacts`
- **PINNED-BY** `a_glob_bringing_the_confined_prefix_outside_the_permitted_module_reacts`

#### Scenario: The permitted module, its inline tests and a mention elsewhere are clean
- **WHEN** `crate::exec` and its inline `mod tests` call the prefix, a sibling only names the type, and siblings hold `mod tests { use super::*; }` with no alias of the prefix anywhere
- **THEN** the system exits 0
- **PINNED-BY** `inline_calls_within_the_permitted_module_and_its_inline_tests_are_clean`
- **PINNED-BY** `a_type_only_mention_outside_the_permitted_module_is_clean`
- **PINNED-BY** `a_private_use_in_the_permitted_module_and_sibling_test_globs_are_clean`

#### Scenario: Narrowing applies to the permitting form
- **WHEN** the boundary declares `.confine_inline_call("std::process::Command").ending_with(["new"])` and a sibling calls `std::process::Command::output(…)`
- **THEN** the system exits 0
- **PINNED-BY** `a_narrowed_inline_call_confinement_ignores_other_verbs`

#### Scenario: A permitting confinement it cannot judge is refused
- **WHEN** the permitted module is `crate`, no compiled root declares it, the confined prefix is empty, or the boundary is declared at `ScanDepth::Shallow`
- **THEN** the system exits 2
- **PINNED-BY** `an_inline_call_confinement_to_the_crate_root_is_refused`
- **PINNED-BY** `an_inline_call_confinement_to_a_module_no_root_declares_is_refused`
- **PINNED-BY** `an_inline_call_confinement_with_an_empty_prefix_is_refused`
- **PINNED-BY** `an_inline_call_confinement_at_shallow_depth_is_refused`

#### Scenario: Severity and baseline apply to the permitting form
- **WHEN** the boundary is declared `warn()`, or one finding is baselined and a second call is added in another module
- **THEN** the warn boundary exits 0 with the advisory reported, and only the new call reacts against the baseline
- **PINNED-BY** `a_warn_inline_call_confinement_reports_without_failing`
- **PINNED-BY** `a_baselined_inline_call_does_not_mask_a_new_one`

### Requirement: Explicit read-verb narrowing owns its false negative

A confinement MAY be narrowed with `.ending_with([…])`; when narrowed, the system SHALL react
only on calls whose terminal segment (leaf-exact) is one of the declared verbs (e.g. `["now"]`).
Narrowing is a deliberate, adopter-owned act: a read reachable only through a verb the adopter
did not declare (a future `::current()`) SHALL be a false negative the **adopter** accepts by
narrowing. The engine MUST NOT bake a default verb set of its own.

#### Scenario: Narrowing drops a benign constructor call
- **WHEN** a boundary declares `.must_not_call_inline("std::time").ending_with(["now"])` and `crate::core` calls both `std::time::Instant::now()` and `std::time::Duration::from_secs(5)`
- **THEN** the system reacts on `Instant::now()` and does NOT react on `Duration::from_secs(5)` (terminal `from_secs` is not a declared verb)

#### Scenario: A future read verb outside the declared set is a documented bound
- **WHEN** `crate::clock` defines `now` and `current`, a boundary on `crate::core` confining `crate::clock` is narrowed to `.ending_with(["now"])`, and `crate::core` calls `crate::clock::current()`
- **THEN** the system does NOT react (a false negative the adopter owns by narrowing), rather than the engine silently guessing which verbs are reads
- **PINNED-BY** `inline_a_verb_outside_the_declared_set_is_a_bound`

### Requirement: Strict escalation forbids non-call mentions

A confinement MAY be escalated with `.strict_prefix_only()`; when escalated, the system SHALL
react on **any** path resolving under the prefix, call or not — including type annotations,
constants, and value-position mentions. A `use` leaf SHALL be judged as the `use` path it is, read from the scope
the `use` stands in in the target's edition, and never as an expression path: a grouped leaf holds the whole path
its group spells, and an empty group, which imports nothing, mentions the path before it. A glob a macro's group holds
SHALL be judged as a glob, wherever in the group it stands. This is the whole-surface isolation posture for a subtree
that may not even name the module. Narrowing and escalation are mutually exclusive: combining
`.ending_with(…)` with `.strict_prefix_only()` on one boundary SHALL be a constitution error
(exit 2), never a silent precedence choice.

#### Scenario: A use in a macro's group is judged as a use path under strict-prefix-only
- **WHEN** `crate::core` writes `id! { use crate::{clock::now}; }` with a `macro_rules! id` passing its tokens through, or a `macro_rules!` body writes `use $crate::clock::now;` and `crate::core` invokes it, or `crate::core` defines and invokes a `macro_rules!` whose body writes `use $crate::{clock::now};`, under `.strict_prefix_only()` confining `crate::clock` over `crate`
- **THEN** the system reports `crate::clock::now in crate::core`, `crate::clock::now in crate` and `crate::clock::now in crate::core` respectively, refusing none: a `use` a macro's group holds is judged as the `use` path it is, a `$crate` head read as `crate`, and rustc 1.96.0, edition 2021, builds each
- **PINNED-BY** `a_use_in_a_macros_group_is_judged_as_a_use_path_under_strict_prefix_only`

#### Scenario: A use in a macro's block binds, and a glob in a macro's group is judged
- **WHEN** `crate::core` invokes `macro_rules! m { () => { use crate::clock::{self}; clock::now(); }; }`, or the same with `use crate::clock::*; now();`, under `must_not_call_inline("crate::clock")`; or writes `id! { #[allow(unused_imports)] use crate::clock::*; }` under `.strict_prefix_only()`
- **THEN** the system reports `crate::clock::now in crate::core`, `glob crate::clock in crate::core` and `glob crate::clock in crate::core` respectively: a `use` a block inside a macro's group holds binds that block's paths as the expansion does, and every glob a macro's group holds is judged under strict as a glob; rustc 1.96.0, edition 2021, builds each
- **PINNED-BY** `a_use_in_a_macros_group_binds_and_globs_as_written`

#### Scenario: A path in a discriminant's turbofish is read
- **WHEN** the crate root writes `pub const fn f<A, B>() -> isize { 0 }` and `pub enum E { A = f::<u8, std::process::Command>() }`, or the same turbofish in `pub const K: isize = …;`, or `A = 1 + <u8 as Tr<u8, std::process::Command>>::X` and `A = -<u8 as Tr<u8, std::process::Command>>::X` beside a trait `Tr` declaring `X`, under `.strict_prefix_only()` confining `std::process` over `crate`
- **THEN** the system reports `std::process::Command in crate` for each: a comma inside a discriminant's turbofish or qualified path separates no variants, a qualified path's `<` read by the one judgement every path reader asks, and rustc 1.96.0, edition 2021, compiles each
- **PINNED-BY** `a_path_in_a_discriminants_turbofish_is_read`

#### Scenario: A use leaf is judged as a use path under strict-prefix-only
- **WHEN** `crate::sub` writes `use crate::clock::now;`, `use crate::{clock::now};`, `use crate::{clock::{now}};`, `use {crate::clock::now};`, `use crate::clock::{self};`, `use crate::clock::{};` or `use crate::{clock::{}};`, or in an edition-2015 package `use clock::now;`, under `.strict_prefix_only()` confining `crate::clock` over `crate`; and, beside a `mod clock` of its own, `use crate::other::{clock::now};` under a prefix `crate::sub::clock`
- **THEN** the system reports `crate::clock::now in crate`, or `crate::clock in crate` for the `{self}` leaf and each empty group, for each of the first, and nothing for the last: rustc 1.96.0 compiles each, and a leaf names the path its tree spells from where the `use` stands
- **PINNED-BY** `a_use_leaf_is_judged_as_a_use_path_under_strict_prefix_only`

#### Scenario: Strict flags a type annotation
- **WHEN** a boundary declares `.must_not_call_inline("std::time").strict_prefix_only()` and `crate::core` contains `now: std::time::Instant` (a type annotation)
- **THEN** the system reacts (strict forbids mentions, not only calls)

#### Scenario: Combining narrowing and strict is a constitution error
- **WHEN** a boundary declares `.must_not_call_inline("std::time").ending_with(["now"]).strict_prefix_only()`
- **THEN** the system reacts with exit 2 (a contradictory declaration), not a silent resolution

### Requirement: Macro bodies are conservatively scanned, never silently skipped

Within a governed subtree, a macro-invocation body SHALL be token-scanned for paths resolving
under the prefix through the scope table's bindings, and SHALL react on a match. The system
MUST NOT silently skip macro-invocation bodies — that would be a false negative, the one
forbidden bug (real reads hide in `cfg_if!` / logging / async DSL bodies).

#### Scenario: A forbidden call inside a macro body reacts
- **WHEN** `crate::core` contains `cfg_if! { if #[cfg(feature="x")] { std::time::Instant::now() } }` under a boundary forbidding inline calls under `std::time`
- **THEN** the system reacts (the macro body is scanned, not skipped)

### Requirement: Observation bounds are stated, not silent

The following SHALL be OUT OF SCOPE as stated coverage bounds, never a claimed reaction and never
a silent pass beyond them: (1) a read whose type is not in a plain written path — a
receiver-method call (`instant.elapsed()`) or a call through a path beginning with `<` (`<Type>::now()`,
`<Type as Trait>::now()`, type inside `<…>`) — no type inference; (2) an alias introduced *within* an unexpanded
macro-invocation body; (3) a symbol name assembled by fragment/proc-macro construction (`paste!`,
`concat_idents!`) or generated by a proc-macro; (4) a path reached through an **external**-crate
re-export (foreign AST is not observed); (5) a **fully-qualified, un-`use`d external-crate call**
whose head is a declared dependency (`chrono::Utc::now()` with no `use chrono`) — a stated
non-observation **under the default**, resolved as external and observed **only** under
`.strict_external()` (see "Strict-external observation of fully-qualified external calls"); (6) a
forbidden path taken as a **value** (fn-item / closure) rather than called — covered only under
`.strict_prefix_only()`; and (7) the module scanner's **inherited file-scope bounds** —
`#[cfg]`-gated code (observed as written, cfg-blind) — **except**
macro-invocation bodies, which this rule overrides by scanning them (per "Macro bodies are
conservatively scanned"). Even under `.strict_external()`, the following SHALL remain a stated
bound, never a silent claim of coverage: a name
brought in by a **glob** import except via the glob-hazard reaction — which under `.strict_external()`
**extends to external-crate globs** (an external glob that can bring a prefix-resolving name into
scope reacts fail-closed, as under the sysroot case). A bare head shadowed by a local module /
definition / import (the local-precedence carve-out) likewise stays local — checked against the
call's TRUE inline module, so a file-top item no longer masks an external call inside an inline
`mod name { … }` submodule (that inline-submodule shadow is now CLOSED, at any nesting depth).
Finally, strict-external only: a `mod name {` token or unbalanced braces **inside a
macro-invocation body** opens a module scope for the calls inside that body, while no declaration
inside a macro's group is recorded, so a call's true module may be mis-attributed — a stated bound. Two more follow from a head no scope binds naming nothing: (8) an item a macro
generates is not in the scope table, so a bare call of it in its own module names nothing, while a
crate-rooted path naming it from elsewhere still reacts; and (9) a prelude name called bare — `drop(x)`,
`Some(..)`, `Box::new(..)` — is not read as its standard-library path, since the prelude's contents are
not read. Each bound is a declared non-observation, not a silent pass on a case within scope.
These over-reactions SHALL be stated beside them. The scope table does not read generic parameter
lists, so a head naming a generic parameter is read as whatever the module binds under that name, and records no
`fn` or closure parameter and no `let` binding, so a head naming one is read as whatever the enclosing scopes bind
under that name. A path's
role is read from the tokens beside it, so a tuple-struct or tuple-variant pattern — in a `let`, `if let`,
`while let` or let-else, a `for` loop, a match arm, a `fn` or closure parameter, a macro's arguments or a
destructuring assignment's left side — is read as a call. A `}` is read as an operand's end, so under
`.strict_external()` the tail of a qualified path opening a statement right after a `}` is read as a rooted path;
and a `<<` opening a generic list is read as a shift, so the tail of a qualified path it opens there —
`Vec<<u8 as Tr>::md5x>` — is read as a rooted path under `.strict_prefix_only()` and `.strict_external()`. A glob
reacts to any alias or re-export beneath the module it resolves to, whether or not the glob brings that alias into
scope.

#### Scenario: A `#[path]`-remapped file in the subtree is observed
- **WHEN** a `#[path = "…"]`-remapped module inside `crate::core` contains `std::time::Instant::now()`, whether the attribute is written directly or wrapped in `cfg_attr`
- **THEN** the system reacts, naming the remapped module — the scanner follows an unconditional remap to its target and union-scans a `cfg_attr`-wrapped one, so the confinement observes the call there exactly as it does one written in the module's own file

#### Scenario: A receiver-method read is a documented bound
- **WHEN** `crate::core` calls `some_instant.elapsed()` where `some_instant` is an `Instant` value received by injection, or calls `<std::time::SystemTime>::now()`, `<S as crate::clock::Clock>::now()` or `<SystemTime as Clone>::clone(t)`
- **THEN** the system does not claim to observe it (no type inference on the receiver or the qualified type) — a stated bound, not a silent assertion of cleanliness
- **PINNED-BY** `inline_receiver_method_read_is_a_bound`
- **PINNED-BY** `inline_qualified_path_is_the_type_directed_bound`

#### Scenario: A macro-generated item called bare in its own module is not observed — a stated bound
- **WHEN** a file module `m` expands `make!()` into `pub fn gen() -> u8`, calls `gen()` bare, and is governed from `crate` under a prefix `crate::m`; beside it, the crate root calls `crate::m::gen()`
- **THEN** the system does not claim to observe the bare call — the macro's item is not in the scope table, so the head names nothing — and reports the crate-rooted call as `crate::m::gen in crate`, in either mode
- **PINNED-BY** `a_macro_generated_item_called_bare_in_its_module_is_a_bound`
- **PINNED-BY** `a_crate_rooted_call_of_a_macro_generated_item_reports`

#### Scenario: A use written in a macro group outside any block binds nothing — a stated bound
- **WHEN** `crate::core` writes `id! { use crate::clock::{self}; pub fn f() { clock::now(); } }` with a `macro_rules! id` passing its tokens through, under `must_not_call_inline("crate::clock")` over `crate::core`
- **THEN** the system does not claim to observe the call — where the macro expands the `use` is not read, so it binds in no scope and `clock` names nothing
- **PINNED-BY** `a_use_written_in_a_macro_group_outside_any_block_binds_nothing`

#### Scenario: A prelude name called bare is not read as its std path — a stated bound
- **WHEN** a module calls `drop(x)` bare under a boundary forbidding inline calls under `std::mem`
- **THEN** the system does not claim to observe the call — the prelude's contents are not read, so `drop` names nothing rather than `std::mem::drop`
- **PINNED-BY** `a_prelude_name_called_bare_is_not_read_as_its_std_path`

#### Scenario: A path in a pattern position is read as a call — a stated bound
- **WHEN** a file module declaring `pub struct P(pub u8)`, `pub enum E { A(u8), B }` and `pub struct S { pub a: P }` writes `let P(x) = p;`, a let-else, `if let E::A(x) = e`, `while let Some(E::A(x)) = v.pop()`, `for P(x) in v`, a match arm `E::A(x) if x > 0 =>`, a struct pattern `S { a: P(x) }`, a parameter `P(x): P`, closure parameters `|P(x): P|` and `move |P(x): P|`, `matches!(e, E::A(_))` and `let x; P(x) = p;`, governed from `crate` under a prefix naming the type
- **THEN** each reports the pattern's path — `crate::m::P in crate::m` or `crate::m::E::A in crate::m` — in either mode: an over-reaction declared, not a precision claim
- **PINNED-BY** `a_path_in_a_pattern_position_is_read_as_a_call`

#### Scenario: A qualified path after a closing brace is read as a rooted path — a stated bound
- **WHEN** `<W>::md5x()` follows, as a statement, an `if … else`, a `loop`, a `match`, a bare block, a labelled loop, an `unsafe` block, a nested `fn` item or a `while`, in a package depending on `md5x`; and, as controls, follows a `;` or a function body's `{`
- **THEN** after a `}` the system reports `md5x in crate` under `md5x` with `.strict_external()` and nothing under the default, and after the controls nothing in either mode: an over-reaction declared, not a precision claim
- **PINNED-BY** `a_qualified_path_after_a_closing_brace_is_read_as_a_rooted_path`

#### Scenario: A qualified path a shift opens in a generic list is read as a rooted path — a stated bound
- **WHEN** a package depending on `md5x` writes `Vec<<u8 as Tr>::md5x>` in a `let` type, a parameter, a `type` alias or an `impl`'s self type, `Tr` declaring an associated type `md5x`
- **THEN** the system reports `md5x in crate` under `md5x` with `.strict_prefix_only()` and `.strict_external()`, and nothing with `.strict_external()` alone: an over-reaction declared, not a precision claim
- **PINNED-BY** `a_qualified_path_a_shift_opens_in_a_generic_list_is_read_as_a_rooted_path`

#### Scenario: A generic parameter named like an import is read as the import — a stated bound
- **WHEN** `crate::core` writes `use std::process::Command;` and `pub fn f<Command: Default>() -> Command { Command::default() }` under a boundary forbidding inline calls under `std::process`
- **THEN** the system reports `std::process::Command::default in crate::core`: Rust resolves `Command` to the generic parameter, and the scanner, which does not read generic parameter lists, reads the module's import — an over-reaction declared, not a precision claim
- **PINNED-BY** `inline_generic_parameter_named_like_an_import_is_read_as_the_import`

#### Scenario: An import in a block of what is not read is read with the scope around it — a stated bound
- **WHEN** the crate root writes `pub mod forbidden { pub fn id() -> u32 { 0 } }`, `use crate::forbidden::id;` and `pub fn g() -> u32 { use std::process::id; id() }` under a prefix `crate::forbidden`
- **THEN** the system reports `crate::forbidden::id in crate`, with and without `.strict_external()`: rustc calls `std::process::id`, and the scanner, which does not read `std` and so cannot tell whether the block's import holds a value, also reads `id` from the scope around the block — an over-reaction declared, not a precision claim
- **PINNED-BY** `a_block_import_of_what_is_not_read_is_read_with_the_scope_around_it`

#### Scenario: An import of what is not read beside a glob is read with the glob — a stated bound
- **WHEN** `crate::core` writes `use crate::forbidden::*;` beside `use std::fmt;` and calls `fmt()`, or beside `use std::mem::swap;` and calls `swap(&mut a, &mut b)`, where `crate::forbidden` defines `fmt` and `swap` as functions, under a prefix `crate::forbidden`
- **THEN** the system reports `crate::forbidden::fmt in crate::core` and `crate::forbidden::swap in crate::core` respectively, each beside `glob crate::forbidden in crate::core`, with and without `.strict_external()`: rustc 1.96.0, edition 2021, calls the glob's `fmt`, since `std::fmt` names a module and no value, and calls `std::mem::swap`, which the scanner, not reading `std`, cannot tell holds a value — the second an over-reaction declared, not a precision claim
- **PINNED-BY** `an_import_of_what_is_not_read_beside_a_glob_is_read_with_the_glob`

#### Scenario: A parenthesized fn bound is read as a call — a stated bound
- **WHEN** the crate root writes `pub fn f<F: std::ops::Fn(u8) -> u8>(g: F) -> u8 { g(1) }`, `pub fn h() -> impl std::ops::FnOnce() { || () }`, or `pub fn k(g: &dyn std::ops::FnMut(u8)) -> usize { … }`, under a prefix `std::ops`
- **THEN** the system reports `std::ops::Fn in crate`, `std::ops::FnOnce in crate` and `std::ops::FnMut in crate`, with and without `.strict_external()`: rustc 1.96.0, edition 2021, compiles each and calls nothing there, and the scanner reads a path followed by a parenthesized group as a call — an over-reaction declared, not a precision claim
- **PINNED-BY** `a_parenthesized_fn_bound_is_read_as_a_call`

#### Scenario: A local binding named like an import is read as the import — a stated bound
- **WHEN** the crate root writes `use crate::clock::now;` beside `pub mod clock { pub fn now() {} }`, and a function calls `now()` where `now` is its own `fn` parameter, a `let` binding of a closure, or a closure's parameter, under a prefix `crate::clock`
- **THEN** the system reports `crate::clock::now in crate` for each, with and without `.strict_external()`: Rust resolves `now` to the local binding, and the scanner, which records no parameter or `let` binding, reads the import — an over-reaction declared, not a precision claim
- **PINNED-BY** `a_local_binding_named_like_an_import_is_read_as_the_import`

#### Scenario: A path taken as a value is a documented bound under the default
- **WHEN** `crate::core` writes `let f = std::time::SystemTime::now; f();` under a default (non-strict) confinement
- **THEN** the system does not react (value-position mention is a stated bound under the default; `.strict_prefix_only()` catches it) — declared, not silent
- **PINNED-BY** `inline_value_capture_is_a_bound_under_the_default`

#### Scenario: An external-crate re-export is a documented bound
- **WHEN** a foreign crate re-exports `std::time::SystemTime` and `crate::core` reaches it through that foreign path
- **THEN** the system does not claim to observe it (foreign AST is not scanned) — a stated bound
- **PINNED-BY** `inline_foreign_reexport_of_the_confined_path_is_a_bound`

### Requirement: Constitution errors are loud, never silent

A misdeclared boundary SHALL react with exit 2 (constitution error), never a silent no-op: an
empty prefix; an empty verb set passed to `.ending_with([])`; the contradictory
`.ending_with(…).strict_prefix_only()` combination; a governed subtree anchor that resolves
to no reachable module; and a governed subtree anchor — the judged module of `must_not_call_inline`
or the permitted module of `confine_inline_call` — written in any spelling but the canonical module
path `module-boundary` states. Each misdeclaration the boundary alone decides — a blank or non-canonical prefix,
narrowing combined with strict, an empty verb set, and `confine_inline_call` over `crate` or at `ScanDepth::Shallow`
— SHALL be refused before any compilation unit is walked, so a scan refusal in some file never stands in front of
the declaration to repair. A governed source file that exists but cannot be read SHALL likewise be a
scan error (exit 2), never silently skipped. In contrast, a **valid** prefix that matches no
inline call in a resolvable subtree is **clean** (exit 0), not an error — a confinement with zero
findings is a passing reaction, exactly as a never-imported confined crate is clean under
`external-crate-confinement`.

The confined prefix of either builder SHALL be held to one spelling and to naming something, through one
implementation both share, because a prefix is compared with resolved call paths segment by segment and one
that matches no spelling a resolved path takes never reacts. Its spelling SHALL be `::`-separated identifiers,
read by the identifier test module paths are read by, optionally starting with a leading `::` for explicit
external crate disambiguation, and starting with a head that can name a crate or module; `r#x` and `x` SHALL be
one identifier, recorded without the raw prefix. Every identifier — of a prefix, a module path, and every token the
scanner reads — SHALL be compared and reported in Unicode Normalization Form C, as rustc compares one, so a name
written decomposed and the same name written precomposed are one name. An identifier SHALL be read as the Rust Reference reads one — `_`
or a Unicode `XID_Start` character, then `XID_Continue` characters — so a segment holding a soft
hyphen, a word joiner or an emoji is no identifier under any head, and a segment `_` alone, which the Reference reads
as no identifier, SHALL be refused past the head as well. Under a sysroot head — `std`,
`core`, `alloc`, `proc_macro` or `test`, the one list every reader of a sysroot head takes — every segment SHALL be
ASCII, since every path those crates publish is and nothing else holds a sysroot prefix to what it names. Any other
spelling — an empty segment, a trailing `::`, whitespace, or a character past ASCII under a sysroot head — SHALL be
exit 2, quoting the written prefix and suggesting its trimmed, non-empty segments
when those are a valid spelling. The heads that can name nothing are those the Rust Reference's identifier grammar
excludes: `_`, which is not an identifier, and `r#crate`, `r#self`, `r#super` and `r#Self`, which are not raw
identifiers; with bare `self`, `super` and `Self`, which are relative to a module or type a declaration does not
have, and `crate` after a leading `::`. Each SHALL be exit 2, suggesting the unraw spelling where that is a valid
prefix. Any other head, keyword or not and in any edition, SHALL be accepted written bare or raw. A blank prefix SHALL keep the empty-prefix refusal above. A first segment
that is not `crate` names a crate, whose contents the scanner does not read. A sysroot crate (`std`, `core`,
`alloc`, `proc_macro`, `test`), a dependency the package declares under the local name a rename gives it, and the
package's own library SHALL be accepted. A first segment none of those confirms SHALL be accepted too, since a
dependency's crate name can differ from what `--no-deps` metadata reports, unless the same path rooted at `crate`
names something the crate declares — that SHALL be exit 2 suggesting the rooted spelling. A
`crate`-rooted prefix SHALL name a module some compiled root of the package declares, or an item one defines at
its top level, or be exit 2; a module or item present in one compilation unit is present. What a prefix names
past its first segment when that is not `crate`, or past an item of the crate, is not read — the bounds below.

#### Scenario: A prefix naming nothing, or written without its root, is a constitution error
- **WHEN** a crate declares `crate::clock` with `fn now`, `crate::core` calls `crate::clock::now()`, and a boundary declares `.must_not_call_inline(p)` on `crate::core`, or `.module("crate::clock").confine_inline_call(p)`, for `p` of `crate::clcok` or `clock`
- **THEN** the system exits 2 — naming the written prefix, and for `clock` suggesting `crate::clock`, which names a module the crate declares
- **PINNED-BY** `a_misspelled_crate_prefix_is_refused_not_judged_clean`
- **PINNED-BY** `a_prefix_written_without_its_crate_root_is_refused_with_the_rooted_spelling`

#### Scenario: An identifier is one name in either composition
- **WHEN** `crate` declares `pub mod sécret { pub fn go() {} }` and `crate::core` calls `crate::sécret::go()`, with the module, the call and a `must_not_call_inline` prefix each written either precomposed (U+00E9) or decomposed (`e` + U+0301), one of them in the other composition; or `crate::core` writes `use crate::sécret::go;` decomposed under `must_not_import` of the module precomposed
- **THEN** the system reports `crate::sécret::go in crate::core`, precomposed, for each call, and the import as a violation: rustc 1.96.0, edition 2021, builds each row, and refuses a file-form `mod` with a name past ASCII (E0754)
- **PINNED-BY** `an_identifier_is_one_name_in_either_composition`
- **PINNED-BY** `an_identifier_is_read_in_nfc_and_a_literal_as_written`

#### Scenario: A non-canonical prefix is a constitution error
- **WHEN** either builder is given `crate::clock::`, `std::time::`, `::std::time::`, `self::clock`, or `Self::clock`; or `std::pro` + U+00AD + `cess`, `std::` + U+2060 + `process`, `std::process` + U+2014, `core::pro` + U+00AD + `cess`, `proc_macro::Token` + U+0405 + `tream` or `test::bl` + U+043E + `ck_box`; or `std::_` or `md5x::_`; or `md5x::pro` + U+00AD + `cess`, `md5x::` + U+2060 + `hash` or `md5x::` + U+1F980; and, as controls, `md5x::été`, `md5x::模組` and `md5x::_é`
- **THEN** the system exits 2, quoting the written prefix and suggesting `crate::clock`, `std::time`, or `::std::time` where the trimmed segments are a valid spelling, and nothing for the sysroot spellings past ASCII or the five segments no identifier is; and accepts each control, whose characters are `XID_Start` and `XID_Continue`
- **PINNED-BY** `a_prefix_with_a_trailing_separator_is_refused`
- **PINNED-BY** `an_inline_prefix_is_accepted_only_in_its_canonical_spelling`

#### Scenario: A prefix head that names no crate or module is refused
- **WHEN** either builder is given `_`, `_::clock`, `r#_::clock`, `Self::now`, `Self::clock`, `r#Self::clock`, `self::clock`, `r#self::clock`, `super::clock`, `r#super::clock`, `::crate::clock`, `r#crate::clock` or `r#crate`
- **THEN** the system exits 2 quoting the written prefix, suggesting `crate::clock` for `r#crate::clock`, `crate` for `r#crate`, and nothing for the others
- **PINNED-BY** `a_prefix_head_naming_no_crate_or_module_is_refused`

#### Scenario: A keyword head is accepted bare or raw
- **WHEN** a package depends on a crate renamed `async`, `crate::core` writes `use r#async::f;` and calls `f()`, and a boundary's prefix is `async` or `r#async` — and likewise for `dyn`, `try`, `gen` and `union` in editions 2018, 2021 and 2024
- **THEN** the system accepts the prefix, reports the call under it, and records one identity for both spellings
- **PINNED-BY** `a_keyword_prefix_head_is_accepted_bare_or_raw`

#### Scenario: A prefix naming something that exists is accepted
- **WHEN** either builder is given `crate::clock`, `crate::clock::now`, `crate::r#clock`, `std::time`, `::std::time`, a dependency's local name, a renamed dependency's local name, or a module only the binary root declares
- **THEN** the system accepts it, and `crate::clock` and `crate::clock::now` react on the call exactly as before
- **PINNED-BY** `an_inline_prefix_naming_a_module_reacts_on_its_call`
- **PINNED-BY** `an_inline_prefix_naming_something_that_exists_is_accepted`

#### Scenario: A first segment naming another crate is accepted
- **WHEN** either builder is given `proc_macro`, `test`, the package's own library name followed by a module, a dependency's underscore-folded name, or `inilike::load` where the dependency declaring `[lib] name = "inilike"` is reported under its package name
- **THEN** the system accepts the prefix
- **PINNED-BY** `a_first_segment_naming_another_crate_is_accepted`

#### Scenario: A prefix segment past what guibiao reads is not verified — a stated bound
- **WHEN** either builder is given `std::tiem`, `extdep::nosuch` under a declared dependency `extdep`, `crate::clock::Clock::nwo` where `Clock` is a type `crate::clock` defines, or `clcok` where no `crate::clcok` exists and no dependency is named `clcok`
- **THEN** the system accepts the prefix and reports no violation: another crate's contents are its own source, associated items are not collected, and a first segment nothing confirms may be a dependency's crate name, so a misspelling there matches nothing and is not refused
- **PINNED-BY** `a_prefix_past_what_guibiao_reads_is_not_verified`

#### Scenario: A prefix naming a macro-generated item is refused — a stated bound
- **WHEN** `crate::clock` defines `stamp` through `make!(pub fn stamp() -> u64 { 0 });` or `make! { pub fn stamp() -> u64 { 0 } }`, `crate::core` calls `crate::clock::stamp()`, and either builder is given `crate::clock::stamp`
- **THEN** the system exits 2 as for a prefix naming nothing: no declaration inside a macro's group is recorded, so the item is not in the set the prefix is held to
- **PINNED-BY** `a_prefix_naming_a_macro_generated_item_is_refused`

#### Scenario: An empty prefix is a constitution error
- **WHEN** a boundary declares `.must_not_call_inline("")`
- **THEN** the system reacts with exit 2 (a misdeclaration), never a silent match-everything or match-nothing

#### Scenario: A misdeclaration is refused before the walk
- **WHEN** a crate root declares `mod ghost;` with no file, and a boundary declares an empty prefix; `confine_inline_call` over `crate`; `confine_inline_call` at `ScanDepth::Shallow`; `.ending_with(["now"]).strict_prefix_only()`; or `.ending_with([])`
- **THEN** the system reacts with exit 2 naming that misdeclaration, not the missing file
- **PINNED-BY** `a_misdeclared_inline_confinement_is_refused_before_the_walk`

#### Scenario: A non-canonical governed subtree anchor is a constitution error
- **WHEN** a boundary declares `.module("core").must_not_call_inline("std::time")`, or `.module("r#crate::exec").confine_inline_call("std::process::Command")`, over a crate declaring `crate::core` and `crate::exec`
- **THEN** the system reacts with exit 2, quoting the written anchor and suggesting `crate::core` or `crate::exec`
- **PINNED-BY** `every_module_path_role_refuses_a_non_canonical_spelling`

#### Scenario: A valid confinement with no matching call is clean
- **WHEN** a boundary confines `std::time` on `crate::core` and `crate::core` makes no inline call resolving under `std::time`
- **THEN** the system reports no violation and the reaction passes (exit 0)

### Requirement: Identity distinguishes the confined prefix and the call

A violation's baseline identity SHALL distinguish both the **confined prefix** and the **specific
offending element**, so that no two distinct confinements or offending elements collapse into one
baseline entry (which would let a baseline mask a new violation — the one forbidden bug). The
`finding` SHALL identify the offending element and its module: for a **call**, the resolved
canonical call path plus the call-site module; for a **glob** import (fail-closed), the glob's
import path plus its module. Distinct canonical call paths, or distinct glob imports, therefore
stay distinct findings; two textually-different calls resolving to the *same* canonical path in
the *same* module are the *same* violation (finding-level dedup, as the other module rules do —
not per-source-occurrence). The confined prefix SHALL be carried in the identity (in `target` or
in the `finding`), so two confinements with nested prefixes on the same subtree (e.g. `std` and
`std::time`) breached by the same call do not share an identity. The rule string alone SHALL NOT
be relied on to distinguish prefixes.

#### Scenario: Nested-prefix confinements do not mask each other
- **WHEN** `crate::core` is confined against both `std` and `std::time`, both breached by `std::time::Instant::now()`, and the `std` violation is in the baseline
- **THEN** the `std::time` violation still fails the reaction (exit 1) — the confined prefix is part of the identity, so baselining one prefix does not mask the other

#### Scenario: Two distinct calls in one module stay distinct
- **WHEN** `crate::core` calls both `std::time::Instant::now()` and `std::time::SystemTime::now()`, and only the first is in the baseline
- **THEN** the second still fails the reaction (finding is per-call: the resolved call path plus module, so one baselined call does not mask another)

### Requirement: CI reaction, severity, and baseline parity

The system SHALL fold inline-symbol-path findings into the same exit-code contract as the other
dimensions (`0` clean / `1` enforce violation / `2` constitution or scan error) and aggregate
them with the other boundaries. A boundary SHALL carry a severity (`enforce` default, `warn`
reports without failing), and its violations SHALL be gated against the same `Baseline` (identity
per "Identity distinguishes the confined prefix and the call"), so a project may adopt on a dirty
subtree and gate only on new calls.

#### Scenario: A warn boundary reports without failing
- **WHEN** a `warn`-severity inline-symbol-path boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A new call beyond the baseline fails
- **WHEN** an enforce-severity boundary has an inline call not present in the baseline
- **THEN** the system fails the reaction (exit 1) for that new call

### Requirement: Strict-external observation of fully-qualified external calls (opt-in)

A confinement MAY be extended with `.strict_external()`. When set, the system SHALL resolve a
written path's bare head that matches a **declared dependency name** (rename-aware, `-`→`_`
normalized to its import identifier), and any `::`-rooted head other than a sysroot crate, as that
external crate, so a **fully-qualified, un-`use`d external call** — e.g. `chrono::Utc::now()` or
`::chrono::Utc::now()` with no `use chrono` in scope — resolving under the confined prefix SHALL
react. Without the flag a sysroot head (`std`/`core`/`alloc`) is caught while a fully-qualified
external head, which no scope binds, names nothing — the stated non-observation under the default.

The flag closes **only** the fully-qualified, un-`use`d external call. Paths that already resolve
under the default SHALL keep reacting **without** the flag and are not its concern: a `use`d import
(`use chrono::Utc; Utc::now()`), a `use` rename (`use chrono::Utc as U; U::now()`), a bare crate
import (`use chrono; chrono::Utc::now()`), an `extern crate` under its name or its `as` alias
(`extern crate chrono as chr; chr::Utc::now()`), and a local `pub use` re-export of the external item
chased cross-module (`pub use chrono::Utc;` elsewhere, then `Utc::now()`) all react under the
default through their `use` bindings and the re-export closure. `.strict_external()` adds nothing to those; it only
reclassifies the fully-qualified, un-`use`d head.

The reclassification SHALL apply **only after** local precedence is honored, and local precedence
SHALL be the scope lookup itself: a head that any scope between the path and its module binds — an
import, a `type` alias, a name a glob brings, a child module or any local item definition
(mod/struct/enum/union/trait/type/fn/const/static), at the crate root as at **any module depth** —
stays local and does NOT react. Only a head no scope binds is matched against the dependency names.
Local precedence SHALL be **module-scoped**: only the *current* module's scope (its top-level
definitions, child modules and imports) shadows a bare head — a same-named item of a *different*
module SHALL NOT suppress the reclassification (that would be a false negative). An un-`use`d,
non-dependency bare head no scope binds SHALL name nothing a prefix can reach.

Where no scope binds the head, one **over-reaction** bound SHALL be stated, not silent, and only under a
**single-segment** bare crate prefix (`"rand"`) — a multi-segment prefix (`"chrono::Utc"`) is immune, as is every
prefix under the default, where a head no scope binds names nothing: a local `let` / parameter / closure binding
named like the crate may react under strict-external (a declared false positive). Where a scope does bind the
head — an import named like the local binding — the binding is read as what that scope binds, in either mode and
under any prefix reaching it, the local-binding bound stated with the over-reactions above. A `fn`
item's own name is its definition and never reacts, and module-top-level definitions are exempt.

`.strict_external()` is **orthogonal** to `.ending_with(…)` and `.strict_prefix_only()`: it changes
head *resolution*, not call-vs-mention breadth, and SHALL compose with either — unlike the
mutually-exclusive narrowing/escalation pair. When `.strict_external()` is **not** set, the
fully-qualified external call remains a stated non-observation and behavior is byte-identical to a
confinement without the flag, so no existing constitution's reaction changes.

#### Scenario: A fully-qualified external call reacts under strict-external
- **WHEN** a boundary declares `.must_not_call_inline("chrono::Utc").strict_external()`, crate `app` depends on `chrono`, and `crate::core` calls `chrono::Utc::now()` with no `use chrono` in scope
- **THEN** the system resolves the head `chrono` (a declared dependency) as external, matches the prefix `chrono::Utc`, and reacts
- **PINNED-BY** `inline_strict_external_reacts_on_a_fully_qualified_external_call`

#### Scenario: The fully-qualified external call is a stated bound under the default
- **WHEN** the same `chrono::Utc::now()` call is governed by `.must_not_call_inline("chrono::Utc")` **without** `.strict_external()`
- **THEN** the system does NOT react (the fully-qualified un-`use`d external call is a stated non-observation under the default; behavior is unchanged from before this capability)
- **PINNED-BY** `inline_strict_external_absent_fully_qualified_call_is_a_bound`

#### Scenario: A deep local module named like a dependency stays local under strict-external
- **WHEN** a boundary declares `.must_not_call_inline("time").strict_external()`, crate `app` depends on `time`, the governed subtree `crate::core` is **not** the crate root, and `crate::core` declares `mod time;` (a local child module) then calls `time::format()`
- **THEN** the system does NOT react — the local child module `crate::core::time` wins over the dependency-name match by local precedence, at a non-crate-root depth (no false positive on a deep local module)

#### Scenario: A local item definition named like a dependency stays local under strict-external
- **WHEN** a boundary declares `.must_not_call_inline("rand").strict_external()`, crate `app` depends on `rand`, and `crate::core` defines `fn rand() -> u32 { … }` then calls `rand()`
- **THEN** the system does NOT react — the local definition wins over the dependency-name match by local precedence (no false positive on a local item shadowing a dependency name)

#### Scenario: A file-top item does not mask an external call in an inline submodule under strict-external
- **WHEN** a boundary declares `.must_not_call_inline("rand").strict_external()`, crate `app` depends on `rand`, and `crate::core` defines a file-top `fn rand() -> u32 { … }` and an inline `mod tests { fn t() { rand::random(); } }`
- **THEN** the system reacts on the `rand::random()` call — the call's TRUE module is `crate::core::tests`, so the file-top `crate::core::rand` does NOT claim its head (the inline-submodule shadow false negative is closed); local precedence still exempts a `rand` item defined **within** `mod tests` itself, at any nesting depth

#### Scenario: A local alias shadowing a dependency name stays local under strict-external
- **WHEN** a boundary declares `.must_not_call_inline("time").strict_external()`, crate `app` depends on `time`, and `crate::core` declares `use crate::clock as time;` (a local alias) then calls `time::read()`
- **THEN** the system resolves `time` through the local `use`-map (which precedes the dependency-name match) and does NOT react

#### Scenario: An external-crate glob reacts under strict-external
- **WHEN** a boundary declares `.must_not_call_inline("chrono::Utc").strict_external()`, crate `app` depends on `chrono`, and `crate::core` declares `use chrono::*;`
- **THEN** the system resolves the glob head `chrono` as external (an ancestor of the confined `chrono::Utc`) and reacts fail-closed on the glob import (an external glob can bring a prefix-resolving name into scope) — whereas under the default the same glob head, which no scope binds, names nothing and does not react

#### Scenario: Strict-external composes with narrowing
- **WHEN** a boundary declares `.must_not_call_inline("chrono::Utc").strict_external().ending_with(["now"])` and `crate::core` calls both `chrono::Utc::now()` and `chrono::Utc::today()` (both fully-qualified, no `use`)
- **THEN** the system reacts on `now()` and does NOT react on `today()` (the external head is resolved, then the leaf-exact narrowing applies) — the two modifiers compose

### Requirement: A pathologically nested glob or alias chain is a scan error, never a silent drop

The system SHALL react 0/1/2 on a grouped `use` whose brace-group nesting depth is bounded by a
measured stack-safety cap, for both the glob-hazard scan and the alias-carrying `use` bindings
this capability's confinement check depends on, and past that cap SHALL fail loud (a constitution
error, exit 2) rather than silently dropping the nested glob base or alias from observation. A
real, compilable glob-hazard or alias chain nested past the cap would otherwise vanish entirely
from observation with no report — the false negative the core contract forbids. Nesting
comfortably under the cap SHALL be observed exactly as a shallower tree would be. Every reader of
imports — import reports, the scope table, the glob-hazard walk and the re-export closure — reads
`use` statements through one enumeration and use trees through one parser, with one cap of 128 brace
levels, so the readers cannot disagree about what a tree holds or how deep one is read. A cap of its own SHALL bound the chain of
imports, globs, re-exports and `type` aliases one resolution walks, counted in two measures, each capped at 64 on its
own: the bindings and glob paths one lookup is nested inside, and the segments one branch replaces as a crate-rooted path
is carried through what it names, each branch counted from where it began. Resolution SHALL answer a walk past the cap with a typed result quoting the binding
the chain was measured from and the module it stands in — a block in that module's path written `{block}`, since the
scanner's own numbering of blocks names nothing in the source — never with a truncated answer, and the scan SHALL refuse it as a scan
error (exit 2) only where it resolves a judged occurrence — a call, or any mention under
`.strict_prefix_only()` — or a glob. A chain no judged occurrence or glob walks through SHALL refuse
nothing. A cycle of aliases, imports or re-exports, which rustc refuses, SHALL end resolution with a
verdict and never a panic. Whether a resolution refuses SHALL NOT depend on the order a unit's modules or a
module's names are met in: they are read in one order, and a fold across several answers — the candidates of a
lookup, the paths of a crate-rooted path, the modules a glob hazard reads — reads every answer and takes the least
refusal, so a glob hazard that one module reaches and another cannot judge is refused. No
expression is read for where it ends, so no nesting of expressions is refused. Every refusal the scan meets in a
governed source file — a chain past its cap, a `use` tree past its cap — SHALL name that file.

A scope's answer for a name SHALL be read once per depth of the walk that asks it, rather than once per path
through the scope's globs, so a lattice of glob diamonds whose path count doubles per layer resolves in time its
size bounds. What every glob of a compilation unit names SHALL be read together, to a fixed point, in passes
starting from none, and a glob's own path SHALL never be read through the glob itself, as rustc never resolves a glob
through what it imports: a pass reads each glob from what the globs before it in the same pass have just been read
as, the passes read the unit's globs forwards and backwards in turn, and a pass that changes nothing ends the
reading, so a glob's path is read once per pass rather than once for each set of globs a walk could pass it through,
and a chain of globs each read through another settles past the chain cap in either order. The passes are bounded by
construction, by 64 or twice the unit's glob count plus two, whichever is more; a unit whose passes have not settled
by then refuses every glob as a scan error (exit 2), a refusal of what the scanner cannot judge. A lookup through globs SHALL read every scope
its chain of globs reaches once, and settle what each answers to a fixed point, so a lookup that comes back to a
scope ends there and a chain of globs meets neither the chain cap nor a depth of recursion. A large source SHALL be
read in time its size bounds, which the scenarios below measure for the shapes found to break it: a crate root's
many globs, modules globbing one another, a long chain of globs, a long list of brace groups, many inline modules
nested or side by side, many modules each globbed, and a chain through a re-export named as its own path's head. An
import SHALL be resolved without itself, as rustc resolves it, and that reading SHALL be kept as the scope's answer
for the lookup it answers rather than as a cycle the walk cut.

#### Scenario: A lattice of globs resolves once per scope
- **WHEN** the crate root globs the first layer of thirty layers of modules, each layer's two modules globbing both of the next layer's, whose last layer defines `leaf`, and calls `leaf()` under a prefix naming the last layer's first module
- **THEN** the system reports `crate::l29x::leaf in crate` beside the glob findings, within ten seconds
- **PINNED-BY** `a_lattice_of_globs_is_read_once_per_scope`

#### Scenario: A chain through a self-named re-export is read once per link
- **WHEN** `pub mod m0 { pub use md5x::md5x; }` stands beside forty modules each writing `pub use crate::m{i-1}::md5x;`, the crate depends on `md5x`, and `g` calls `crate::m40::md5x::f()` under a prefix `md5x` with `.strict_external()`
- **THEN** the system reports `md5x::md5x::f in crate` within ten seconds: rustc 1.96.0, edition 2021, builds the crate, and the lookup of `md5x` inside its own import reads the scope without it
- **PINNED-BY** `a_chain_through_a_self_named_re_export_is_read_once_per_link`

#### Scenario: A lookup through two modules globbing each other ends
- **WHEN** `mod a { pub use crate::b::*; pub fn f() { drop(1u8); } }` and `mod b { pub use crate::a::*; pub fn g() -> u32 { std::process::id() } }` stand in the crate root, under a prefix `std::process` with `.strict_external()`
- **THEN** the system reports `std::process::id in crate` within ten seconds, the lookup of `drop` ending where it comes back to the scope it went out from
- **PINNED-BY** `a_lookup_through_two_modules_globbing_each_other_ends`

#### Scenario: A crate root's globs read through each other are read once each
- **WHEN** a crate root writes sixteen globs `use std::<module>::*;` and calls `drop(1u8)` and `std::process::id()`; or writes `pub mod m { pub mod x { pub fn f() {} } }`, `use m::*;`, `use x::*;` and calls `f()`
- **THEN** the system reports `std::process::id in crate` under `std::process` within ten seconds for the first, and `crate::m::x::f in crate` beside both globs' findings under `crate::m::x` for the second
- **PINNED-BY** `a_crate_roots_globs_read_through_each_other_are_read_once_each`

#### Scenario: A long chain of globs resolves
- **WHEN** the crate root globs `m0`, each of three hundred modules `mN` globs `m(N+1)`, the last defines `leaf`, and the root calls `leaf()`
- **THEN** the system reports `crate::m300::leaf in crate` within ten seconds
- **PINNED-BY** `a_long_chain_of_globs_resolves`

#### Scenario: Modules globbing one another are read once each
- **WHEN** twelve child modules each `pub(crate) use super::*;` beneath a crate root that `pub use`s every child's glob, or eight modules each glob all seven others, each calling `drop(1u8)` beside a call to `std::process::id()`
- **THEN** the system reports `std::process::id in crate` for each, under `.strict_external()`, within ten seconds
- **PINNED-BY** `modules_globbing_one_another_are_read_once_each`

#### Scenario: Globs read through one another settle past the chain cap
- **WHEN** a crate root globs each of a hundred and fifty nested modules `a0::a1::…` by its bare name, in order or in reverse order, and calls `leaf()`, which the innermost defines
- **THEN** the system reports the innermost `leaf` in `crate` for each, within ten seconds
- **PINNED-BY** `globs_read_through_one_another_settle_however_long_their_chain`

#### Scenario: A glob is never read through itself
- **WHEN** a module writes `use std::collections::*;` and `use self::hash_map::*;` and calls `std::process::id()`, under a prefix `std::process` with `.strict_external()`
- **THEN** the system reports `std::process::id in crate`: the second glob's path names the `hash_map` the first brings, and is not read through what the second glob brings in turn
- **PINNED-BY** `a_glob_is_never_read_through_itself`

#### Scenario: Globs whose readings grow more than once settle
- **WHEN** a crate root writes `use c3::*;`, `use c2::*;`, `use c1::*;` and `use x::*;`, with `x` imported as `crate::p` under `cfg(unix)` and as `late::q` under `cfg(not(unix))`, `crate::p::c1::c2::c3` defining `leaf`, and calls `leaf()`, under a prefix `crate::p`
- **THEN** the system reports `crate::p::c1::c2::c3::leaf in crate`, the cfg-blind reading of `x` growing the root's globs more than once before they settle
- **PINNED-BY** `globs_whose_readings_grow_more_than_once_settle`

#### Scenario: A long list of brace groups is read once
- **WHEN** a function writes `let _x = [a < b, {0} > 0, {1} > 0, …]` with forty thousand brace groups, then calls `std::process::id()`
- **THEN** the system reports `std::process::id in crate` within ten seconds
- **PINNED-BY** `a_long_list_of_brace_groups_is_read_once`

#### Scenario: A glob hazard reads only the modules beneath its glob
- **WHEN** a crate root declares twelve thousand modules `pub mod mN { pub fn f() {} }`, globs each with `use mN::*;`, and calls `std::process::id()`
- **THEN** the system reports `std::process::id in crate` within ten seconds
- **PINNED-BY** `a_glob_hazard_reads_only_the_modules_beneath_its_glob`

#### Scenario: Many inline modules are read once each
- **WHEN** a crate root nests six hundred inline modules `pub mod m { … }`, the innermost calling `std::process::id()`; or declares two thousand inline modules side by side and one more calling it
- **THEN** the system reports `std::process::id in crate` for each, within ten seconds
- **PINNED-BY** `many_inline_modules_are_read_once_each`

#### Scenario: A grouped glob nested past the depth cap is a scan error

- **WHEN** a governed file declares a grouped glob import whose brace-group nesting exceeds the
  depth cap this scanner supports
- **THEN** the system reports a constitution error (exit 2) naming the depth bound it could not
  judge past, rather than silently treating the glob as absent

#### Scenario: An alias chain nested past the depth cap is a scan error

- **WHEN** a governed file declares a grouped `use` introducing an alias whose brace-group nesting
  exceeds the depth cap this scanner supports
- **THEN** the system reports a constitution error (exit 2), rather than silently dropping the
  alias's binding (which would let an inline call through it pass unresolved)

#### Scenario: An alias or re-export chain past the depth cap is a scan error
- **WHEN** a function body or a module declares `type A0 = std::time::SystemTime;` followed by a hundred aliases each naming the one before and calls through the last, or a hundred modules each re-export the previous one's `X` and the crate root calls through the last
- **THEN** the system reports a constitution error (exit 2) naming the cap and quoting the `type` alias or the binding the chain was measured from, in either mode
- **PINNED-BY** `an_alias_or_reexport_chain_past_the_depth_cap_is_a_scan_error`

#### Scenario: A chain past the cap that no judged occurrence walks refuses nothing
- **WHEN** a module declares `pub type A0 = u8;` followed by seventy aliases each naming the one before, with no call through them, beside a call `std::time::SystemTime::now()` or none; or a hundred modules each re-export the previous one's `X` and nothing calls through the chain
- **THEN** the system reports the unrelated call, or nothing, in either mode; and the re-export chain, whose `pub use` paths are mentions, is refused (exit 2) only under `.strict_prefix_only()`, which judges a mention
- **PINNED-BY** `an_unused_alias_chain_past_the_cap_is_not_judged`
- **PINNED-BY** `a_call_beside_an_unused_alias_chain_past_the_cap_reports`
- **PINNED-BY** `an_unused_reexport_chain_past_the_cap_is_judged_only_where_a_mention_is`

#### Scenario: An alias or re-export chain under the depth cap resolves
- **WHEN** the same chains have thirty-two links
- **THEN** the system reports the call under the item the chain names, in either mode
- **PINNED-BY** `an_alias_or_reexport_chain_under_the_depth_cap_resolves`

#### Scenario: A cyclic alias or re-export ends resolution
- **WHEN** a function body or a module writes `type A = B; type B = A;` and calls `A::now()`, or two modules re-export each other's `X` and code calls through one
- **THEN** the system returns a verdict — clean or violations — and does not panic
- **PINNED-BY** `a_cyclic_alias_or_reexport_ends_resolution`

#### Scenario: The verdict does not depend on the order modules are read
- **WHEN** a glob of `crate::p` reaches `std::process` through `crate::p::a`, and a re-export chain past the cap stands in `crate::p::b`, which sorts after it
- **THEN** the system reports a constitution error (exit 2) quoting the chain, in either mode
- **PINNED-BY** `the_verdict_does_not_depend_on_the_order_modules_are_read`

#### Scenario: A refusal names the file it was met in
- **WHEN** a crate of three files nests a `use` tree past the use-tree parser's cap in `src/deep.rs` alone
- **THEN** the constitution error names `src/deep.rs` and neither `src/lib.rs` nor `src/other.rs`, in either mode
- **PINNED-BY** `a_refusal_names_the_file_it_was_met_in`

#### Scenario: A refusal names a block-declared module as a reader can find it
- **WHEN** a module `inner` declared in a function body holds a chain of more than 64 `type` aliases, and a call in that body resolves through it
- **THEN** the constitution error quotes the chain in `crate::{block}::inner`
- **PINNED-BY** `a_chain_refused_in_a_block_declared_module_names_it_readably`

#### Scenario: A grouped glob nested just under the depth cap is still observed

- **WHEN** a governed file declares a grouped glob nested well under the depth cap
- **THEN** the system observes and reacts to it exactly as a shallower grouped glob would
