## MODIFIED Requirements

### Requirement: Unsafe-site observation

The system SHALL walk the whole target crate (descending file-based `mod x;` and inline `mod x { … }` alike) and observe every `unsafe` **site**, attributing each to its enclosing module. The observed sites SHALL be: an `unsafe fn` (free function, inherent method, trait method declaration, or trait-impl method), an `unsafe impl`, an `unsafe trait`, an `unsafe extern` block (the `unsafe` keyword form), and an `unsafe {}` expression block (observed within item bodies, including bodies of `const`/`static` initializers, closures, and nested functions). A `mod` declared **inside a function or block body** (which the top-level module walk does not descend) SHALL still be observed — its `unsafe` attributed to the enclosing file module — so no body-nested `unsafe` is silently dropped. A site SHALL react iff its enclosing module is **not under** any allowed subtree (a module equal to or beneath an allowed subtree passes). Within the observed source there SHALL be no false negative: an observed `unsafe` site outside every allowed subtree MUST react.

#### Scenario: An unsafe block outside the subtree is a violation

- **WHEN** the crate has `only_under(["crate::ffi"])` and a function in `crate::net` contains an `unsafe { … }` block
- **THEN** the system emits a violation naming the module `crate::net` and the `unsafe block`

#### Scenario: An unsafe fn / impl / trait outside the subtree is a violation

- **WHEN** the crate has `only_under(["crate::ffi"])` and `crate::net` declares `unsafe fn decode()`, an `unsafe impl` block, or an `unsafe trait`
- **THEN** the system emits a violation for each, named by kind and (where present) name, qualified by `crate::net`

#### Scenario: Two unsafe impls that differ in trait or self type stay distinct

- **WHEN** `crate::net` (outside the subtree) declares `unsafe impl Send for Foo {}` alongside either `unsafe impl Sync for Foo {}` (a different trait) or `unsafe impl Send for Bar {}` (the same trait, a different self type)
- **THEN** the system emits two distinct findings (both the trait **and** the self type are part of the finding), so neither masks the other under the baseline

#### Scenario: Two same-named unsafe fns on different owners stay distinct

- **WHEN** `crate::net` (outside the subtree) declares `impl Foo { unsafe fn m(&self) {} }` alongside `impl Bar { unsafe fn m(&self) {} }` (the same method name, different owners), or two traits each declaring `unsafe fn m`
- **THEN** the system emits two distinct findings — each `unsafe fn` finding is qualified by its enclosing owner (`unsafe fn Foo::m`, the inherent-impl self type, or `unsafe fn A::m`, the declaring trait) — so neither masks the other under the baseline, the `unsafe fn` counterpart of the `unsafe impl` distinctness above

#### Scenario: A trait-impl unsafe fn stays distinct from the inherent method on the same type

- **WHEN** `crate::net` (outside the subtree) declares, on the **same** self type `Foo`, an inherent `impl Foo { unsafe fn m(&self) {} }` alongside `impl A for Foo { unsafe fn m(&self) {} }` and `impl B for Foo { unsafe fn m(&self) {} }` (safe traits `A`/`B`, so no independent `unsafe impl` finding)
- **THEN** the system emits three distinct findings — a trait-impl `unsafe fn` is qualified by `<trait for self>` (`unsafe fn <A for Foo>::m`, `unsafe fn <B for Foo>::m`), distinct from the inherent `unsafe fn Foo::m` and from each other — because self-type qualification alone separates only *different* self types, so on one self type a baseline of the inherent method would otherwise mask a later-added trait-impl `unsafe fn` (a false negative)

#### Scenario: Unsafe in a body-nested module is attributed to the enclosing module

- **WHEN** the crate has `only_under(["crate::ffi"])` and a function in `crate::net` declares `mod raw { pub unsafe fn poke() {} }` (a `mod` inside a fn body)
- **THEN** the system emits a violation for the `unsafe fn`, attributed to `crate::net`, never silently dropping it because the top-level walk did not descend a body-nested `mod`

#### Scenario: Unsafe under the allowed subtree is clean

- **WHEN** the crate has `only_under(["crate::ffi"])` and all `unsafe` (blocks, `fn`, `impl`, `trait`, `extern`) sits in `crate::ffi` or a submodule beneath it
- **THEN** the system reports no violation

#### Scenario: An observed unsafe site is never silently passed

- **WHEN** an `unsafe` site the scan observes lies outside every allowed subtree
- **THEN** the system emits a violation, never exit 0 for that boundary

#### Scenario: Unsafe in an unconditionally #[path]-relocated module reacts

- **WHEN** the crate has `only_under(["crate::ffi"])` and `crate::net` declares `#[path = "net_raw.rs"] mod raw;` where `net_raw.rs` contains an `unsafe fn`
- **THEN** the walk follows the `#[path]` to `net_raw.rs` and emits a violation for the `unsafe fn` attributed to `crate::net::raw`, never silently dropping it as off the conventional path

#### Scenario: Unsafe in a #[path] nested inside an inline module reacts at the accumulated file

- **WHEN** the crate root declares `mod inline { #[path = "other.rs"] mod inner; }`, `inline/other.rs` holds an `unsafe fn`, and a same-named `other.rs` decoy sits beside the crate root
- **THEN** the walk resolves `crate::inline::inner` to `inline/other.rs` (the enclosing inline-`mod` name accumulated onto the base, as rustc compiles it) and emits the `unsafe fn` violation, never reading the `other.rs` decoy and passing at exit 0 — the false negative this closes

#### Scenario: Unsafe in a cfg_attr-wrapped-path inline module reacts

- **WHEN** the crate has `only_under(["crate::ffi"])` and `crate::net` declares `#[cfg_attr(windows, path = "net_raw.rs")] mod raw { pub fn f() { unsafe {} } }` with no `net_raw.rs` present
- **THEN** the walk observes `raw`'s body regardless — `#[path]`, cfg-wrapped or not, has no effect on an inline module's own content — and emits a violation for the `unsafe` block, never silently dropping the whole body

#### Scenario: Unsafe in a cfg_attr-wrapped-path file module reacts, whichever candidate exists

- **WHEN** the crate has `only_under(["crate::ffi"])` and `crate::net` declares `#[cfg_attr(any(), path = "never.rs")] mod raw;` where `raw.rs` (the conventional file, present) contains an `unsafe fn` and `never.rs` (the target, absent) does not exist
- **THEN** the walk reads `raw.rs` — the file every build actually compiles here — and emits the violation, never treating the `cfg_attr` attribute as a bound to skip the module outright

### Requirement: Observation bounds and scope

The rule SHALL observe the executable-`unsafe` **code sites** (blocks, `fn`, `impl`, `trait`, `unsafe extern`); other lexical `unsafe` tokens and non-source `unsafe` SHALL be **stated bounds, never a silent claim of safety**:

- **Peripheral `unsafe` keywords, out of scope by design:** an `unsafe(...)` **attribute** (`#[unsafe(no_mangle)]`, Rust 2024 — a linkage assertion, not a code region), a bare **`unsafe fn` pointer type** (`type H = unsafe fn(...)` — a type signature, not an execution), and a **plain `extern "C" { … }` block** carrying no `unsafe` keyword (only the `unsafe extern {}` form is a site; the plain block's foreign-fn *call sites* are `unsafe {}` and DO react). The rule confines executable-`unsafe` code sites, not every lexical `unsafe` token.
- **Incidental bounds** (the dimension's inherited whole-crate-scan bounds): `unsafe` produced by a macro expansion or inside an unexpanded macro body is not observed; a module reached through an **unconditional** `#[path = "…"]` remap **is** observed (the walk follows it to its author-chosen file); a module reached only through a **`cfg_attr`-wrapped** `#[path]` is ALSO observed — an inline body regardless (the attribute has no effect on it), a file module's conventional file and its `cfg_attr` target both read when they exist on disk (cfg-blind union: neither is silently preferred), and only when NEITHER candidate exists, with no other cfg-conditional gate, is the module a genuine scan error; a `#[cfg]`-gated module absent when its feature is off is tolerated, while cfg-present code is observed **as written** (cfg-blind); a distinct `[lib] name` is a bound.

The system makes no claim about `unsafe` outside these observed sites.

#### Scenario: Macro-generated unsafe is a documented bound

- **WHEN** an `unsafe` block is produced by a macro expansion in a module outside the allowed subtree
- **THEN** the system does not claim to observe it (out of scope, the dimension's macro bound), rather than silently asserting the module is unsafe-free

