# external-crate-confinement Specification

## Purpose

Confine an **external** crate's imports to one declared module subtree — "the `libc` surface
may be used only under `crate::ffi`", the FFI/platform-vocabulary pattern. This is the middle
cell between crate-granularity dependency rules (`crate-dependency-boundary`, via `cargo
metadata` — a crate is depended on or not, whole-crate) and intra-crate module direction
(`module-boundary`'s `restrict_imports_to`, which by design never flags an external import):
"crate Y *may* depend on C, but C may be imported only under subtree S". `cargo metadata`
structurally cannot express this — it has no module resolution; only the source `use`-scan sees
the importing module. This is therefore the **first 圭表 rule that observes external-crate
imports**: it inverts, for this one rule only, the module scanner's standing "external imports
are out of scope" stance, while every other module rule keeps ignoring them. The confined
external crate is the violation `target` (the offending importer the `finding`), so `(target,
rule, finding)` is injective by structure — a baseline of one confined crate can never mask a
new violation of a different confined crate breached by the same importer. It reacts through
severity (default `enforce`, `warn` available) and the baseline exactly like every other module
rule, and introduces no new builder-intermediate type (one method on the existing module draft
stage). The rule is use-only and inherits the module scanner's stated bounds (cfg-blind,
`#[path]`-blind, macro-blind, `extern crate`-blind); a deeper **inline-symbol-path** layer
(`C::foo()` / `C::CONST` written with no `use`) is a stated forward bound, not this capability
(see `BACKLOG.md`). Not `cargo-deny`'s lane — declared and per-module, not resolved and
whole-graph.

## Requirements

### Requirement: External-crate confinement declared in Rust

A module boundary SHALL support a rule confining an **external** crate's imports to a declared module subtree, declared in Rust as `ModuleBoundary::in_crate(p).module(s).confine_external_crate(c).because("…")`, where `s` is the permitted subtree and `c` is the confined external crate name. It SHALL carry a severity (default `enforce`, `warn` available) like every other module rule, and SHALL be accepted by the umbrella `Boundary`. The declaration SHALL introduce no new builder-intermediate type: the method hangs off the same draft stage as the other module rules and finishes through the same `.warn()`/`.because()` path.

#### Scenario: The confinement holds its crate, permitted subtree, and reason

- **WHEN** a developer declares `ModuleBoundary::in_crate("app").module("crate::ffi").confine_external_crate("libc").because("…")`
- **THEN** the constitution holds a module boundary on crate `app`, whose permitted subtree is `crate::ffi`, confining the external crate `libc`, with a non-empty reason and default `enforce` severity

### Requirement: An external import outside the permitted subtree is a violation

The system SHALL scan every reachable file of the target crate and emit a violation when a file whose enclosing module is **not** within the permitted subtree `s` (neither `s` nor beneath it, the `::`-delimited "or beneath" test) imports the confined external crate `c` (a `use c::…` declaration). The violation SHALL name the **confined external crate `c`** as its `target` and the offending importing module as its `finding`. It SHALL react according to its severity (enforce fails with exit 1, warn is advisory) and any baseline, exactly as the other module rules do. Its repair polarity SHALL be `AllowlistGap` (the import is outside the permitted region — repair by moving it into `s` or widening the confinement).

#### Scenario: An import of the confined crate from outside the subtree violates

- **WHEN** a file in `crate::service` declares `use libc::c_int;` and the boundary is `module("crate::ffi").confine_external_crate("libc")`
- **THEN** the system emits a violation whose `target` is the confined crate `libc` and whose `finding` is the offending importer `crate::service`, and exits 1 at enforce severity

#### Scenario: A glob or bare import of the confined crate is observed

- **WHEN** a file in `crate::service` declares `use libc::*;` (or a bare `use libc;`) under the confinement of `libc` to `crate::ffi`
- **THEN** the system emits a violation naming `crate::service`, because the confined crate's head is observed regardless of a trailing glob or the absence of a sub-path

#### Scenario: An import of the confined crate from within the subtree is clean

- **WHEN** a file in `crate::ffi` declares `use libc::c_int;` under the same boundary
- **THEN** the system reports no violation, because the import is within the permitted subtree

#### Scenario: An import from beneath the permitted subtree is clean

- **WHEN** a file in `crate::ffi::raw` declares `use libc::c_int;` under the same boundary
- **THEN** the system reports no violation, because `crate::ffi::raw` is beneath the permitted subtree `crate::ffi`

#### Scenario: A prefix-colliding sibling of the permitted subtree is not permitted

- **WHEN** a file in `crate::ffi_utils` declares `use libc::c_int;` under `module("crate::ffi").confine_external_crate("libc")`
- **THEN** the system emits a violation, because the "or beneath" test is `::`-delimited: `crate::ffi_utils` is neither `crate::ffi` nor beneath `crate::ffi::`

### Requirement: Only the named external crate is observed

The rule SHALL observe only imports of the confined crate `c`; imports of any other external crate SHALL NOT be considered. An external crate that is confined but never imported anywhere in the target crate SHALL be clean — the rule performs no `cargo metadata` cross-check, so confining a crate the target does not import reacts exactly as forbidding a crate you do not depend on does (no violation, no error).

#### Scenario: A different external crate outside the subtree is ignored

- **WHEN** a file in `crate::service` declares `use serde::Deserialize;` under `module("crate::ffi").confine_external_crate("libc")`
- **THEN** the system reports no violation, because the rule observes only imports of `libc`

#### Scenario: A confined crate that is never imported is clean

- **WHEN** no file in the target crate imports `libc` and the boundary confines `libc` to `crate::ffi`
- **THEN** the system reports no violation and no constitution error, because the rule does not cross-check the dependency table

### Requirement: External imports observed with the scanner's existing resolution

The system SHALL observe the confined crate's imports using the **identical** external/internal resolution the module scanner already applies (the resolution by which the internal rules ignore external imports): a bare first segment in a submodule reaches the extern prelude and is external; a bare first segment in the crate root is external unless it names a crate-root `mod`-declared module; a leading-`::` path (`use ::c::…`) is the explicit external/global form and is external even when its head matches a crate-root module; raw identifiers are canonicalized; text inside comments, string literals, and macro bodies is stripped before scanning; `#[path]`-remapped modules, `cfg_attr`-wrapped path attributes, cfg-gated code, and the lib+bin conventional-path conflation remain the scanner's stated out-of-scope bounds. The confinement SHALL therefore observe an external import of `c` **exactly when** the internal rules would have ignored that import as external — one definition of "external," never a divergent one.

The rule is **use-only**, matching the scanner: an `extern crate c;` declaration SHALL NOT be observed (a stated, inherited bound, noted here because FFI crates occasionally still use `extern crate`). Because the scanner is cfg-blind and scans in-`src` inline modules, a `#[cfg(…)] use c::…` — including one inside a `#[cfg(test)] mod tests { … }` — outside the permitted subtree SHALL be observed and react, regardless of the active build configuration; this is aligned with the rule's intent (confinement is a source-location property, independent of platform), and consistent with how the internal rules already treat cfg-gated and test-module imports. Integration tests under `tests/` are a separate compilation target outside the lib/bin root and SHALL NOT be scanned.

#### Scenario: A confined name that shadows a crate-root module is resolved internal, not observed

- **WHEN** the target crate declares `mod libc;` (a crate-root module), a file at the crate root declares `use libc::helper;`, and the boundary confines the external crate `libc` to `crate::ffi`
- **THEN** the system reports no violation, because a root-file bare `use libc::…` with a crate-root `mod libc;` resolves to the internal `crate::libc` — it is not an external import — so the confinement does not observe it (no false positive)

#### Scenario: A submodule-bare and an explicit-external import of the confined crate are observed

- **WHEN** a file in `crate::service` (a submodule) declares `use libc::c_int;`, and another file declares `use ::libc::c_void;`, under the confinement of `libc` to `crate::ffi`
- **THEN** the system emits a violation for each, because a submodule's bare first segment reaches only the extern prelude and a leading-`::` path is the explicit external form — both are external imports of `libc` outside the permitted subtree

#### Scenario: A confined-crate use inside a string or macro body is not observed

- **WHEN** a file in `crate::service` contains a string literal or a macro body whose text is `use libc::c_int;`, and no real `use libc::…` outside it
- **THEN** the system reports no violation, because comments, string literals, and macro bodies are stripped before scanning, matching the scanner's stated bounds

### Requirement: Identity distinguishes the confined crate

The system SHALL make a violation's identity `(target, rule, finding)` distinguish the confined crate, by carrying the confined crate name as the `target`. A confinement of crate `c` and a confinement of a different crate `d`, both anchored on the same permitted subtree `s` and both breached by the same offending importer, SHALL therefore produce **distinct** identities. Were the confined crate absent from the identity, baselining one confined crate's violation would mask a new, un-baselined violation of the other confined crate from the same module — a false negative the identity model forbids.

#### Scenario: Baselining one confined crate does not mask another on the same subtree

- **WHEN** `crate::service` imports both `libc` and `winapi`, two boundaries confine `libc` and `winapi` respectively to `crate::ffi`, and the `libc` violation from `crate::service` is recorded in the baseline
- **THEN** the `winapi` violation from `crate::service` still reacts, because its `target` (`winapi`) differs from the baselined `libc` violation's `target` — the baseline masks only the confinement it recorded

### Requirement: A confinement reports each violation once

The system SHALL report each distinct confinement violation at most once, deduplicated by identity `(target, rule, finding)`. When one offending module imports the confined crate on multiple lines, or across multiple files of the same enclosing module, the system SHALL emit a single violation for that (subtree, confined-crate, importer) triple, not one per import site.

#### Scenario: One importer importing the confined crate on many lines yields one violation

- **WHEN** a file in `crate::service` declares `use libc::c_int;` and `use libc::c_void;` under the confinement of `libc` to `crate::ffi`
- **THEN** the system emits exactly one violation for `libc` imported by `crate::service`, not two

### Requirement: The permitted subtree is a reachable file-based module

The permitted subtree `s` SHALL be a reachable, file-based module of the target crate, with the same constitution-error handling as every other module target: a path that is not reachable at all is an unknown-module constitution error (exit 2), and a reachable-but-inline (`mod s { … }`, file-less) path is the self-describing inline-module constitution error (exit 2). Neither is a silent pass.

#### Scenario: An unknown permitted subtree is a constitution error

- **WHEN** a confinement's permitted subtree names a module path not reachable in the target crate (e.g. a typo)
- **THEN** the system reports a constitution error (exit 2) that the module was not found, never a silent pass

### Requirement: Confining to the crate root is a constitution error

Declaring the confinement with `s` = `crate` (the crate root) SHALL be a self-describing constitution error (exit 2), distinct from a boundary violation and never a silent pass, because confining a crate to the whole root permits it everywhere and the rule could never react — the same "the rule could never react as declared" reasoning that makes the closed-allowlist module rules on `crate` an error.

#### Scenario: Confining an external crate to the crate root is a constitution error

- **WHEN** a boundary declares `confine_external_crate(c)` on `crate` (the crate root)
- **THEN** the system emits a self-describing constitution error and exits 2 — naming `crate` and that confining to the root permits the crate everywhere — distinct from a boundary violation

### Requirement: Raw identifiers in the subtree and crate name are canonicalized

The system SHALL canonicalize raw identifiers (`r#name` → `name`) in both the permitted subtree path and the confined crate name, matching how it canonicalizes observed module and `use` paths, so a boundary MAY be declared with either form and SHALL match the observed imports regardless of which form the source uses.

#### Scenario: A raw-identifier confined crate name matches its plain-form import

- **WHEN** the boundary confines `r#match` to `crate::ffi` and a file in `crate::service` declares `use r#match::Thing;` (equivalently `use match::…` where the source uses the raw form)
- **THEN** the system emits a violation, the raw and plain forms of the crate name having been canonicalized to one identity

### Requirement: The confinement projects like the other module rules

The rule SHALL appear in the constitution projection (text and JSON) alongside the other module rules. Its JSON parameter SHALL name the confined crate under a self-describing key `external_crate`, and its text projection SHALL state the confined crate and the permitted subtree, so the projection is legible without reading the rule label.

#### Scenario: The confinement renders its confined crate in the projection

- **WHEN** the constitution is projected and it contains `module("crate::ffi").confine_external_crate("libc")`
- **THEN** the JSON projection carries `"external_crate": "libc"` for that rule and the text projection names both `libc` and `crate::ffi`
